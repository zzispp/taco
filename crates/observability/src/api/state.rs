use std::sync::Arc;

use kernel::runtime_config::ExportConfigProvider;

use crate::application::{ObservabilityError, SystemLogCleanupExecutionPort, SystemLogExportUseCase, SystemLogUseCase};

#[derive(Clone)]
pub struct SystemLogApiState {
    pub logs: Arc<dyn SystemLogUseCase>,
    pub exporter: Arc<dyn SystemLogExportUseCase>,
    pub cleanup_executions: Arc<dyn SystemLogCleanupExecutionPort>,
    pub export_config: Arc<dyn ExportConfigProvider<Error = ObservabilityError>>,
}

pub struct SystemLogApiStateParts {
    pub logs: Arc<dyn SystemLogUseCase>,
    pub exporter: Arc<dyn SystemLogExportUseCase>,
    pub cleanup_executions: Arc<dyn SystemLogCleanupExecutionPort>,
    pub export_config: Arc<dyn ExportConfigProvider<Error = ObservabilityError>>,
}

impl SystemLogApiState {
    pub fn new(parts: SystemLogApiStateParts) -> Self {
        Self {
            logs: parts.logs,
            exporter: parts.exporter,
            cleanup_executions: parts.cleanup_executions,
            export_config: parts.export_config,
        }
    }
}
