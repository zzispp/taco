use std::sync::Arc;

use audit_contract::{AuditOutboxRecorder, SecurityAuditRecorder};
use client_info::IpLocationResolver;
use kernel::runtime_config::{ExportBatchConfig, ExportConfigProvider};
use rbac::application::RbacUseCase;

const AVATAR_STORAGE_DISABLED_ERROR: &str = "infra.user.avatar_storage_disabled";
const AVATAR_CONFIG_DISABLED_ERROR: &str = "infra.user.avatar_config_disabled";
const EXPORT_CONFIG_DISABLED_ERROR: &str = "infra.user.export_config_disabled";

use crate::{
    api::TokenService,
    application::{
        AccountVerifier, AppError, AppResult, AvatarConfigProvider, AvatarOwner, AvatarStorage, NormalizedAvatar, SystemConfigProvider, UserUseCase,
    },
    domain::AvatarFileId,
};

#[derive(Clone)]
pub struct ApiState {
    pub users: Arc<dyn UserUseCase>,
    pub tokens: TokenService,
    pub rbac: Arc<dyn RbacUseCase>,
    pub config: Arc<dyn SystemConfigProvider>,
    pub account_verifier: Arc<dyn AccountVerifier>,
    pub ip_location_resolver: Arc<dyn IpLocationResolver>,
    pub operation_audit: Arc<dyn AuditOutboxRecorder>,
    pub security_audit: Arc<dyn SecurityAuditRecorder>,
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
    pub ip_location_resolver: Arc<dyn IpLocationResolver>,
    pub operation_audit: Arc<dyn AuditOutboxRecorder>,
    pub security_audit: Arc<dyn SecurityAuditRecorder>,
}

impl ApiState {
    pub fn new(parts: ApiStateParts) -> Self {
        Self {
            users: parts.users,
            tokens: parts.tokens,
            rbac: parts.rbac,
            config: parts.config,
            account_verifier: parts.account_verifier,
            ip_location_resolver: parts.ip_location_resolver,
            operation_audit: parts.operation_audit,
            security_audit: parts.security_audit,
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
    async fn store_avatar(&self, _owner: AvatarOwner, _avatar: NormalizedAvatar) -> AppResult<AvatarFileId> {
        Err(AppError::Infrastructure(AVATAR_STORAGE_DISABLED_ERROR.into()))
    }

    async fn trash_avatar(&self, _owner: AvatarOwner, _file_id: AvatarFileId) -> AppResult<()> {
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
