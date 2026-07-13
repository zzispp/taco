use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Debug, Deserialize)]
pub struct JobListQuery {
    #[serde(default)]
    pub page: u64,
    #[serde(default)]
    pub page_size: u64,
    pub job_name: Option<String>,
    pub job_group: Option<String>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct JobLogListQuery {
    #[serde(default)]
    pub page: u64,
    #[serde(default)]
    pub page_size: u64,
    pub job_name: Option<String>,
    pub job_group: Option<String>,
    pub status: Option<String>,
    pub trigger_type: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ImportJobRequest {
    pub task_key: String,
    pub job_name: String,
    pub job_group: String,
    pub cron_expression: String,
    pub misfire_policy: String,
    pub concurrent: String,
    pub task_params: Value,
    pub remark: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReplaceJobRequest {
    pub job_name: String,
    pub job_group: String,
    pub cron_expression: String,
    pub misfire_policy: String,
    pub concurrent: String,
    pub task_params: Value,
    pub remark: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UpdateJobStatusRequest {
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BatchIdsRequest {
    pub ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CronNextTimesRequest {
    pub expression: String,
    pub count: Option<u8>,
}
