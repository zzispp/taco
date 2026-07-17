use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;

use crate::domain::SystemLogFilter;

use super::ObservabilityResult;

#[derive(Clone, Debug)]
pub struct ManualSystemLogCleanupRequest {
    pub filter: SystemLogFilter,
    pub requested_by: String,
    pub audit: AuditOutboxRecord,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemLogCleanupExecutionState {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
    Interrupted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemLogCleanupExecution {
    pub execution_id: String,
    pub state: SystemLogCleanupExecutionState,
    pub deleted: Option<u64>,
    pub batches: Option<u64>,
}

#[async_trait]
pub trait SystemLogCleanupExecutionPort: Send + Sync + 'static {
    async fn enqueue_manual_cleanup(&self, request: ManualSystemLogCleanupRequest) -> ObservabilityResult<String>;
    async fn cleanup_execution(&self, execution_id: &str) -> ObservabilityResult<SystemLogCleanupExecution>;
}
