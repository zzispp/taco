use async_trait::async_trait;
use kernel::error::LocalizedError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{CaptchaError, CaptchaResult};

#[derive(Clone, Debug, PartialEq)]
pub struct CaptchaSettings {
    pub enabled: bool,
    pub provider: String,
    pub providers: Value,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct CaptchaConfigDocument {
    pub enabled: bool,
    pub provider: String,
    pub providers: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CaptchaConfigResponse {
    pub enabled: bool,
    pub provider: String,
    pub public_config: Value,
}

impl CaptchaSettings {
    pub fn provider_config(&self, provider: &str) -> CaptchaResult<&Value> {
        self.providers.get(provider).filter(|item| item.is_object()).ok_or_else(|| {
            CaptchaError::InvalidInput(LocalizedError::new("errors.captcha.provider_config_required").with_param("provider", provider.to_owned()))
        })
    }
}

impl TryFrom<CaptchaConfigDocument> for CaptchaSettings {
    type Error = CaptchaError;

    fn try_from(value: CaptchaConfigDocument) -> Result<Self, Self::Error> {
        let provider = value.provider.trim().to_owned();
        if provider.is_empty() {
            return Err(CaptchaError::InvalidInput(LocalizedError::new("errors.captcha.provider_required")));
        }
        if !value.providers.is_object() {
            return Err(CaptchaError::InvalidInput(LocalizedError::new("errors.captcha.providers_required")));
        }
        Ok(Self {
            enabled: value.enabled,
            provider,
            providers: value.providers,
        })
    }
}

#[async_trait]
pub trait CaptchaSettingsReader: Send + Sync + 'static {
    async fn config(&self) -> CaptchaResult<Value>;
}

#[async_trait]
pub trait CaptchaProvider: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    async fn public_config(&self, settings: &CaptchaSettings) -> CaptchaResult<Value>;
    async fn challenge(&self, settings: &CaptchaSettings) -> CaptchaResult<Value>;
    async fn redeem(&self, settings: &CaptchaSettings, payload: Value) -> CaptchaResult<Value>;
    async fn verify(&self, settings: &CaptchaSettings, token: Option<&str>) -> CaptchaResult<()>;
}

#[async_trait]
pub trait CaptchaUseCase: Send + Sync + 'static {
    async fn config(&self) -> CaptchaResult<CaptchaConfigResponse>;
    async fn challenge(&self) -> CaptchaResult<Value>;
    async fn redeem(&self, payload: Value) -> CaptchaResult<Value>;
    async fn verify_account(&self, token: Option<&str>) -> CaptchaResult<()>;
}
