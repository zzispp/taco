use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;

use crate::{
    application::{
        ImportJobCommand, ManualExecutionRequest, ReplaceJobCommand, SchedulerAuditedUseCase, SchedulerError, SchedulerResult, UpdateJobStatusCommand,
        service_support::{
            new_job, replacement, require_deletion_allowed, require_editable, require_runnable, require_status_change_allowed, validate_import,
            validate_replace,
        },
        task::{SystemLogCleanupFilter, TaskParams},
        tasks::{SYSTEM_LOG_CLEANUP_TASK_KEY, SystemLogCleanupParams, manual_system_log_cleanup_params},
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
        (definition.params.validate_persisted)(&command.task_params)?;
        let job = self.audited_commands.insert_job_with_audit(new_job(command, definition)?, audit).await?;
        Ok(self.decorate(job))
    }

    async fn replace_job_with_audit(&self, command: ReplaceJobCommand, audit: AuditOutboxRecord) -> SchedulerResult<crate::application::JobView> {
        validate_replace(&command)?;
        let current = self.query.find_job(&command.id).await?;
        let definition = require_editable(self.catalog.as_ref(), &current)?;
        (definition.params.validate_persisted)(&command.task_params)?;
        let job = self.audited_commands.replace_job_with_audit(replacement(command, definition)?, audit).await?;
        Ok(self.decorate(job))
    }

    async fn update_job_status_with_audit(&self, command: UpdateJobStatusCommand, audit: AuditOutboxRecord) -> SchedulerResult<crate::application::JobView> {
        let current = self.query.find_job(&command.id).await?;
        require_status_change_allowed(self.catalog.as_ref(), &current, command.status)?;
        if command.status == JobStatus::Normal {
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

    async fn run_system_log_cleanup_with_audit(
        &self,
        id: &str,
        filter: SystemLogCleanupFilter,
        requested_by: &str,
        audit: AuditOutboxRecord,
    ) -> SchedulerResult<String> {
        let job = self.query.find_job(id).await?;
        require_runnable(self.catalog.as_ref(), &job)?;
        let params = manual_system_log_cleanup_params(&job.task_params, filter)?;
        let request = ManualExecutionRequest {
            expected_revision: job.schedule_revision,
            snapshot: manual_cleanup_snapshot(&job, params)?,
            scheduled_at: self.clock.now().await?,
            requested_by: requested_by.to_owned(),
        };
        self.audited_commands.enqueue_manual_with_audit(request, audit).await
    }

    async fn delete_job_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SchedulerResult<()> {
        require_deletion_allowed(self.catalog.as_ref(), &self.query.find_job(id).await?)?;
        self.audited_commands.delete_jobs_with_audit(vec![id.to_owned()], audit).await
    }

    async fn delete_jobs_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SchedulerResult<()> {
        let ids = validate_ids(ids)?;
        for id in &ids {
            require_deletion_allowed(self.catalog.as_ref(), &self.query.find_job(id).await?)?;
        }
        self.audited_commands.delete_jobs_with_audit(ids, audit).await
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

fn manual_cleanup_snapshot(job: &crate::domain::Job, params: serde_json::Value) -> SchedulerResult<ExecutionSnapshot> {
    if job.task_key != SYSTEM_LOG_CLEANUP_TASK_KEY {
        return Err(SchedulerError::InvalidInput(crate::application::localized("errors.scheduler.invalid_params")));
    }
    let invoke_target = SystemLogCleanupParams::render_invoke_target(SYSTEM_LOG_CLEANUP_TASK_KEY, &params)?;
    let mut snapshot = ExecutionSnapshot::from(job);
    snapshot.task_params = params;
    snapshot.invoke_target = invoke_target;
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use crate::{
        application::task::{SystemLogCleanupFilter, SystemLogCleanupLevel},
        domain::{ConcurrentPolicy, Job, JobStatus, MisfirePolicy},
    };

    use super::manual_cleanup_snapshot;

    #[test]
    fn manual_cleanup_snapshot_inherits_the_persisted_batch_size() {
        let params = crate::application::tasks::manual_system_log_cleanup_params(&json!({"retention_days": 7, "batch_size": 2400}), filter()).unwrap();

        let snapshot = manual_cleanup_snapshot(&job(), params).unwrap();

        assert_eq!(snapshot.task_params["batch_size"], 2400);
        assert_eq!(snapshot.task_params["filter"]["levels"], json!(["error"]));
        assert_eq!(snapshot.invoke_target, "observability.cleanupSystemLogs(manual_filter,batch_size=2400)");
    }

    fn filter() -> SystemLogCleanupFilter {
        SystemLogCleanupFilter {
            keyword: None,
            levels: vec![SystemLogCleanupLevel::Error],
            target: None,
            begin_time: OffsetDateTime::parse("2026-07-16T00:00:00Z", &Rfc3339).unwrap(),
            end_time: OffsetDateTime::parse("2026-07-17T00:00:00Z", &Rfc3339).unwrap(),
        }
    }

    fn job() -> Job {
        Job {
            id: "system-log-cleanup".into(),
            name: "cleanup".into(),
            group: "SYSTEM".into(),
            task_key: "observability.cleanupSystemLogs".into(),
            task_params: json!({"retention_days": 7, "batch_size": 2400}),
            params_schema_version: 1,
            repeatable: false,
            invoke_target: "observability.cleanupSystemLogs(retention_days=7,batch_size=2400)".into(),
            cron_expression: "0 0 19 * * *".into(),
            misfire_policy: MisfirePolicy::FireOnce,
            concurrent: ConcurrentPolicy::Disallow,
            status: JobStatus::Normal,
            schedule_revision: 1,
            next_run_at: None,
            runtime_error: None,
            create_by: "admin".into(),
            create_time: Utc::now(),
            update_by: "admin".into(),
            update_time: None,
            remark: None,
        }
    }
}
