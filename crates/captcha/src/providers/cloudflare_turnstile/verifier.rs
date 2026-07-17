use std::time::Instant;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use taco_tracing::{InfrastructureDependency, InfrastructureObserver};

use crate::application::{CaptchaError, CaptchaResult};

const SITEVERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";
const SECRET_ERROR_MISSING: &str = "missing-input-secret";
const SECRET_ERROR_INVALID: &str = "invalid-input-secret";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CloudflareTurnstileVerifyRequest {
    pub secret: String,
    pub response: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct CloudflareTurnstileVerifyResponse {
    pub success: bool,
    #[serde(default, rename = "error-codes")]
    pub error_codes: Vec<String>,
}

#[async_trait]
pub trait CloudflareTurnstileVerifier: Send + Sync + 'static {
    async fn verify(&self, request: CloudflareTurnstileVerifyRequest) -> CaptchaResult<CloudflareTurnstileVerifyResponse>;
}

#[derive(Clone)]
pub struct ReqwestTurnstileVerifier {
    client: reqwest::Client,
    endpoint: String,
    observer: InfrastructureObserver,
}

impl CloudflareTurnstileVerifyRequest {
    pub fn new(secret: String, response: String) -> Self {
        Self { secret, response }
    }
}

impl CloudflareTurnstileVerifyResponse {
    pub fn has_secret_error(&self) -> bool {
        self.error_codes.iter().any(|code| code == SECRET_ERROR_MISSING || code == SECRET_ERROR_INVALID)
    }

    pub fn failure_message(&self) -> String {
        if self.error_codes.is_empty() {
            return "cloudflare turnstile verification failed".into();
        }
        format!("cloudflare turnstile verification failed: {}", self.error_codes.join(","))
    }
}

impl ReqwestTurnstileVerifier {
    pub fn new(observer: InfrastructureObserver) -> Self {
        Self::with_endpoint(SITEVERIFY_URL.into(), observer)
    }

    pub fn with_endpoint(endpoint: String, observer: InfrastructureObserver) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint,
            observer,
        }
    }
}

#[async_trait]
impl CloudflareTurnstileVerifier for ReqwestTurnstileVerifier {
    async fn verify(&self, request: CloudflareTurnstileVerifyRequest) -> CaptchaResult<CloudflareTurnstileVerifyResponse> {
        let started = Instant::now();
        let result = request_verification(&self.client, &self.endpoint, request).await;
        self.observer
            .record(InfrastructureDependency::OutboundHttp, "turnstile_verify", started.elapsed(), result.is_ok());
        result
    }
}

async fn request_verification(
    client: &reqwest::Client,
    endpoint: &str,
    request: CloudflareTurnstileVerifyRequest,
) -> CaptchaResult<CloudflareTurnstileVerifyResponse> {
    let response = client.post(endpoint).json(&request).send().await.map_err(reqwest_error)?;
    let status = response.status();
    if !status.is_success() {
        return Err(CaptchaError::Infrastructure(format!("cloudflare turnstile siteverify returned HTTP {status}")));
    }
    response.json().await.map_err(reqwest_error)
}

fn reqwest_error(error: reqwest::Error) -> CaptchaError {
    CaptchaError::Infrastructure(format!("cloudflare turnstile siteverify error: {error}"))
}
