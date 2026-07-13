use serde::Serialize;
use serde_json::Value;

use crate::domain::{ParamCondition, ParamSchema, ParamWidget};

#[derive(Clone, Debug, Serialize)]
pub struct JobResponse {
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
    pub registry_status: String,
    pub param_form: Option<TaskParamFormResponse>,
    pub schedule_revision: i64,
    pub next_run_at: Option<String>,
    pub runtime_error: Option<RuntimeErrorResponse>,
    pub create_by: String,
    pub create_time: String,
    pub update_by: String,
    pub update_time: Option<String>,
    pub remark: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RuntimeErrorResponse {
    pub code: String,
    pub message: String,
    pub occurred_at: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExecutionLogResponse {
    pub execution_id: String,
    pub job_id: String,
    pub job_name: String,
    pub job_group: String,
    pub task_key: String,
    pub invoke_target: String,
    pub has_detail: bool,
    pub job_message: String,
    pub trigger_type: String,
    pub scheduled_at: String,
    pub status: String,
    pub exception_info: Option<String>,
    pub start_time: Option<String>,
    pub end_time: String,
    pub create_time: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExecutionLogDetailResponse {
    #[serde(flatten)]
    pub summary: ExecutionLogResponse,
    pub job_revision: i64,
    pub requested_by: Option<String>,
    pub task_params: Value,
    pub detail: Option<ExecutionDetailResponse>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ExecutionDetailResponse {
    pub kind: String,
    pub schema_version: i16,
    pub payload: Value,
}

#[derive(Clone, Debug, Serialize)]
pub struct ImportableTaskResponse {
    pub task_key: String,
    pub name: String,
    pub group: String,
    pub group_label: String,
    pub description: String,
    pub repeatable: bool,
    pub default_params: Value,
    pub param_form: TaskParamFormResponse,
}

#[derive(Clone, Debug, Serialize)]
pub struct TaskParamFormResponse {
    pub schema_version: i16,
    pub schema: ParamSchema,
    pub ui: ParamUiResponse,
}

#[derive(Clone, Debug, Serialize)]
pub struct ParamUiResponse {
    pub fields: Vec<ParamFieldResponse>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ParamFieldResponse {
    pub path: String,
    pub label: String,
    pub widget: ParamWidget,
    pub placeholder: Option<String>,
    pub help: Option<String>,
    pub options: Vec<String>,
    pub disabled_when: Option<ParamCondition>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RunJobResponse {
    pub accepted: bool,
    pub execution_id: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CronNextTimesResponse {
    pub times: Vec<String>,
}
