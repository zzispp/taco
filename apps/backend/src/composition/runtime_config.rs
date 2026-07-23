use std::sync::Arc;

use async_trait::async_trait;
use captcha::application::{CaptchaError, CaptchaSettingsReader, CaptchaUseCase, parse_captcha_config_json};
use client_info::{ClientInfoError, ClientInfoResult, IpLocationConfig, IpLocationSettingsReader, parse_ip_location_config};
use constants::system_config::{
    AVATAR_CONFIG_KEY, CAPTCHA_CONFIG_KEY, EXPORT_BATCH_CONFIG_KEY, FILE_MANAGEMENT_CONFIG_KEY, IP_LOCATION_CONFIG_KEY, LOGIN_LOCK_CONFIG_KEY,
    PASSWORD_POLICY_KEY, TOKEN_CONFIG_KEY,
};
use file::FileError;
use file::application::{FileManagementConfig, FileManagementConfigProvider, parse_file_management_config};
use kernel::runtime_config::{ExportBatchConfig, ExportConfigProvider};
use rbac::application::{RbacError, parse_export_batch_config as parse_rbac_export_batch_config};
use scheduler::application::{SchedulerError, parse_export_batch_config as parse_scheduler_export_batch_config};
use serde_json::Value;
use system::application::{SystemError, SystemUseCase, parse_export_batch_config as parse_system_export_batch_config};

const REQUIRED_CAPTCHA_SYSTEM_CONFIG_ERROR: &str = "infra.system_config.captcha_required_missing";
const REQUIRED_SYSTEM_CONFIG_ERROR: &str = "infra.system_config.required_missing";
const UNEXPECTED_SYSTEM_CONFIG_CURSOR_ERROR: &str = "infra.system_config.unexpected_cursor";

use user::{
    api::{TokenSettingsReader, TokenTtlConfig, parse_token_ttl_config},
    application::{
        AccountVerifier, AppError, AppResult, AvatarConfig, AvatarConfigProvider, LoginLockConfig, LoginLockConfigProvider, PasswordPolicy,
        PasswordPolicyProvider, SystemConfigProvider, parse_avatar_config, parse_export_batch_config as parse_user_export_batch_config,
        parse_login_lock_config, parse_password_policy,
    },
};

pub(super) struct CaptchaSystemConfig {
    system: Arc<dyn SystemUseCase>,
}

#[derive(Clone)]
pub(super) struct RuntimeFileConfig {
    system: Arc<dyn SystemUseCase>,
}

impl RuntimeFileConfig {
    pub(super) fn new(system: Arc<dyn SystemUseCase>) -> Self {
        Self { system }
    }
}

#[async_trait]
impl FileManagementConfigProvider for RuntimeFileConfig {
    async fn file_management_config(&self) -> Result<FileManagementConfig, FileError> {
        let value = self.system.config_by_key(FILE_MANAGEMENT_CONFIG_KEY).await.map_err(file_config_error)?;
        parse_file_management_config(&value)
    }
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
        parse_captcha_config_json(&value)
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
        parse_user_export_batch_config(&self.user_config(EXPORT_BATCH_CONFIG_KEY).await?)
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
    async fn ip_location_config(&self) -> ClientInfoResult<IpLocationConfig> {
        let value = self.system.config_by_key(IP_LOCATION_CONFIG_KEY).await.map_err(client_info_config_error)?;
        parse_ip_location_config(&value)
    }
}

#[async_trait]
impl LoginLockConfigProvider for RuntimeUserConfig {
    async fn login_lock_config(&self) -> AppResult<LoginLockConfig> {
        parse_login_lock_config(&self.user_config(LOGIN_LOCK_CONFIG_KEY).await?)
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
        parse_rbac_export_batch_config(&value)
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
        parse_system_export_batch_config(&value)
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
        parse_scheduler_export_batch_config(&value)
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

fn captcha_config_error(error: SystemError) -> CaptchaError {
    match error {
        SystemError::NotFound => CaptchaError::Infrastructure(REQUIRED_CAPTCHA_SYSTEM_CONFIG_ERROR.into()),
        SystemError::InvalidCursor => CaptchaError::Infrastructure(UNEXPECTED_SYSTEM_CONFIG_CURSOR_ERROR.into()),
        SystemError::Forbidden(message) | SystemError::Conflict(message) | SystemError::InvalidInput(message) => CaptchaError::InvalidInput(message),
        SystemError::Infrastructure(message) => CaptchaError::Infrastructure(message),
    }
}

fn file_config_error(error: SystemError) -> FileError {
    match error {
        SystemError::NotFound => FileError::Infrastructure(REQUIRED_SYSTEM_CONFIG_ERROR.into()),
        SystemError::InvalidCursor => FileError::Infrastructure(UNEXPECTED_SYSTEM_CONFIG_CURSOR_ERROR.into()),
        SystemError::InvalidInput(message) | SystemError::Conflict(message) | SystemError::Forbidden(message) => FileError::Infrastructure(message.to_string()),
        SystemError::Infrastructure(message) => FileError::Infrastructure(message),
    }
}

fn user_config_error(error: SystemError) -> AppError {
    match error {
        SystemError::NotFound => AppError::Infrastructure(REQUIRED_SYSTEM_CONFIG_ERROR.into()),
        SystemError::InvalidCursor => AppError::Infrastructure(UNEXPECTED_SYSTEM_CONFIG_CURSOR_ERROR.into()),
        SystemError::Forbidden(message) => AppError::Forbidden(message),
        SystemError::Conflict(message) => AppError::Conflict(message),
        SystemError::InvalidInput(message) => AppError::InvalidInput(message),
        SystemError::Infrastructure(message) => AppError::Infrastructure(message),
    }
}

fn client_info_config_error(error: SystemError) -> ClientInfoError {
    match error {
        SystemError::NotFound => ClientInfoError::Provider(REQUIRED_SYSTEM_CONFIG_ERROR.into()),
        other => ClientInfoError::Provider(other.to_string()),
    }
}

fn rbac_config_error(error: SystemError) -> RbacError {
    match error {
        SystemError::NotFound => RbacError::Infrastructure(REQUIRED_SYSTEM_CONFIG_ERROR.into()),
        SystemError::InvalidCursor => RbacError::Infrastructure(UNEXPECTED_SYSTEM_CONFIG_CURSOR_ERROR.into()),
        SystemError::Forbidden(_) => RbacError::Forbidden,
        SystemError::Conflict(message) => RbacError::Conflict(message),
        SystemError::InvalidInput(message) => RbacError::InvalidInput(message),
        SystemError::Infrastructure(message) => RbacError::Infrastructure(message),
    }
}

fn system_error_to_scheduler(error: SystemError) -> SchedulerError {
    match error {
        SystemError::NotFound => SchedulerError::Infrastructure(REQUIRED_SYSTEM_CONFIG_ERROR.into()),
        SystemError::InvalidCursor => SchedulerError::Infrastructure(UNEXPECTED_SYSTEM_CONFIG_CURSOR_ERROR.into()),
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
