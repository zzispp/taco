use std::sync::Arc;

use async_trait::async_trait;
use captcha::application::{CaptchaError, CaptchaSettingsReader, CaptchaUseCase};
use constants::system_config::{AVATAR_CONFIG_KEY, CAPTCHA_CONFIG_KEY, EXPORT_BATCH_CONFIG_KEY, IP_LOCATION_CONFIG_KEY, PASSWORD_POLICY_KEY, TOKEN_CONFIG_KEY};
use kernel::runtime_config::{ExportBatchConfig, ExportConfigProvider, RuntimeConfigError, parse_export_batch_config};
use rbac::application::RbacError;
use scheduler::application::SchedulerError;
use serde_json::Value;
use system::application::{SystemError, SystemUseCase};

const REQUIRED_CAPTCHA_SYSTEM_CONFIG_ERROR: &str = "infra.system_config.captcha_required_missing";
const REQUIRED_SYSTEM_CONFIG_ERROR: &str = "infra.system_config.required_missing";

use user::{
    api::{TokenSettingsReader, TokenTtlConfig, parse_token_ttl_config},
    application::{
        AccountVerifier, AppError, AppResult, AvatarConfig, AvatarConfigProvider, IpLocationConfig, IpLocationSettingsReader, PasswordPolicy,
        PasswordPolicyProvider, SystemConfigProvider, parse_avatar_config, parse_ip_location_config, parse_password_policy,
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
        parse_export_batch_config(&self.user_config(EXPORT_BATCH_CONFIG_KEY).await?).map_err(runtime_config_to_user)
    }
}

#[async_trait]
impl TokenSettingsReader for RuntimeUserConfig {
    async fn token_ttl_config(&self) -> AppResult<TokenTtlConfig> {
        parse_token_ttl_config(&self.user_config(TOKEN_CONFIG_KEY).await?)
    }
}

#[async_trait]
impl IpLocationSettingsReader for RuntimeUserConfig {
    async fn ip_location_config(&self) -> AppResult<IpLocationConfig> {
        parse_ip_location_config(&self.user_config(IP_LOCATION_CONFIG_KEY).await?)
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
        parse_export_batch_config(&value).map_err(runtime_config_to_rbac)
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
        parse_export_batch_config(&value).map_err(runtime_config_to_system)
    }
}

#[derive(Clone)]
pub(super) struct RuntimeSchedulerConfig {
    system: Arc<dyn SystemUseCase>,
}

impl RuntimeSchedulerConfig {
    pub(super) fn new(system: Arc<dyn SystemUseCase>) -> Self {
        Self { system }
    }
}

#[async_trait]
impl ExportConfigProvider for RuntimeSchedulerConfig {
    type Error = SchedulerError;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error> {
        let value = self.system.config_by_key(EXPORT_BATCH_CONFIG_KEY).await.map_err(system_error_to_scheduler)?;
        parse_export_batch_config(&value).map_err(runtime_config_to_scheduler)
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
    hook_tracing::error_with_fields!("invalid captcha runtime config JSON", &error, key = CAPTCHA_CONFIG_KEY);
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

fn runtime_config_to_user(error: RuntimeConfigError) -> AppError {
    trace_invalid_export_config(&error);
    AppError::InvalidInput(kernel::error::LocalizedError::new("errors.user.invalid_system_config").with_param("key", EXPORT_BATCH_CONFIG_KEY))
}

fn runtime_config_to_rbac(error: RuntimeConfigError) -> RbacError {
    trace_invalid_export_config(&error);
    RbacError::InvalidInput(kernel::error::LocalizedError::new("errors.rbac.invalid_export_batch_config"))
}

fn runtime_config_to_system(error: RuntimeConfigError) -> SystemError {
    trace_invalid_export_config(&error);
    SystemError::InvalidInput(kernel::error::LocalizedError::new("errors.system.invalid_export_batch_config"))
}

fn runtime_config_to_scheduler(error: RuntimeConfigError) -> SchedulerError {
    trace_invalid_export_config(&error);
    SchedulerError::InvalidInput(kernel::error::LocalizedError::new("errors.scheduler.invalid_export_batch_config"))
}

fn trace_invalid_export_config(error: &RuntimeConfigError) {
    hook_tracing::error_with_fields!("invalid export runtime config", error, key = EXPORT_BATCH_CONFIG_KEY);
}

fn system_error_to_scheduler(error: SystemError) -> SchedulerError {
    match error {
        SystemError::NotFound => SchedulerError::Infrastructure(REQUIRED_SYSTEM_CONFIG_ERROR.into()),
        SystemError::Forbidden(message) => SchedulerError::Forbidden(message),
        SystemError::Conflict(message) => SchedulerError::Conflict {
            code: "scheduler_config_conflict",
            details: message,
        },
        SystemError::InvalidInput(message) => SchedulerError::InvalidInput(message),
        SystemError::Infrastructure(message) => SchedulerError::Infrastructure(message),
    }
}

fn captcha_account_error(error: CaptchaError) -> AppError {
    match error {
        CaptchaError::InvalidInput(message) => AppError::InvalidInput(message),
        CaptchaError::Infrastructure(message) => AppError::Infrastructure(message),
    }
}
