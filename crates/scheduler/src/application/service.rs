use std::sync::Arc;

use async_trait::async_trait;
use kernel::pagination::{CursorPage, CursorPageRequest};

use crate::domain::{Execution, ExecutionSnapshot, Job, JobListFilter, JobLogListFilter, JobStatus};

use super::{
    AuditedSchedulerCommandStore, Clock, ExecutionLogDetail, ExecutionLogSummary, ImportJobCommand, ImportableTask, JobView, ManualExecutionRequest,
    ReplaceJobCommand, SchedulerCommandStore, SchedulerCursorQuery, SchedulerCursorSlice, SchedulerError, SchedulerQueryStore, SchedulerResult,
    UpdateJobStatusCommand,
    cron::next_times_after,
    job_cursor_page, job_cursor_query, log_cursor_page, log_cursor_query,
    service_support::{
        lifecycle_capabilities, new_job, registry_status, replacement, require_deletion_allowed, require_editable, require_runnable,
        require_status_change_allowed, validate_import, validate_replace,
    },
    task::{ScheduledTaskDefinition, TaskCatalog},
    tasks::{ManualSystemLogCleanupExecution, manual_system_log_cleanup_execution},
    validation::{require_text, validate_ids},
};

mod audited;
mod export;

#[async_trait]
pub trait SchedulerExportSession: Send {
    async fn page_jobs(&mut self, filter: JobListFilter, page: SchedulerCursorQuery) -> SchedulerResult<SchedulerCursorSlice<JobView>>;
    async fn page_job_logs(&mut self, filter: JobLogListFilter, page: SchedulerCursorQuery) -> SchedulerResult<SchedulerCursorSlice<ExecutionLogSummary>>;
    async fn finish(self: Box<Self>) -> SchedulerResult<()>;
}

#[async_trait]
pub trait SchedulerUseCase: Send + Sync + 'static {
    async fn importable_tasks(&self) -> SchedulerResult<Vec<ImportableTask>>;
    async fn page_jobs(&self, filter: JobListFilter, page: CursorPageRequest) -> SchedulerResult<CursorPage<JobView>>;
    async fn begin_export(&self) -> SchedulerResult<Box<dyn SchedulerExportSession>>;
    async fn get_job(&self, id: &str) -> SchedulerResult<JobView>;
    async fn import_job(&self, command: ImportJobCommand) -> SchedulerResult<JobView>;
    async fn replace_job(&self, command: ReplaceJobCommand) -> SchedulerResult<JobView>;
    async fn update_job_status(&self, command: UpdateJobStatusCommand) -> SchedulerResult<JobView>;
    async fn run_job(&self, id: &str, requested_by: &str) -> SchedulerResult<String>;
    async fn delete_job(&self, id: &str) -> SchedulerResult<()>;
    async fn delete_jobs(&self, ids: Vec<String>) -> SchedulerResult<()>;
    async fn cron_next_times(&self, expression: &str, count: Option<u8>) -> SchedulerResult<Vec<chrono::DateTime<chrono::Utc>>>;
    async fn page_job_logs(&self, filter: JobLogListFilter, page: CursorPageRequest) -> SchedulerResult<CursorPage<ExecutionLogSummary>>;
    async fn get_execution(&self, id: &str) -> SchedulerResult<Execution>;
    async fn get_manual_system_log_cleanup_execution(&self, id: &str) -> SchedulerResult<ManualSystemLogCleanupExecution>;
    async fn get_job_log(&self, id: &str) -> SchedulerResult<ExecutionLogSummary>;
    async fn get_job_log_detail(&self, id: &str) -> SchedulerResult<ExecutionLogDetail>;
    async fn delete_job_log(&self, id: &str) -> SchedulerResult<()>;
    async fn delete_job_logs(&self, ids: Vec<String>) -> SchedulerResult<()>;
    async fn clear_job_logs(&self) -> SchedulerResult<()>;
}

pub struct SchedulerService {
    query: Arc<dyn SchedulerQueryStore>,
    commands: Arc<dyn SchedulerCommandStore>,
    audited_commands: Arc<dyn AuditedSchedulerCommandStore>,
    catalog: Arc<dyn TaskCatalog>,
    clock: Arc<dyn Clock>,
}

pub struct SchedulerServiceParts {
    pub query: Arc<dyn SchedulerQueryStore>,
    pub commands: Arc<dyn SchedulerCommandStore>,
    pub audited_commands: Arc<dyn AuditedSchedulerCommandStore>,
    pub catalog: Arc<dyn TaskCatalog>,
    pub clock: Arc<dyn Clock>,
}

impl SchedulerService {
    pub fn new(parts: SchedulerServiceParts) -> Self {
        Self {
            query: parts.query,
            commands: parts.commands,
            audited_commands: parts.audited_commands,
            catalog: parts.catalog,
            clock: parts.clock,
        }
    }

    fn decorate(&self, job: Job) -> JobView {
        let registry_status = registry_status(self.catalog.as_ref(), &job);
        let param_form = self.catalog.get(&job.task_key).map(|definition| (definition.params.form)());
        JobView {
            capabilities: lifecycle_capabilities(self.catalog.as_ref(), &job),
            job,
            registry_status,
            param_form,
        }
    }

    fn definition(&self, task_key: &str) -> SchedulerResult<ScheduledTaskDefinition> {
        self.catalog
            .get(task_key)
            .ok_or_else(|| SchedulerError::InvalidInput(super::error::localized("errors.scheduler.task_missing")))
    }
}

#[async_trait]
impl SchedulerUseCase for SchedulerService {
    async fn importable_tasks(&self) -> SchedulerResult<Vec<ImportableTask>> {
        let mut tasks = Vec::new();
        for definition in self.catalog.all() {
            if !definition.repeatable && self.query.task_key_exists(definition.task_key).await? {
                continue;
            }
            tasks.push(importable_task(definition));
        }
        Ok(tasks)
    }

