use std::sync::Arc;

use kernel::runtime_config::ExportConfigProvider;

use crate::application::{AuditError, AuditUseCase, LoginUnlocker};

#[derive(Clone)]
pub struct AuditApiState {
    pub audit: Arc<dyn AuditUseCase>,
    pub export_config: Arc<dyn ExportConfigProvider<Error = AuditError>>,
    pub unlocker: Arc<dyn LoginUnlocker>,
}

pub struct AuditApiStateParts {
    pub audit: Arc<dyn AuditUseCase>,
    pub export_config: Arc<dyn ExportConfigProvider<Error = AuditError>>,
    pub unlocker: Arc<dyn LoginUnlocker>,
}

impl AuditApiState {
    pub fn new(parts: AuditApiStateParts) -> Self {
        Self {
            audit: parts.audit,
            export_config: parts.export_config,
            unlocker: parts.unlocker,
        }
    }
}
