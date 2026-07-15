use std::collections::BTreeMap;

use crate::{
    application::{AuditError, AuditResult},
    domain::{AuditLocation, AuditStatus, BusinessType, LoginEventType, LoginLog, OperationLogDetail, OperationLogSummary, OperatorType},
};

use super::records::{LoginRecord, OperationDetailRecord, OperationSummaryRecord};

pub fn operation_summary(record: OperationSummaryRecord) -> AuditResult<OperationLogSummary> {
    Ok(OperationLogSummary {
        id: record.oper_id,
        title_key: record.title,
        business_type: parse_code(record.business_type, BusinessType::parse, "business_type")?,
        handler: record.method,
        request_method: record.request_method,
        operator_type: parse_code(record.operator_type, OperatorType::parse, "operator_type")?,
        operator_name: record.oper_name,
        department_name: record.dept_name,
        operation_url: record.oper_url,
        operation_ip: record.oper_ip,
        operation_location: location(record.oper_location_kind, record.oper_location, "operation")?,
        status: parse_code(record.status, AuditStatus::parse, "operation status")?,
        operation_time: record.oper_time,
        cost_time_ms: record.cost_time,
    })
}

pub fn operation_detail(record: OperationDetailRecord) -> AuditResult<OperationLogDetail> {
    Ok(OperationLogDetail {
        summary: operation_summary(record.summary)?,
        request_id: record.request_id,
        operator_id: record.operator_id,
        department_id: record.dept_id,
        request_params: record.oper_param,
        response_result: record.json_result,
        error_message: record.error_msg,
    })
}

pub fn login(record: LoginRecord) -> AuditResult<LoginLog> {
    let message_params = serde_json::from_value::<BTreeMap<String, String>>(record.message_params)
        .map_err(|error| invalid_record(format!("invalid login message parameters: {error}")))?;
    Ok(LoginLog {
        id: record.info_id,
        user_id: record.user_id,
        username: record.user_name,
        ip_address: record.ipaddr,
        login_location: location(record.login_location_kind, record.login_location, "login")?,
        browser: record.browser,
        os: record.os,
        status: parse_code(record.status, AuditStatus::parse, "login status")?,
        event_type: LoginEventType::parse(&record.event_type).ok_or_else(|| invalid_record("invalid login event_type"))?,
        message_key: record.message_key,
        message_params,
        login_time: record.login_time,
    })
}

pub fn sqlx_error(error: sqlx::Error) -> AuditError {
    match error {
        sqlx::Error::RowNotFound => AuditError::NotFound,
        other => AuditError::Infrastructure(other.to_string()),
    }
}

fn parse_code<T>(code: i16, parser: impl FnOnce(i16) -> Option<T>, field: &str) -> AuditResult<T> {
    parser(code).ok_or_else(|| invalid_record(format!("invalid {field}: {code}")))
}

fn location(kind: String, text: String, record_type: &str) -> AuditResult<AuditLocation> {
    AuditLocation::from_persisted(&kind, text).ok_or_else(|| invalid_record(format!("invalid {record_type} location kind/text combination: {kind}")))
}

fn invalid_record(message: impl Into<String>) -> AuditError {
    AuditError::Infrastructure(message.into())
}
