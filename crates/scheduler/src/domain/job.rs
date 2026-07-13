use chrono::{DateTime, Utc};
use serde_json::Value;

use super::{ConcurrentPolicy, ExecutionOutcome, ExecutionState, LocalizedMessage, MisfirePolicy, RuntimeErrorCode, TriggerType};

#[derive(Clone, Debug, PartialEq)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub group: String,
    pub task_key: String,
    pub task_params: Value,
    pub params_schema_version: i16,
    pub repeatable: bool,
    pub invoke_target: String,
    pub cron_expression: String,
    pub misfire_policy: MisfirePolicy,
    pub concurrent: ConcurrentPolicy,
    pub status: super::JobStatus,
    pub schedule_revision: i64,
    pub next_run_at: Option<DateTime<Utc>>,
    pub runtime_error: Option<RuntimeErrorState>,
    pub create_by: String,
    pub create_time: DateTime<Utc>,
    pub update_by: String,
    pub update_time: Option<DateTime<Utc>>,
    pub remark: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeErrorState {
    pub code: RuntimeErrorCode,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Execution {
    pub id: String,
    pub snapshot: ExecutionSnapshot,
    pub trigger: TriggerType,
    pub scheduled_at: DateTime<Utc>,
    pub state: ExecutionState,
    pub outcome: Option<ExecutionOutcome>,
    pub executor_epoch: Option<String>,
    pub requested_by: Option<String>,
    pub message: Option<LocalizedMessage>,
    pub error: Option<LocalizedMessage>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub create_time: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutionSnapshot {
    pub job_id: String,
    pub job_revision: i64,
    pub job_name: String,
    pub job_group: String,
    pub task_key: String,
    pub task_params: Value,
    pub params_schema_version: i16,
    pub repeatable: bool,
    pub invoke_target: String,
    pub concurrent: ConcurrentPolicy,
}

impl From<&Job> for ExecutionSnapshot {
    fn from(job: &Job) -> Self {
        Self {
            job_id: job.id.clone(),
            job_revision: job.schedule_revision,
            job_name: job.name.clone(),
            job_group: job.group.clone(),
            task_key: job.task_key.clone(),
            task_params: job.task_params.clone(),
            params_schema_version: job.params_schema_version,
            repeatable: job.repeatable,
            invoke_target: job.invoke_target.clone(),
            concurrent: job.concurrent,
        }
    }
}
