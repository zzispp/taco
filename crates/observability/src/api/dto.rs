use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoParams, ToSchema};

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub struct SystemLogListQuery {
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub limit: Option<u64>,
    pub cursor: Option<String>,
    pub keyword: Option<String>,
    pub levels: Option<String>,
    pub target: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub struct SystemLogExportQuery {
    pub keyword: Option<String>,
    pub levels: Option<String>,
    pub target: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub struct SystemLogCleanupQuery {
    pub keyword: Option<String>,
    pub levels: Option<String>,
    pub target: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct BatchIdsRequest {
    pub ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct SystemLogSummaryResponse {
    pub log_id: String,
    pub occurred_at: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct SystemLogDetailResponse {
    #[serde(flatten)]
    pub summary: SystemLogSummaryResponse,
    pub fields: Value,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct SystemLogCleanupCountResponse {
    pub count: u64,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct SystemLogCleanupAcceptedResponse {
    pub accepted: bool,
    pub execution_id: String,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct SystemLogCleanupExecutionResponse {
    pub execution_id: String,
    pub state: String,
    pub deleted: Option<u64>,
    pub batches: Option<u64>,
}
