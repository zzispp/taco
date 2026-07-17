use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::{Execution, Job, JobListFilter, JobLogListFilter};

use super::{
    ClaimExecutionRequest, ExecutionLogDetail, ExecutionLogSummary, FinishExecutionRequest, InterruptExecutionRequest, ManualExecutionRequest,
    OccurrenceRequest, OccurrenceResult, PersistJobReplacement, PersistNewJob, RuntimeErrorUpdate, ScheduleInitialization, SchedulerCursorQuery,
    SchedulerCursorSlice, SchedulerResult, UpdateJobStatusCommand,
};

#[async_trait]
pub trait SchedulerQueryStore: Send + Sync + 'static {
    async fn find_job(&self, id: &str) -> SchedulerResult<Job>;
    async fn page_jobs(&self, filter: JobListFilter, page: SchedulerCursorQuery) -> SchedulerResult<SchedulerCursorSlice<Job>>;
    async fn begin_export(&self) -> SchedulerResult<Box<dyn SchedulerQueryExportSession>>;
    async fn task_key_exists(&self, task_key: &str) -> SchedulerResult<bool>;
    async fn find_execution(&self, id: &str) -> SchedulerResult<Execution>;
    async fn find_execution_log(&self, id: &str) -> SchedulerResult<ExecutionLogSummary>;
    async fn find_execution_log_detail(&self, id: &str) -> SchedulerResult<ExecutionLogDetail>;
    async fn page_execution_logs(&self, filter: JobLogListFilter, page: SchedulerCursorQuery) -> SchedulerResult<SchedulerCursorSlice<ExecutionLogSummary>>;
}

#[async_trait]
pub trait SchedulerQueryExportSession: Send {
    async fn page_jobs(&mut self, filter: JobListFilter, page: SchedulerCursorQuery) -> SchedulerResult<SchedulerCursorSlice<Job>>;
    async fn page_execution_logs(&mut self, filter: JobLogListFilter, page: SchedulerCursorQuery)
    -> SchedulerResult<SchedulerCursorSlice<ExecutionLogSummary>>;
    async fn finish(self: Box<Self>) -> SchedulerResult<()>;
}

#[async_trait]
pub trait SchedulerCommandStore: Send + Sync + 'static {
    async fn insert_job(&self, command: PersistNewJob) -> SchedulerResult<Job>;
    async fn replace_job(&self, command: PersistJobReplacement) -> SchedulerResult<Job>;
    async fn update_job_status(&self, command: UpdateJobStatusCommand) -> SchedulerResult<Job>;
    async fn enqueue_manual(&self, request: ManualExecutionRequest) -> SchedulerResult<String>;
    async fn delete_job(&self, id: &str) -> SchedulerResult<()>;
    async fn delete_jobs(&self, ids: Vec<String>) -> SchedulerResult<()>;
    async fn delete_execution_log(&self, id: &str) -> SchedulerResult<()>;
    async fn delete_execution_logs(&self, ids: Vec<String>) -> SchedulerResult<()>;
    async fn clear_execution_logs(&self) -> SchedulerResult<()>;
}

#[async_trait]
pub trait SchedulerRuntimeStore: Send + Sync + 'static {
    async fn database_now(&self) -> SchedulerResult<DateTime<Utc>>;
    async fn schedulable_jobs(&self) -> SchedulerResult<Vec<Job>>;
    async fn initialize_schedule(&self, request: ScheduleInitialization) -> SchedulerResult<bool>;
    async fn materialize_occurrence(&self, request: OccurrenceRequest) -> SchedulerResult<OccurrenceResult>;
    async fn pending_executions(&self) -> SchedulerResult<Vec<Execution>>;
    async fn running_executions(&self) -> SchedulerResult<Vec<Execution>>;
    async fn claim_execution(&self, request: ClaimExecutionRequest) -> SchedulerResult<Option<Execution>>;
    async fn finish_execution(&self, request: FinishExecutionRequest) -> SchedulerResult<bool>;
    async fn interrupt_execution(&self, request: InterruptExecutionRequest) -> SchedulerResult<bool>;
    async fn set_runtime_error(&self, request: RuntimeErrorUpdate) -> SchedulerResult<()>;
    async fn clear_runtime_error(&self, job_id: &str, expected_revision: i64) -> SchedulerResult<()>;
}

#[async_trait]
pub trait Clock: Send + Sync + 'static {
    async fn now(&self) -> SchedulerResult<DateTime<Utc>>;
}

#[async_trait]
pub trait LeaderSession: Send {
    async fn is_alive(&mut self) -> SchedulerResult<bool>;
    async fn release(&mut self) -> SchedulerResult<()>;
}

#[async_trait]
pub trait LeaderLease: Send + Sync + 'static {
    async fn try_acquire(&self) -> SchedulerResult<Option<Box<dyn LeaderSession>>>;
}

#[async_trait]
pub trait ChangeListener: Send {
    async fn wait(&mut self) -> SchedulerResult<()>;
}

#[async_trait]
pub trait ChangeListenerFactory: Send + Sync + 'static {
    async fn connect(&self) -> SchedulerResult<Box<dyn ChangeListener>>;
}

#[async_trait]
pub trait ExecutionLeaseSession: Send {
    async fn try_acquire(&mut self, execution_id: &str) -> SchedulerResult<bool>;
    async fn release(&mut self, execution_id: &str) -> SchedulerResult<()>;
    async fn is_alive(&mut self) -> SchedulerResult<bool>;
    async fn release_all(&mut self) -> SchedulerResult<()>;
}

#[async_trait]
pub trait ExecutionLease: Send + Sync + 'static {
    async fn open_session(&self) -> SchedulerResult<Box<dyn ExecutionLeaseSession>>;
    async fn is_owned(&self, execution_id: &str) -> SchedulerResult<bool>;
}

pub trait SchedulerTelemetry: Send + Sync + 'static {
    fn leadership(&self, leader: bool);
    fn reconcile(&self, reason: &'static str, success: bool);
    fn runtime_error(&self, operation: &'static str);
    fn execution(&self, trigger: &'static str, outcome: &'static str);
    fn active_executions(&self, pending: usize, running: usize);
    fn schedule_lag(&self, seconds: f64);
}
