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
    assert_eq!(config.public_config, json!({}));
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
    let redeem = service.redeem(redeem_payload(&challenge)).await.expect("redeem must succeed");
    let token = redeem["token"].as_str().expect("redeem token must exist");

    service.verify_account(Some(token)).await.expect("first token use must pass");
    let error = service.verify_account(Some(token)).await.expect_err("second token use must fail");

    assert_eq!(redeem["success"], true);
    assert!(matches!(error, CaptchaError::InvalidInput(message) if message.key() == "errors.captcha.verification_failed"));
}

fn service(settings: TestSettings, store: TestStore) -> CaptchaService<TestSettings> {
    let provider = CapProvider::with_options(store, test_options());
    CaptchaService::new(settings, vec![Arc::new(provider) as Arc<dyn CaptchaProvider>])
}

fn settings(enabled: bool, provider: &str) -> TestSettings {
    TestSettings {
        enabled,
        provider: provider.into(),
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

#[derive(Clone)]
struct TestSettings {
    enabled: bool,
    provider: String,
}

#[async_trait]
impl CaptchaSettingsReader for TestSettings {
    async fn config(&self) -> CaptchaResult<Value> {
        Ok(json!({
            "enabled": self.enabled,
            "provider": self.provider,
            "providers": {
                "cap": {}
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
