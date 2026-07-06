use std::sync::Arc;

use kernel::error::LocalizedError;
use kernel::runtime_config::{ExportBatchConfig, ExportConfigProvider};

use crate::application::{RbacAdminUseCase, RbacError, RbacUseCase};

#[derive(Clone)]
pub struct RbacApiState {
    pub rbac: Arc<dyn RbacUseCase>,
    pub rbac_admin: Arc<dyn RbacAdminUseCase>,
    pub export_config: Arc<dyn ExportConfigProvider<Error = RbacError>>,
}

impl RbacApiState {
    pub fn new(rbac: Arc<dyn RbacUseCase>, rbac_admin: Arc<dyn RbacAdminUseCase>) -> Self {
        Self {
            rbac,
            rbac_admin,
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
