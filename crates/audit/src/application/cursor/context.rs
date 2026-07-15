use kernel::pagination::{CursorContext, CursorPageRequest, cursor_fingerprint};
use serde_json::json;
use time::OffsetDateTime;

use crate::{
    application::{AuditError, AuditResult, localized},
    domain::{LoginLogFilter, OperationLogFilter},
};

use super::cursor_encode_error;

const OPERATION_RESOURCE: &str = "audit.operation_logs";
const LOGIN_RESOURCE: &str = "audit.login_logs";
const GLOBAL_SCOPE: &str = "audit.global";

pub(super) fn operation_context<'a>(sort: &'a str, fingerprint: &'a str, limit: u64) -> CursorContext<'a> {
    CursorContext {
        resource: OPERATION_RESOURCE,
        sort,
        filter_fingerprint: fingerprint,
        scope_fingerprint: GLOBAL_SCOPE,
        limit,
    }
}

pub(super) fn login_context<'a>(sort: &'a str, fingerprint: &'a str, limit: u64) -> CursorContext<'a> {
    CursorContext {
        resource: LOGIN_RESOURCE,
        sort,
        filter_fingerprint: fingerprint,
        scope_fingerprint: GLOBAL_SCOPE,
        limit,
    }
}

pub(super) fn operation_fingerprint(filter: &OperationLogFilter) -> AuditResult<String> {
    let mut business_types = filter.business_types.iter().map(|value| value.code()).collect::<Vec<_>>();
    business_types.sort_unstable();
    business_types.dedup();
    let mut title_keys = filter.title_keys.clone();
    title_keys.sort_unstable();
    title_keys.dedup();
    fingerprint(&json!({
        "title": filter.title.as_deref(),
        "title_keys": title_keys,
        "business_types": business_types,
        "status": filter.status.map(|value| value.code()),
        "operator_name": filter.operator_name.as_deref(),
        "operation_ip": filter.operation_ip.as_deref(),
        "begin_time": timestamp_value(filter.begin_time),
        "end_time": timestamp_value(filter.end_time),
    }))
}

pub(super) fn login_fingerprint(filter: &LoginLogFilter) -> AuditResult<String> {
    fingerprint(&json!({
        "username": filter.username.as_deref(),
        "ip_address": filter.ip_address.as_deref(),
        "status": filter.status.map(|value| value.code()),
        "event_type": filter.event_type.map(|value| value.code()),
        "begin_time": timestamp_value(filter.begin_time),
        "end_time": timestamp_value(filter.end_time),
    }))
}

pub(super) fn sort_key(field: &str, direction: &str, id: &str) -> String {
    format!("{field}:{direction},{id}:{direction}")
}

pub(super) fn validate_page(page: &CursorPageRequest) -> AuditResult<()> {
    page.validate().map_err(|_| {
        AuditError::InvalidInput(
            localized("errors.validation.cursor_limit_range")
                .with_param("min", kernel::pagination::MIN_CURSOR_LIMIT.to_string())
                .with_param("max", kernel::pagination::MAX_CURSOR_LIMIT.to_string()),
        )
    })
}

fn fingerprint(value: &serde_json::Value) -> AuditResult<String> {
    cursor_fingerprint(value).map_err(cursor_encode_error)
}

fn timestamp_value(value: Option<OffsetDateTime>) -> Option<String> {
    value.map(|value| value.unix_timestamp_nanos().to_string())
}
