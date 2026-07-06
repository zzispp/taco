use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;

use super::CaptchaResult;

#[derive(Clone, Debug, PartialEq)]
pub struct CaptchaSettings {
    pub enabled: bool,
    pub provider: String,
    pub public_config: Value,
    pub private_config: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CaptchaConfigResponse {
    pub enabled: bool,
    pub provider: String,
    pub public_config: Value,
}

#[async_trait]
pub trait CaptchaSettingsReader: Send + Sync + 'static {
    async fn enabled(&self) -> CaptchaResult<bool>;
    async fn provider(&self) -> CaptchaResult<String>;
    async fn public_config(&self) -> CaptchaResult<Value>;
    async fn private_config(&self) -> CaptchaResult<Value>;
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
