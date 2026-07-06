mod verifier;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    application::{CaptchaError, CaptchaProvider, CaptchaResult, CaptchaSettings},
    providers::config::provider_config,
};

pub use verifier::{CloudflareTurnstileVerifier, CloudflareTurnstileVerifyRequest, CloudflareTurnstileVerifyResponse, ReqwestTurnstileVerifier};

const PROVIDER_NAME: &str = "cloudflare_turnstile";
const SCRIPT_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit";
const DEFAULT_THEME: &str = "auto";
const DEFAULT_SIZE: &str = "normal";

pub struct CloudflareTurnstileProvider<V> {
    verifier: V,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
struct TurnstilePublicConfig {
    site_key: String,
    #[serde(default)]
    theme: Option<String>,
    #[serde(default)]
    size: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
struct TurnstilePrivateConfig {
    secret_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct TurnstilePublicResponse {
    site_key: String,
    script_url: &'static str,
    theme: String,
    size: String,
}

impl<V> CloudflareTurnstileProvider<V>
where
    V: CloudflareTurnstileVerifier,
{
    pub const fn new(verifier: V) -> Self {
        Self { verifier }
    }

    fn public_response(&self, settings: &CaptchaSettings) -> CaptchaResult<TurnstilePublicResponse> {
        let config = public_config(settings)?;
        Ok(TurnstilePublicResponse {
            site_key: required_value("cloudflare turnstile site_key", &config.site_key)?,
            script_url: SCRIPT_URL,
            theme: optional_value(config.theme, DEFAULT_THEME),
            size: optional_value(config.size, DEFAULT_SIZE),
        })
    }
}

#[async_trait]
impl<V> CaptchaProvider for CloudflareTurnstileProvider<V>
where
    V: CloudflareTurnstileVerifier,
{
    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    async fn public_config(&self, settings: &CaptchaSettings) -> CaptchaResult<Value> {
        to_value(self.public_response(settings)?)
    }

    async fn challenge(&self, _settings: &CaptchaSettings) -> CaptchaResult<Value> {
        Err(CaptchaError::InvalidInput(
            "cloudflare_turnstile does not support captcha challenge endpoint".into(),
        ))
    }

    async fn redeem(&self, _settings: &CaptchaSettings, _payload: Value) -> CaptchaResult<Value> {
        Err(CaptchaError::InvalidInput(
            "cloudflare_turnstile does not support captcha redeem endpoint".into(),
        ))
    }

    async fn verify(&self, settings: &CaptchaSettings, token: Option<&str>) -> CaptchaResult<()> {
        let token = required_token(token)?;
        let private = private_config(settings)?;
        let secret = required_value("cloudflare turnstile secret_key", &private.secret_key)?;
        let response = self.verifier.verify(CloudflareTurnstileVerifyRequest::new(secret, token)).await?;
        validate_response(response)
    }
}

fn public_config(settings: &CaptchaSettings) -> CaptchaResult<TurnstilePublicConfig> {
    serde_json::from_value(provider_config(&settings.public_config, PROVIDER_NAME).clone()).map_err(invalid_public_config)
}

fn private_config(settings: &CaptchaSettings) -> CaptchaResult<TurnstilePrivateConfig> {
    serde_json::from_value(provider_config(&settings.private_config, PROVIDER_NAME).clone()).map_err(invalid_private_config)
}

fn validate_response(response: CloudflareTurnstileVerifyResponse) -> CaptchaResult<()> {
    if response.success {
        return Ok(());
    }
    if response.has_secret_error() {
        return Err(CaptchaError::Infrastructure(response.failure_message()));
    }
    Err(CaptchaError::InvalidInput(response.failure_message()))
}

fn required_token(token: Option<&str>) -> CaptchaResult<String> {
    let Some(token) = token.map(str::trim).filter(|value| !value.is_empty()) else {
        return Err(CaptchaError::InvalidInput("captcha verification is required".into()));
    };
    Ok(token.to_owned())
}

fn required_value(name: &str, value: &str) -> CaptchaResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(CaptchaError::InvalidInput(format!("{name} is required")));
    }
    Ok(value.to_owned())
}

fn optional_value(value: Option<String>, default: &str) -> String {
    value
        .map(|item| item.trim().to_owned())
        .filter(|item| !item.is_empty())
        .unwrap_or_else(|| default.into())
}

fn invalid_public_config(error: serde_json::Error) -> CaptchaError {
    CaptchaError::InvalidInput(format!("invalid cloudflare turnstile public config: {error}"))
}

fn invalid_private_config(error: serde_json::Error) -> CaptchaError {
    CaptchaError::InvalidInput(format!("invalid cloudflare turnstile private config: {error}"))
}

fn to_value<T: Serialize>(value: T) -> CaptchaResult<Value> {
    serde_json::to_value(value).map_err(json_error)
}

fn json_error(error: serde_json::Error) -> CaptchaError {
    CaptchaError::Infrastructure(format!("captcha json error: {error}"))
}

pub fn default_public_config_template() -> Value {
    json!({
        "cap": {},
        PROVIDER_NAME: {
            "site_key": "",
            "theme": DEFAULT_THEME,
            "size": DEFAULT_SIZE,
        }
    })
}

pub fn default_private_config_template() -> Value {
    json!({
        "cap": {},
        PROVIDER_NAME: {
            "secret_key": ""
        }
    })
}

#[cfg(test)]
mod tests;
