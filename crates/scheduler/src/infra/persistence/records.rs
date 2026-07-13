use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;

#[derive(Clone, Debug, FromRow)]
pub struct JobRecord {
    pub job_id: String,
    pub job_name: String,
    pub job_group: String,
    pub task_key: String,
    pub task_params: Value,
    pub params_schema_version: i16,
    pub repeatable: bool,
    pub invoke_target: String,
    pub cron_expression: String,
    pub misfire_policy: String,
    pub concurrent: String,
    pub status: String,
    pub schedule_revision: i64,
    pub next_run_at: Option<DateTime<Utc>>,
    pub runtime_error_code: Option<String>,
    pub runtime_error_time: Option<DateTime<Utc>>,
    pub create_by: String,
    pub create_time: DateTime<Utc>,
    pub update_by: String,
    pub update_time: Option<DateTime<Utc>>,
    pub remark: Option<String>,
}

#[derive(Clone, Debug, FromRow)]
pub struct ExecutionRecord {
    pub execution_id: String,
    pub job_id: String,
    pub job_revision: i64,
    pub job_name: String,
    pub job_group: String,
    pub task_key: String,
    pub task_params: Value,
    pub params_schema_version: i16,
    pub repeatable: bool,
    pub invoke_target: String,
    pub concurrent: String,
    pub trigger_type: String,
    pub scheduled_at: DateTime<Utc>,
    pub state: String,
    pub outcome: Option<String>,
    pub executor_epoch: Option<String>,
    pub requested_by: Option<String>,
    pub message_key: Option<String>,
    pub message_params: Value,
    pub error_key: Option<String>,
    pub error_params: Value,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub create_time: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow)]
pub struct ExecutionLogSummaryRecord {
    pub execution_id: String,
    pub job_id: String,
    pub job_name: String,
    pub job_group: String,
    pub task_key: String,
    pub invoke_target: String,
    pub trigger_type: String,
    pub scheduled_at: DateTime<Utc>,
    pub outcome: Option<String>,
    pub message_key: Option<String>,
    pub message_params: Value,
    pub error_key: Option<String>,
    pub error_params: Value,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub create_time: DateTime<Utc>,
    pub has_detail: bool,
}

#[derive(Clone, Debug, FromRow)]
pub struct ExecutionLogDetailRecord {
    #[sqlx(flatten)]
    pub summary: ExecutionLogSummaryRecord,
    pub job_revision: i64,
    pub requested_by: Option<String>,
    pub task_params: Value,
    pub detail_kind: Option<String>,
    pub detail_schema_version: Option<i16>,
    pub detail_payload: Option<Value>,
}