    async fn page_jobs(&self, filter: JobListFilter, page: CursorPageRequest) -> SchedulerResult<CursorPage<JobView>> {
        let query = job_cursor_query(&filter, &page)?;
        let slice = self.query.page_jobs(filter.clone(), query.clone()).await?;
        Ok(job_cursor_page(&filter, &query, slice)?.map(|job| self.decorate(job)))
    }

    async fn get_job(&self, id: &str) -> SchedulerResult<JobView> {
        Ok(self.decorate(self.query.find_job(id).await?))
    }

    async fn begin_export(&self) -> SchedulerResult<Box<dyn SchedulerExportSession>> {
        Ok(Box::new(export::ServiceExportSession::new(
            self.query.begin_export().await?,
            Arc::clone(&self.catalog),
        )))
    }

    async fn import_job(&self, command: ImportJobCommand) -> SchedulerResult<JobView> {
        validate_import(&command)?;
        let definition = self.definition(&command.task_key)?;
        if !definition.repeatable && self.query.task_key_exists(definition.task_key).await? {
            return Err(SchedulerError::conflict(
                "scheduler_task_already_imported",
                "errors.scheduler.task_already_imported",
            ));
        }
        (definition.params.validate_persisted)(&command.task_params)?;
        Ok(self.decorate(self.commands.insert_job(new_job(command, definition)?).await?))
    }

    async fn replace_job(&self, command: ReplaceJobCommand) -> SchedulerResult<JobView> {
        validate_replace(&command)?;
        let current = self.query.find_job(&command.id).await?;
        let definition = require_editable(self.catalog.as_ref(), &current)?;
        (definition.params.validate_persisted)(&command.task_params)?;
        Ok(self.decorate(self.commands.replace_job(replacement(command, definition)?).await?))
    }

    async fn update_job_status(&self, command: UpdateJobStatusCommand) -> SchedulerResult<JobView> {
        let current = self.query.find_job(&command.id).await?;
        require_status_change_allowed(self.catalog.as_ref(), &current, command.status)?;
        if command.status == JobStatus::Normal {
            require_runnable(self.catalog.as_ref(), &current)?;
        }
        Ok(self.decorate(self.commands.update_job_status(command).await?))
    }

    async fn run_job(&self, id: &str, requested_by: &str) -> SchedulerResult<String> {
        let job = self.query.find_job(id).await?;
        require_runnable(self.catalog.as_ref(), &job)?;
        let request = ManualExecutionRequest {
            expected_revision: job.schedule_revision,
            snapshot: ExecutionSnapshot::from(&job),
            scheduled_at: self.clock.now().await?,
            requested_by: requested_by.to_owned(),
        };
        self.commands.enqueue_manual(request).await
    }

    async fn delete_job(&self, id: &str) -> SchedulerResult<()> {
        require_deletion_allowed(self.catalog.as_ref(), &self.query.find_job(id).await?)?;
        self.commands.delete_job(id).await
    }

    async fn delete_jobs(&self, ids: Vec<String>) -> SchedulerResult<()> {
        let ids = validate_ids(ids)?;
        for id in &ids {
            require_deletion_allowed(self.catalog.as_ref(), &self.query.find_job(id).await?)?;
        }
        self.commands.delete_jobs(ids).await
    }

    async fn cron_next_times(&self, expression: &str, count: Option<u8>) -> SchedulerResult<Vec<chrono::DateTime<chrono::Utc>>> {
        require_text(expression, "errors.scheduler.cron_required")?;
        next_times_after(expression, count, self.clock.now().await?)
    }

    async fn page_job_logs(&self, filter: JobLogListFilter, page: CursorPageRequest) -> SchedulerResult<CursorPage<ExecutionLogSummary>> {
        let query = log_cursor_query(&filter, &page)?;
        let slice = self.query.page_execution_logs(filter.clone(), query.clone()).await?;
        log_cursor_page(&filter, &query, slice)
    }

    async fn get_execution(&self, id: &str) -> SchedulerResult<Execution> {
        self.query.find_execution(id).await
    }

    async fn get_manual_system_log_cleanup_execution(&self, id: &str) -> SchedulerResult<ManualSystemLogCleanupExecution> {
        let execution = self.query.find_execution(id).await?;
        let detail = if execution.state == crate::domain::ExecutionState::Terminal {
            self.query.find_execution_log_detail(id).await?.detail
        } else {
            None
        };
        manual_system_log_cleanup_execution(&execution, detail.as_ref())
    }

    async fn get_job_log(&self, id: &str) -> SchedulerResult<ExecutionLogSummary> {
        self.query.find_execution_log(id).await
    }

    async fn get_job_log_detail(&self, id: &str) -> SchedulerResult<ExecutionLogDetail> {
        self.query.find_execution_log_detail(id).await
    }

    async fn delete_job_log(&self, id: &str) -> SchedulerResult<()> {
        self.commands.delete_execution_log(id).await
    }

    async fn delete_job_logs(&self, ids: Vec<String>) -> SchedulerResult<()> {
        self.commands.delete_execution_logs(validate_ids(ids)?).await
    }

    async fn clear_job_logs(&self) -> SchedulerResult<()> {
        self.commands.clear_execution_logs().await
    }
}

fn importable_task(definition: ScheduledTaskDefinition) -> ImportableTask {
    ImportableTask {
        task_key: definition.task_key,
        name_key: definition.name_key,
        group: definition.group,
        group_key: definition.group_key,
        description_key: definition.description_key,
        repeatable: definition.repeatable,
        default_params: (definition.params.default_params)(),
        param_form: (definition.params.form)(),
    }
}
