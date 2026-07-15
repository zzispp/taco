use std::sync::Arc;

use async_trait::async_trait;
use kernel::error::LocalizedError;
use serde_json::Value;

use crate::application::{
    CaptchaConfigDocument, CaptchaConfigResponse, CaptchaError, CaptchaProvider, CaptchaResult, CaptchaSettings, CaptchaSettingsReader, CaptchaUseCase,
};

pub struct CaptchaService<S> {
    settings: S,
    providers: Vec<Arc<dyn CaptchaProvider>>,
}

impl<S> CaptchaService<S>
where
    S: CaptchaSettingsReader,
{
    pub fn new(settings: S, providers: Vec<Arc<dyn CaptchaProvider>>) -> Self {
        Self { settings, providers }
    }

    async fn settings(&self) -> CaptchaResult<CaptchaSettings> {
        let document = serde_json::from_value::<CaptchaConfigDocument>(self.settings.config().await?).map_err(invalid_config)?;
        CaptchaSettings::try_from(document)
    }

    fn provider(&self, name: &str) -> CaptchaResult<&dyn CaptchaProvider> {
        self.providers
            .iter()
            .find(|provider| provider.name() == name)
            .map(Arc::as_ref)
            .ok_or_else(|| CaptchaError::InvalidInput(LocalizedError::new("errors.captcha.unsupported_provider").with_param("provider", name.to_owned())))
    }
}

#[async_trait]
impl<S> CaptchaUseCase for CaptchaService<S>
where
    S: CaptchaSettingsReader,
{
    async fn config(&self) -> CaptchaResult<CaptchaConfigResponse> {
        let settings = self.settings().await?;
        let provider = self.provider(&settings.provider)?;
        let public_config = provider.public_config(&settings).await?;
        Ok(CaptchaConfigResponse {
            enabled: settings.enabled,
            provider: settings.provider,
            public_config,
        })
    }

    async fn challenge(&self) -> CaptchaResult<Value> {
        let settings = self.settings().await?;
        self.provider(&settings.provider)?.challenge(&settings).await
    }

    async fn redeem(&self, payload: Value) -> CaptchaResult<Value> {
        let settings = self.settings().await?;
        self.provider(&settings.provider)?.redeem(&settings, payload).await
    }

    async fn verify_account(&self, token: Option<&str>) -> CaptchaResult<()> {
        let settings = self.settings().await?;
        if !settings.enabled {
            return Ok(());
        }
        self.provider(&settings.provider)?.verify(&settings, token).await
    }
}

fn invalid_config(error: serde_json::Error) -> CaptchaError {
    hook_tracing::error_with_fields!("invalid captcha config document", &error, key = constants::system_config::CAPTCHA_CONFIG_KEY);
    super::invalid_captcha_config_json()
}
