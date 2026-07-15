use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;

use crate::{
    application::{
        ImportJobCommand, ManualExecutionRequest, ReplaceJobCommand, SchedulerAuditedUseCase, SchedulerResult, UpdateJobStatusCommand,
        service_support::{new_job, replacement, require_editable, require_runnable, validate_import, validate_replace},
        validation::validate_ids,
    },
    domain::{ExecutionSnapshot, JobStatus},
};

use super::SchedulerService;

#[async_trait]
impl SchedulerAuditedUseCase for SchedulerService {
    async fn import_job_with_audit(&self, command: ImportJobCommand, audit: AuditOutboxRecord) -> SchedulerResult<crate::application::JobView> {
        validate_import(&command)?;
        let definition = self.definition(&command.task_key)?;
        if !definition.repeatable && self.query.task_key_exists(definition.task_key).await? {
            return Err(crate::application::SchedulerError::conflict(
                "scheduler_task_already_imported",
                "errors.scheduler.task_already_imported",
            ));
        }
        (definition.params.validate)(&command.task_params)?;
        let job = self.audited_commands.insert_job_with_audit(new_job(command, definition)?, audit).await?;
        Ok(self.decorate(job))
    }

    async fn replace_job_with_audit(&self, command: ReplaceJobCommand, audit: AuditOutboxRecord) -> SchedulerResult<crate::application::JobView> {
        validate_replace(&command)?;
        let current = self.query.find_job(&command.id).await?;
        let definition = require_editable(self.catalog.as_ref(), &current)?;
        (definition.params.validate)(&command.task_params)?;
        let job = self.audited_commands.replace_job_with_audit(replacement(command, definition)?, audit).await?;
        Ok(self.decorate(job))
    }

    async fn update_job_status_with_audit(&self, command: UpdateJobStatusCommand, audit: AuditOutboxRecord) -> SchedulerResult<crate::application::JobView> {
        if command.status == JobStatus::Normal {
            let current = self.query.find_job(&command.id).await?;
            require_runnable(self.catalog.as_ref(), &current)?;
        }
        Ok(self.decorate(self.audited_commands.update_job_status_with_audit(command, audit).await?))
    }

    async fn run_job_with_audit(&self, id: &str, requested_by: &str, audit: AuditOutboxRecord) -> SchedulerResult<String> {
        let job = self.query.find_job(id).await?;
        require_runnable(self.catalog.as_ref(), &job)?;
        let request = ManualExecutionRequest {
            expected_revision: job.schedule_revision,
            snapshot: ExecutionSnapshot::from(&job),
            scheduled_at: self.clock.now().await?,
            requested_by: requested_by.to_owned(),
        };
        self.audited_commands.enqueue_manual_with_audit(request, audit).await
    }

    async fn delete_job_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SchedulerResult<()> {
        self.audited_commands.delete_jobs_with_audit(vec![id.to_owned()], audit).await
    }

    async fn delete_jobs_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SchedulerResult<()> {
        self.audited_commands.delete_jobs_with_audit(validate_ids(ids)?, audit).await
    }

    async fn delete_job_log_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SchedulerResult<()> {
        self.audited_commands.delete_execution_logs_with_audit(vec![id.to_owned()], audit).await
    }

    async fn delete_job_logs_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SchedulerResult<()> {
        self.audited_commands.delete_execution_logs_with_audit(validate_ids(ids)?, audit).await
    }

    async fn clear_job_logs_with_audit(&self, audit: AuditOutboxRecord) -> SchedulerResult<()> {
        self.audited_commands.clear_execution_logs_with_audit(audit).await
    }
}
