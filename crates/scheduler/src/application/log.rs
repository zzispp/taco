use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::domain::{ExecutionDetail, ExecutionOutcome, LocalizedMessage, TriggerType};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionLogSummary {
    pub id: String,
    pub job_id: String,
    pub job_name: String,
    pub job_group: String,
    pub task_key: String,
    pub invoke_target: String,
    pub trigger: TriggerType,
    pub scheduled_at: DateTime<Utc>,
    pub outcome: ExecutionOutcome,
    pub message: LocalizedMessage,
    pub error: Option<LocalizedMessage>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: DateTime<Utc>,
    pub create_time: DateTime<Utc>,
    pub has_detail: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionLogDetail {
    pub summary: ExecutionLogSummary,
    pub job_revision: i64,
    pub requested_by: Option<String>,
    pub task_params: Value,
    pub detail: Option<ExecutionDetail>,
}
