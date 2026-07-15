use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub struct OperationLogListQuery {
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub limit: Option<u64>,
    pub cursor: Option<String>,
    pub title: Option<String>,
    pub oper_name: Option<String>,
    pub oper_ip: Option<String>,
    pub business_type: Option<String>,
    pub status: Option<i16>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
    /// One of `business_type`, `oper_name`, `status`, `oper_time`, or `cost_time`.
    pub sort_by: Option<String>,
    /// Either `asc` or `desc`.
    pub sort_order: Option<String>,
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub struct OperationLogExportQuery {
    pub title: Option<String>,
    pub oper_name: Option<String>,
    pub oper_ip: Option<String>,
    pub business_type: Option<String>,
    pub status: Option<i16>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub struct LoginLogListQuery {
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub limit: Option<u64>,
    pub cursor: Option<String>,
    pub ipaddr: Option<String>,
    pub user_name: Option<String>,
    pub status: Option<i16>,
    pub event_type: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
    /// One of `user_name`, `ipaddr`, `status`, or `login_time`.
    pub sort_by: Option<String>,
    /// Either `asc` or `desc`.
    pub sort_order: Option<String>,
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub struct LoginLogExportQuery {
    pub ipaddr: Option<String>,
    pub user_name: Option<String>,
    pub status: Option<i16>,
    pub event_type: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Clone, Debug, Deserialize, ToSchema)]
pub struct BatchIdsRequest {
    pub ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct OperationLogSummaryResponse {
    pub oper_id: String,
    pub title: String,
    pub business_type: i16,
    pub method: String,
    pub request_method: String,
    pub operator_type: i16,
    pub oper_name: Option<String>,
    pub dept_name: Option<String>,
    pub oper_url: String,
    pub oper_ip: String,
    pub oper_location: String,
    pub status: i16,
    pub oper_time: String,
    pub cost_time: i64,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct OperationLogDetailResponse {
    #[serde(flatten)]
    pub summary: OperationLogSummaryResponse,
    pub oper_param: Option<String>,
    pub json_result: Option<String>,
    pub error_msg: Option<String>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct LoginLogResponse {
    pub info_id: String,
    pub user_name: String,
    pub ipaddr: String,
    pub login_location: String,
    pub browser: String,
    pub os: String,
    pub status: i16,
    pub msg: String,
    pub event_type: String,
    pub login_time: String,
}
