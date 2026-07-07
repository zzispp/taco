use std::sync::Arc;

use async_trait::async_trait;
use captcha::application::{CaptchaError, CaptchaSettingsReader, CaptchaUseCase};
use constants::system_config::{AVATAR_CONFIG_KEY, CAPTCHA_CONFIG_KEY, EXPORT_BATCH_CONFIG_KEY, PASSWORD_POLICY_KEY, TOKEN_CONFIG_KEY};
use kernel::runtime_config::{ExportBatchConfig, ExportConfigProvider};
use rbac::application::RbacError;
use serde_json::Value;
use system::application::{SystemError, SystemUseCase};

const REQUIRED_CAPTCHA_SYSTEM_CONFIG_ERROR: &str = "infra.system_config.captcha_required_missing";
const REQUIRED_SYSTEM_CONFIG_ERROR: &str = "infra.system_config.required_missing";

use user::{
    api::{TokenSettingsReader, TokenTtlConfig, parse_token_ttl_config},
    application::{
        AccountVerifier, AppError, AppResult, AvatarConfig, AvatarConfigProvider, PasswordPolicy, PasswordPolicyProvider, SystemConfigProvider,
        parse_avatar_config, parse_export_batch_config, parse_password_policy,
    },
};

pub(super) struct CaptchaSystemConfig {
    system: Arc<dyn SystemUseCase>,
}

impl CaptchaSystemConfig {
    pub(super) fn new(system: Arc<dyn SystemUseCase>) -> Self {
        Self { system }
    }
}

#[async_trait]
impl CaptchaSettingsReader for CaptchaSystemConfig {
    async fn config(&self) -> Result<Value, CaptchaError> {
        let value = self.system.config_by_key(CAPTCHA_CONFIG_KEY).await.map_err(captcha_config_error)?;
        serde_json::from_str(&value).map_err(captcha_json_error)
    }
}

#[derive(Clone)]
pub(super) struct RuntimeUserConfig {
    system: Arc<dyn SystemUseCase>,
}

impl RuntimeUserConfig {
    pub(super) fn new(system: Arc<dyn SystemUseCase>) -> Self {
        Self { system }
    }

    async fn user_config(&self, key: &str) -> AppResult<String> {
        self.system.config_by_key(key).await.map_err(user_config_error)
    }
}

#[async_trait]
impl SystemConfigProvider for RuntimeUserConfig {
    async fn config_by_key(&self, key: &str) -> Result<String, AppError> {
        self.user_config(key).await
    }
}

#[async_trait]
impl PasswordPolicyProvider for RuntimeUserConfig {
    async fn password_policy(&self) -> AppResult<PasswordPolicy> {
        parse_password_policy(&self.user_config(PASSWORD_POLICY_KEY).await?)
    }
}

#[async_trait]
impl AvatarConfigProvider for RuntimeUserConfig {
    async fn avatar_config(&self) -> AppResult<AvatarConfig> {
        parse_avatar_config(&self.user_config(AVATAR_CONFIG_KEY).await?)
    }
}

#[async_trait]
impl ExportConfigProvider for RuntimeUserConfig {
    type Error = AppError;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error> {
        parse_export_batch_config(&self.user_config(EXPORT_BATCH_CONFIG_KEY).await?)
    }
}

#[async_trait]
impl TokenSettingsReader for RuntimeUserConfig {
    async fn token_ttl_config(&self) -> AppResult<TokenTtlConfig> {
        parse_token_ttl_config(&self.user_config(TOKEN_CONFIG_KEY).await?)
    }
}

#[derive(Clone)]
pub(super) struct RuntimeRbacConfig {
    system: Arc<dyn SystemUseCase>,
}

impl RuntimeRbacConfig {
    pub(super) fn new(system: Arc<dyn SystemUseCase>) -> Self {
        Self { system }
    }
}

#[async_trait]
impl ExportConfigProvider for RuntimeRbacConfig {
    type Error = RbacError;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error> {
        let value = self.system.config_by_key(EXPORT_BATCH_CONFIG_KEY).await.map_err(rbac_config_error)?;
        parse_export_batch_config(&value).map_err(user_error_to_rbac)
    }
}

