use serde_json::Value;
use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, FromRow)]
pub struct OperationSummaryRecord {
    pub oper_id: String,
    pub title: String,
    pub business_type: i16,
    pub method: String,
    pub request_method: String,
    pub operator_type: i16,
    pub oper_name: String,
    pub dept_name: String,
    pub oper_url: String,
    pub oper_ip: String,
    pub oper_location_kind: String,
    pub oper_location: String,
    pub status: i16,
    pub oper_time: OffsetDateTime,
    pub cost_time: i64,
}

#[derive(Debug, FromRow)]
pub struct OperationDetailRecord {
    #[sqlx(flatten)]
    pub summary: OperationSummaryRecord,
    pub request_id: String,
    pub operator_id: Option<String>,
    pub dept_id: Option<String>,
    pub oper_param: String,
    pub json_result: String,
    pub error_msg: String,
}

#[derive(Debug, FromRow)]
pub struct LoginRecord {
    pub info_id: String,
    pub user_id: Option<String>,
    pub user_name: String,
    pub ipaddr: String,
    pub login_location_kind: String,
    pub login_location: String,
    pub browser: String,
    pub os: String,
    pub status: i16,
    pub event_type: String,
    pub message_key: String,
    pub message_params: Value,
    pub login_time: OffsetDateTime,
}
