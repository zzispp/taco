use std::sync::Arc;

use kernel::runtime_config::{ExportBatchConfig, ExportConfigProvider};
use rbac::application::RbacUseCase;

const AVATAR_STORAGE_DISABLED_ERROR: &str = "infra.user.avatar_storage_disabled";
const AVATAR_CONFIG_DISABLED_ERROR: &str = "infra.user.avatar_config_disabled";
const EXPORT_CONFIG_DISABLED_ERROR: &str = "infra.user.export_config_disabled";

use crate::{
    api::TokenService,
    application::{AccountVerifier, AppError, AppResult, AvatarConfigProvider, AvatarFile, AvatarStorage, SystemConfigProvider, UserUseCase},
};

#[derive(Clone)]
pub struct ApiState {
    pub users: Arc<dyn UserUseCase>,
    pub tokens: TokenService,
    pub rbac: Arc<dyn RbacUseCase>,
    pub config: Arc<dyn SystemConfigProvider>,
    pub account_verifier: Arc<dyn AccountVerifier>,
    pub avatar_storage: Arc<dyn AvatarStorage>,
    pub avatar_config: Arc<dyn AvatarConfigProvider>,
    pub export_config: Arc<dyn ExportConfigProvider<Error = AppError>>,
}

pub struct ApiStateParts {
    pub users: Arc<dyn UserUseCase>,
    pub tokens: TokenService,
    pub rbac: Arc<dyn RbacUseCase>,
    pub config: Arc<dyn SystemConfigProvider>,
    pub account_verifier: Arc<dyn AccountVerifier>,
}

impl ApiState {
    pub fn new(parts: ApiStateParts) -> Self {
        Self {
            users: parts.users,
            tokens: parts.tokens,
            rbac: parts.rbac,
            config: parts.config,
            account_verifier: parts.account_verifier,
            avatar_storage: Arc::new(DisabledAvatarStorage),
            avatar_config: Arc::new(DisabledAvatarConfigProvider),
            export_config: Arc::new(DisabledExportConfigProvider),
        }
    }

    pub fn with_avatar_storage(mut self, avatar_storage: Arc<dyn AvatarStorage>) -> Self {
        self.avatar_storage = avatar_storage;
        self
    }

    pub fn with_avatar_config(mut self, avatar_config: Arc<dyn AvatarConfigProvider>) -> Self {
        self.avatar_config = avatar_config;
        self
    }

    pub fn with_export_config(mut self, export_config: Arc<dyn ExportConfigProvider<Error = AppError>>) -> Self {
        self.export_config = export_config;
        self
    }
}

struct DisabledAvatarStorage;

#[async_trait::async_trait]
impl AvatarStorage for DisabledAvatarStorage {
    async fn store_avatar(&self, _file: AvatarFile, _max_bytes: usize) -> AppResult<String> {
        Err(AppError::Infrastructure(AVATAR_STORAGE_DISABLED_ERROR.into()))
    }
}

struct DisabledAvatarConfigProvider;

#[async_trait::async_trait]
impl AvatarConfigProvider for DisabledAvatarConfigProvider {
    async fn avatar_config(&self) -> AppResult<crate::application::AvatarConfig> {
        Err(AppError::Infrastructure(AVATAR_CONFIG_DISABLED_ERROR.into()))
    }
}

struct DisabledExportConfigProvider;

#[async_trait::async_trait]
impl ExportConfigProvider for DisabledExportConfigProvider {
    type Error = AppError;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error> {
        Err(AppError::Infrastructure(EXPORT_CONFIG_DISABLED_ERROR.into()))
    }
}
