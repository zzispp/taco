mod jobs;
mod logs;

use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;

use crate::{
    application::{AuditedSchedulerCommandStore, ManualExecutionRequest, PersistJobReplacement, PersistNewJob, SchedulerResult, UpdateJobStatusCommand},
    domain::Job,
    infra::persistence::StorageSchedulerRepository,
};

#[async_trait]
impl AuditedSchedulerCommandStore for StorageSchedulerRepository {
    async fn insert_job_with_audit(&self, command: PersistNewJob, audit: AuditOutboxRecord) -> SchedulerResult<Job> {
        jobs::insert_job(self, command, &audit).await
    }

    async fn replace_job_with_audit(&self, command: PersistJobReplacement, audit: AuditOutboxRecord) -> SchedulerResult<Job> {
        jobs::replace_job(self, command, &audit).await
    }

    async fn update_job_status_with_audit(&self, command: UpdateJobStatusCommand, audit: AuditOutboxRecord) -> SchedulerResult<Job> {
        jobs::update_job_status(self, command, &audit).await
    }

    async fn enqueue_manual_with_audit(&self, request: ManualExecutionRequest, audit: AuditOutboxRecord) -> SchedulerResult<String> {
        jobs::enqueue_manual(self, request, &audit).await
    }

    async fn delete_jobs_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SchedulerResult<()> {
        jobs::delete_jobs(self, ids, &audit).await
    }

    async fn delete_execution_logs_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SchedulerResult<()> {
        logs::delete_execution_logs(self, ids, &audit).await
    }

    async fn clear_execution_logs_with_audit(&self, audit: AuditOutboxRecord) -> SchedulerResult<()> {
        logs::clear_execution_logs(self, &audit).await
    }
}
