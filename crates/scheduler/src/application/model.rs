use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::domain::{
    ConcurrentPolicy, ExecutionDetail, ExecutionOutcome, ExecutionSnapshot, Job, JobStatus, LocalizedMessage, MisfirePolicy, RegistryStatus, RuntimeErrorCode,
    TaskParamFormSpec, TriggerType,
};

use super::task::TaskLifecycleCapabilities;

#[derive(Clone, Debug)]
pub struct JobView {
    pub job: Job,
    pub registry_status: RegistryStatus,
    pub capabilities: TaskLifecycleCapabilities,
    pub param_form: Option<TaskParamFormSpec>,
}

#[derive(Clone, Debug)]
pub struct ImportableTask {
    pub task_key: &'static str,
    pub name_key: &'static str,
    pub group: &'static str,
    pub group_key: &'static str,
    pub description_key: &'static str,
    pub repeatable: bool,
    pub default_params: Value,
    pub param_form: TaskParamFormSpec,
}

#[derive(Clone, Debug)]
pub struct ImportJobCommand {
    pub task_key: String,
    pub name: String,
    pub group: String,
    pub cron_expression: String,
    pub misfire_policy: MisfirePolicy,
    pub concurrent: ConcurrentPolicy,
    pub task_params: Value,
    pub remark: Option<String>,
    pub operator: String,
}

#[derive(Clone, Debug)]
pub struct ReplaceJobCommand {
    pub id: String,
    pub name: String,
    pub group: String,
    pub cron_expression: String,
    pub misfire_policy: MisfirePolicy,
    pub concurrent: ConcurrentPolicy,
    pub task_params: Value,
    pub remark: Option<String>,
    pub operator: String,
}

#[derive(Clone, Debug)]
pub struct PersistNewJob {
    pub input: ImportJobCommand,
    pub params_schema_version: i16,
    pub repeatable: bool,
    pub invoke_target: String,
}

#[derive(Clone, Debug)]
pub struct PersistJobReplacement {
    pub input: ReplaceJobCommand,
    pub params_schema_version: i16,
    pub invoke_target: String,
}

#[derive(Clone, Debug)]
pub struct UpdateJobStatusCommand {
    pub id: String,
    pub status: JobStatus,
    pub operator: String,
}

#[derive(Clone, Debug)]
pub struct ManualExecutionRequest {
    pub expected_revision: i64,
    pub snapshot: ExecutionSnapshot,
    pub scheduled_at: DateTime<Utc>,
    pub requested_by: String,
}

#[derive(Clone, Debug)]
pub struct ScheduleInitialization {
    pub job_id: String,
    pub expected_revision: i64,
    pub next_run_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct OccurrenceRequest {
    pub job_id: String,
    pub expected_revision: i64,
    pub expected_due_at: DateTime<Utc>,
    pub next_run_at: DateTime<Utc>,
    pub action: OccurrenceAction,
}

#[derive(Clone, Debug)]
pub enum OccurrenceAction {
    Queue(TriggerType),
    SkipMisfire,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OccurrenceResult {
    Materialized,
    Stale,
    AlreadyMaterialized,
}

#[derive(Clone, Debug)]
pub struct ClaimExecutionRequest {
    pub execution_id: String,
    pub executor_epoch: String,
    pub started_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinishExecutionRequest {
    pub execution_id: String,
    pub outcome: ExecutionOutcome,
    pub message: LocalizedMessage,
    pub error: Option<LocalizedMessage>,
    pub detail: Option<ExecutionDetail>,
    pub ended_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct InterruptExecutionRequest {
    pub execution_id: String,
    pub ended_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct RuntimeErrorUpdate {
    pub job_id: String,
    pub expected_revision: i64,
    pub code: RuntimeErrorCode,
    pub occurred_at: DateTime<Utc>,
}
