use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;

use super::{
    ImportJobCommand, JobView, ManualExecutionRequest, PersistJobReplacement, PersistNewJob, ReplaceJobCommand, SchedulerCommandStore, SchedulerResult,
    UpdateJobStatusCommand,
};
use crate::application::task::SystemLogCleanupFilter;
use crate::domain::Job;

/// Command persistence required by management routes that must commit their
/// business change and operation-audit event together.
#[async_trait]
pub trait AuditedSchedulerCommandStore: SchedulerCommandStore {
    async fn insert_job_with_audit(&self, command: PersistNewJob, audit: AuditOutboxRecord) -> SchedulerResult<Job>;
    async fn replace_job_with_audit(&self, command: PersistJobReplacement, audit: AuditOutboxRecord) -> SchedulerResult<Job>;
    async fn update_job_status_with_audit(&self, command: UpdateJobStatusCommand, audit: AuditOutboxRecord) -> SchedulerResult<Job>;
    async fn enqueue_manual_with_audit(&self, request: ManualExecutionRequest, audit: AuditOutboxRecord) -> SchedulerResult<String>;
    async fn delete_jobs_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SchedulerResult<()>;
    async fn delete_execution_logs_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SchedulerResult<()>;
    async fn clear_execution_logs_with_audit(&self, audit: AuditOutboxRecord) -> SchedulerResult<()>;
}

/// Audited management mutations. Keeping this separate from read and runtime
/// scheduler operations prevents an API handler from silently using an
/// unaudited write port.
#[async_trait]
pub trait SchedulerAuditedUseCase: Send + Sync + 'static {
    async fn import_job_with_audit(&self, command: ImportJobCommand, audit: AuditOutboxRecord) -> SchedulerResult<JobView>;
    async fn replace_job_with_audit(&self, command: ReplaceJobCommand, audit: AuditOutboxRecord) -> SchedulerResult<JobView>;
    async fn update_job_status_with_audit(&self, command: UpdateJobStatusCommand, audit: AuditOutboxRecord) -> SchedulerResult<JobView>;
    async fn run_job_with_audit(&self, id: &str, requested_by: &str, audit: AuditOutboxRecord) -> SchedulerResult<String>;
    async fn run_system_log_cleanup_with_audit(
        &self,
        id: &str,
        filter: SystemLogCleanupFilter,
        requested_by: &str,
        audit: AuditOutboxRecord,
    ) -> SchedulerResult<String>;
    async fn delete_job_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SchedulerResult<()>;
    async fn delete_jobs_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SchedulerResult<()>;
    async fn delete_job_log_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SchedulerResult<()>;
    async fn delete_job_logs_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SchedulerResult<()>;
    async fn clear_job_logs_with_audit(&self, audit: AuditOutboxRecord) -> SchedulerResult<()>;
}
