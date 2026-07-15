use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use serde_json::{Value, json};

use crate::{
    application::{CaptchaError, CaptchaProvider, CaptchaResult, CaptchaService, CaptchaSettingsReader, CaptchaUseCase},
    providers::cap::{CapChallengeRecord, CapOptions, CapProvider, CapStore, solve_for_test},
};

#[tokio::test]
async fn config_dispatches_to_selected_provider() {
    let service = service(settings(true, "cap"), store());

    let config = service.config().await.expect("config must load");

    assert!(config.enabled);
    assert_eq!(config.provider, "cap");
    assert_eq!(config.public_config, serde_json::to_value(test_options()).unwrap());
}

#[tokio::test]
async fn unknown_provider_is_explicit_error() {
    let service = service(settings(true, "missing"), store());

    let error = service.config().await.expect_err("unknown provider must fail");

    assert!(matches!(error, CaptchaError::InvalidInput(message) if message.key() == "errors.captcha.unsupported_provider"));
}

#[tokio::test]
async fn verify_account_allows_missing_token_when_disabled() {
    let service = service(settings(false, "cap"), store());

    let result = service.verify_account(None).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn verify_account_requires_token_when_enabled() {
    let service = service(settings(true, "cap"), store());

    let error = service.verify_account(None).await.expect_err("missing token must fail");

    assert!(matches!(error, CaptchaError::InvalidInput(message) if message.key() == "errors.captcha.verification_required"));
}

#[tokio::test]
async fn cap_challenge_redeem_and_verify_consumes_token_once() {
    let service = service(settings(true, "cap"), store());
    let challenge = service.challenge().await.expect("challenge must be created");
    assert_wire_expiry(&challenge);
    let redeem = service.redeem(redeem_payload(&challenge)).await.expect("redeem must succeed");
    assert_wire_expiry(&redeem);
    let token = redeem["token"].as_str().expect("redeem token must exist");

    service.verify_account(Some(token)).await.expect("first token use must pass");
    let error = service.verify_account(Some(token)).await.expect_err("second token use must fail");

    assert_eq!(redeem["success"], true);
    assert!(matches!(error, CaptchaError::InvalidInput(message) if message.key() == "errors.captcha.verification_failed"));
}

#[tokio::test]
async fn cap_options_must_be_complete_positive_and_non_overflowing() {
    for options in [
        json!({}),
        json!({"challenge_count":0,"challenge_size":8,"challenge_difficulty":1,"challenge_ttl_seconds":60,"redeemed_token_ttl_seconds":60}),
        json!({"challenge_count":1,"challenge_size":8,"challenge_difficulty":1,"challenge_ttl_seconds":u64::MAX,"redeemed_token_ttl_seconds":60}),
    ] {
        let error = service(settings_with_options(true, "cap", options), store())
            .challenge()
            .await
            .expect_err("invalid CAP options must fail");
        assert!(matches!(error, CaptchaError::InvalidInput(message) if message.key() == "errors.captcha.invalid_provider_config"));
    }
}

fn service(settings: TestSettings, store: TestStore) -> CaptchaService<TestSettings> {
    let provider = CapProvider::new(store);
    CaptchaService::new(settings, vec![Arc::new(provider) as Arc<dyn CaptchaProvider>])
}

fn settings(enabled: bool, provider: &str) -> TestSettings {
    settings_with_options(enabled, provider, serde_json::to_value(test_options()).unwrap())
}

fn settings_with_options(enabled: bool, provider: &str, cap_options: Value) -> TestSettings {
    TestSettings {
        enabled,
        provider: provider.into(),
        cap_options,
    }
}

fn store() -> TestStore {
    TestStore::default()
}

fn test_options() -> CapOptions {
    CapOptions {
        challenge_count: 1,
        challenge_size: 8,
        challenge_difficulty: 1,
        challenge_ttl_seconds: 60,
        redeemed_token_ttl_seconds: 60,
    }
}

fn redeem_payload(challenge: &Value) -> Value {
    let token = challenge["token"].as_str().expect("challenge token must exist");
    let spec = serde_json::from_value(challenge["challenge"].clone()).expect("challenge spec must parse");
    let solution = solve_for_test(token, &spec, 1);
    json!({ "token": token, "solutions": [solution] })
}

fn assert_wire_expiry(response: &Value) {
    let expires = response["expires"].as_str().expect("CAP expiry must be an RFC3339 string");
    assert_eq!(expires.len(), "2026-07-15T09:23:45.000Z".len());
    assert_eq!(&expires[19..20], ".");
    assert!(expires.ends_with('Z'));
}

#[derive(Clone)]
struct TestSettings {
    enabled: bool,
    provider: String,
    cap_options: Value,
}

#[async_trait]
impl CaptchaSettingsReader for TestSettings {
    async fn config(&self) -> CaptchaResult<Value> {
        Ok(json!({
            "enabled": self.enabled,
            "provider": self.provider,
            "providers": {
                "cap": self.cap_options
            }
        }))
    }
}

#[derive(Clone, Default)]
struct TestStore {
    state: Arc<Mutex<TestStoreState>>,
}

#[derive(Default)]
struct TestStoreState {
    challenges: HashMap<String, CapChallengeRecord>,
    redeemed: HashSet<String>,
}

#[async_trait]
impl CapStore for TestStore {
    async fn save_challenge(&self, token: &str, record: &CapChallengeRecord, _ttl_seconds: u64) -> CaptchaResult<()> {
        self.state.lock().unwrap().challenges.insert(token.into(), record.clone());
        Ok(())
    }

    async fn consume_challenge(&self, token: &str) -> CaptchaResult<Option<CapChallengeRecord>> {
        Ok(self.state.lock().unwrap().challenges.remove(token))
    }

    async fn save_redeemed(&self, token_key: &str, _expires: i64, _ttl_seconds: u64) -> CaptchaResult<()> {
        self.state.lock().unwrap().redeemed.insert(token_key.into());
        Ok(())
    }

    async fn consume_redeemed(&self, token_key: &str) -> CaptchaResult<bool> {
        Ok(self.state.lock().unwrap().redeemed.remove(token_key))
    }
}