#[derive(Clone)]
pub(super) struct RuntimeSystemConfig {
    system: Arc<dyn SystemUseCase>,
}

impl RuntimeSystemConfig {
    pub(super) fn new(system: Arc<dyn SystemUseCase>) -> Self {
        Self { system }
    }
}

#[async_trait]
impl ExportConfigProvider for RuntimeSystemConfig {
    type Error = SystemError;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error> {
        let value = self.system.config_by_key(EXPORT_BATCH_CONFIG_KEY).await?;
        parse_export_batch_config(&value).map_err(user_error_to_system)
    }
}

pub(super) struct CaptchaAccountVerifier {
    captcha: Arc<dyn CaptchaUseCase>,
}

impl CaptchaAccountVerifier {
    pub(super) fn new(captcha: Arc<dyn CaptchaUseCase>) -> Self {
        Self { captcha }
    }
}

#[async_trait]
impl AccountVerifier for CaptchaAccountVerifier {
    async fn verify_account(&self, token: Option<&str>) -> AppResult<()> {
        self.captcha.verify_account(token).await.map_err(captcha_account_error)
    }
}

fn captcha_json_error(error: serde_json::Error) -> CaptchaError {
    let _ = error;
    CaptchaError::InvalidInput(kernel::error::LocalizedError::new("errors.captcha.invalid_config_json").with_param("key", CAPTCHA_CONFIG_KEY))
}

fn captcha_config_error(error: SystemError) -> CaptchaError {
    match error {
        SystemError::NotFound => CaptchaError::Infrastructure(REQUIRED_CAPTCHA_SYSTEM_CONFIG_ERROR.into()),
        SystemError::Forbidden(message) | SystemError::Conflict(message) | SystemError::InvalidInput(message) => CaptchaError::InvalidInput(message),
        SystemError::Infrastructure(message) => CaptchaError::Infrastructure(message),
    }
}

fn user_config_error(error: SystemError) -> AppError {
    match error {
        SystemError::NotFound => AppError::Infrastructure(REQUIRED_SYSTEM_CONFIG_ERROR.into()),
        SystemError::Forbidden(message) => AppError::Forbidden(message),
        SystemError::Conflict(message) => AppError::Conflict(message),
        SystemError::InvalidInput(message) => AppError::InvalidInput(message),
        SystemError::Infrastructure(message) => AppError::Infrastructure(message),
    }
}

fn rbac_config_error(error: SystemError) -> RbacError {
    match error {
        SystemError::NotFound => RbacError::Infrastructure(REQUIRED_SYSTEM_CONFIG_ERROR.into()),
        SystemError::Forbidden(_) => RbacError::Forbidden,
        SystemError::Conflict(message) => RbacError::Conflict(message),
        SystemError::InvalidInput(message) => RbacError::InvalidInput(message),
        SystemError::Infrastructure(message) => RbacError::Infrastructure(message),
    }
}

fn user_error_to_rbac(error: AppError) -> RbacError {
    match error {
        AppError::InvalidInput(message) => RbacError::InvalidInput(message),
        AppError::Unauthorized => RbacError::Unauthorized,
        AppError::Forbidden(_) => RbacError::Forbidden,
        AppError::Conflict(message) => RbacError::Conflict(message),
        AppError::NotFound => RbacError::NotFound,
        AppError::Infrastructure(message) => RbacError::Infrastructure(message),
    }
}

fn user_error_to_system(error: AppError) -> SystemError {
    match error {
        AppError::InvalidInput(message) => SystemError::InvalidInput(message),
        AppError::Unauthorized => SystemError::Forbidden(kernel::error::LocalizedError::new("errors.common.forbidden")),
        AppError::Forbidden(message) => SystemError::Forbidden(message),
        AppError::Conflict(message) => SystemError::Conflict(message),
        AppError::NotFound => SystemError::NotFound,
        AppError::Infrastructure(message) => SystemError::Infrastructure(message),
    }
}

fn captcha_account_error(error: CaptchaError) -> AppError {
    match error {
        CaptchaError::InvalidInput(message) => AppError::InvalidInput(message),
        CaptchaError::Infrastructure(message) => AppError::Infrastructure(message),
    }
}
