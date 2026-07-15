use std::sync::Arc;

use kernel::error::LocalizedError;
use kernel::runtime_config::{ExportBatchConfig, ExportConfigProvider};

use crate::application::{RbacAdminUseCase, RbacAuditedAdminUseCase, RbacCacheRefreshUseCase, RbacError, RbacUseCase};

#[derive(Clone)]
pub struct RbacApiState {
    pub rbac: Arc<dyn RbacUseCase>,
    pub rbac_admin: Arc<dyn RbacAdminUseCase>,
    pub rbac_audited_admin: Arc<dyn RbacAuditedAdminUseCase>,
    pub rbac_cache_refresher: Arc<dyn RbacCacheRefreshUseCase>,
    pub export_config: Arc<dyn ExportConfigProvider<Error = RbacError>>,
}

pub struct RbacApiStateParts {
    pub rbac: Arc<dyn RbacUseCase>,
    pub rbac_admin: Arc<dyn RbacAdminUseCase>,
    pub rbac_audited_admin: Arc<dyn RbacAuditedAdminUseCase>,
    pub rbac_cache_refresher: Arc<dyn RbacCacheRefreshUseCase>,
}

impl RbacApiState {
    pub fn new(parts: RbacApiStateParts) -> Self {
        Self {
            rbac: parts.rbac,
            rbac_admin: parts.rbac_admin,
            rbac_audited_admin: parts.rbac_audited_admin,
            rbac_cache_refresher: parts.rbac_cache_refresher,
            export_config: Arc::new(DisabledExportConfigProvider),
        }
    }

    pub fn with_export_config(mut self, export_config: Arc<dyn ExportConfigProvider<Error = RbacError>>) -> Self {
        self.export_config = export_config;
        self
    }
}

struct DisabledExportConfigProvider;

#[async_trait::async_trait]
impl ExportConfigProvider for DisabledExportConfigProvider {
    type Error = RbacError;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error> {
        Err(RbacError::InvalidInput(LocalizedError::new("errors.rbac.export_config_unconfigured")))
    }
}
