use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;

use crate::{
    application::{CaptchaError, CaptchaProvider, CaptchaResult, CaptchaSettings},
    providers::cloudflare_turnstile::{
        CloudflareTurnstileProvider, CloudflareTurnstileVerifier, CloudflareTurnstileVerifyRequest, CloudflareTurnstileVerifyResponse, default_config_template,
    },
};

#[tokio::test]
async fn public_config_returns_frontend_turnstile_settings() {
    let provider = provider(success_response());
    let config = provider.public_config(&settings()).await.expect("public config must parse");

    assert_eq!(config["site_key"], "site-key-1");
    assert_eq!(config["theme"], "auto");
    assert_eq!(config["size"], "normal");
    assert_eq!(config["script_url"], "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit");
}

#[tokio::test]
async fn verify_sends_secret_and_token_to_verifier() {
    let verifier = TestVerifier::new(success_response());
    let calls = verifier.calls.clone();
    let provider = CloudflareTurnstileProvider::new(verifier, "secret-key-1".into());

    provider.verify(&settings(), Some(" token-1 ")).await.expect("token must pass");

    assert_eq!(
        calls.lock().unwrap().as_slice(),
        &[CloudflareTurnstileVerifyRequest::new("secret-key-1".into(), "token-1".into())]
    );
}

#[tokio::test]
async fn public_config_rejects_private_or_unknown_fields() {
    for field in ["secret_key", "unexpected"] {
        let provider = provider(success_response());
        let settings = settings_with_extra_field(field);

        let error = provider.public_config(&settings).await.expect_err("unknown public field must fail");

        assert!(matches!(error, CaptchaError::InvalidInput(message) if message.key() == "errors.captcha.invalid_public_config"));
    }
}

#[test]
fn default_config_template_excludes_private_secret() {
    let config = default_config_template();

    assert_eq!(config.get("secret_key"), None);
}

#[tokio::test]
async fn verify_rejects_missing_token() {
    let provider = provider(success_response());

    let error = provider.verify(&settings(), None).await.expect_err("missing token must fail");

    assert!(matches!(error, CaptchaError::InvalidInput(message) if message.key() == "errors.captcha.verification_required"));
}

#[tokio::test]
async fn verify_rejects_blank_injected_secret_without_calling_verifier() {
    let verifier = TestVerifier::new(success_response());
    let calls = verifier.calls.clone();
    let provider = CloudflareTurnstileProvider::new(verifier, "   ".into());

    let error = provider.verify(&settings(), Some("token-1")).await.expect_err("blank secret must fail");

    assert!(matches!(error, CaptchaError::InvalidInput(message) if message.key() == "errors.captcha.field_required"));
    assert_eq!(calls.lock().unwrap().as_slice(), &[]);
}

#[tokio::test]
async fn verify_maps_failed_token_to_invalid_input() {
    let provider = provider(CloudflareTurnstileVerifyResponse {
        success: false,
        error_codes: vec!["invalid-input-response".into()],
    });

    let error = provider.verify(&settings(), Some("bad-token")).await.expect_err("invalid token must fail");

    assert!(matches!(error, CaptchaError::InvalidInput(message) if message.key() == "errors.captcha.verification_failed"));
}

#[tokio::test]
async fn verify_maps_secret_errors_to_infrastructure_error() {
    let provider = provider(CloudflareTurnstileVerifyResponse {
        success: false,
        error_codes: vec!["invalid-input-secret".into()],
    });

    let error = provider.verify(&settings(), Some("token-1")).await.expect_err("invalid secret must fail");

    assert!(matches!(error, crate::application::CaptchaError::Infrastructure(_)));
    assert_eq!(error.to_string(), "cloudflare turnstile verification failed: invalid-input-secret");
}

fn provider(response: CloudflareTurnstileVerifyResponse) -> CloudflareTurnstileProvider<TestVerifier> {
    CloudflareTurnstileProvider::new(TestVerifier::new(response), "secret-key-1".into())
}

fn settings() -> CaptchaSettings {
    let mut config = default_config_template();
    config["site_key"] = json!("site-key-1");
    CaptchaSettings {
        enabled: true,
        provider: "cloudflare_turnstile".into(),
        providers: json!({
            "cloudflare_turnstile": config
        }),
    }
}

fn settings_with_extra_field(field: &str) -> CaptchaSettings {
    let mut settings = settings();
    settings.providers["cloudflare_turnstile"][field] = json!("must-be-rejected");
    settings
}

fn success_response() -> CloudflareTurnstileVerifyResponse {
    CloudflareTurnstileVerifyResponse {
        success: true,
        error_codes: vec![],
    }
}

#[derive(Clone)]
struct TestVerifier {
    response: CloudflareTurnstileVerifyResponse,
    calls: Arc<Mutex<Vec<CloudflareTurnstileVerifyRequest>>>,
}

impl TestVerifier {
    fn new(response: CloudflareTurnstileVerifyResponse) -> Self {
        Self {
            response,
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl CloudflareTurnstileVerifier for TestVerifier {
    async fn verify(&self, request: CloudflareTurnstileVerifyRequest) -> CaptchaResult<CloudflareTurnstileVerifyResponse> {
        self.calls.lock().unwrap().push(request);
        Ok(self.response.clone())
    }
}
