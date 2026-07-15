use types::http::{Locale, format_utc_rfc3339_millis, translate_message, translate_message_with_params};

use crate::{
    application::{AuditError, AuditResult},
    domain::{AuditLocation, LoginLog, OperationLogDetail, OperationLogSummary},
};

use super::dto::{LoginLogResponse, OperationLogDetailResponse, OperationLogSummaryResponse};

pub fn operation_summary(value: OperationLogSummary, locale: Locale) -> AuditResult<OperationLogSummaryResponse> {
    Ok(OperationLogSummaryResponse {
        oper_id: value.id,
        title: translate_message(locale, &value.title_key),
        business_type: value.business_type.code(),
        method: value.handler,
        request_method: value.request_method,
        operator_type: value.operator_type.code(),
        oper_name: nonempty(value.operator_name),
        dept_name: nonempty(value.department_name),
        oper_url: value.operation_url,
        oper_ip: value.operation_ip,
        oper_location: render_location(&value.operation_location, locale),
        status: value.status.code(),
        oper_time: timestamp(value.operation_time)?,
        cost_time: value.cost_time_ms,
    })
}

pub fn operation_detail(value: OperationLogDetail, locale: Locale) -> AuditResult<OperationLogDetailResponse> {
    Ok(OperationLogDetailResponse {
        summary: operation_summary(value.summary, locale)?,
        oper_param: nonempty(value.request_params),
        json_result: nonempty(value.response_result),
        error_msg: nonempty(value.error_message),
    })
}

pub fn login(value: LoginLog, locale: Locale) -> AuditResult<LoginLogResponse> {
    let params = value.message_params.into_iter().collect::<Vec<_>>();
    let params = params.iter().map(|(key, value)| (key.as_str(), value.clone())).collect::<Vec<_>>();
    Ok(LoginLogResponse {
        info_id: value.id,
        user_name: value.username,
        ipaddr: value.ip_address,
        login_location: render_location(&value.login_location, locale),
        browser: value.browser,
        os: value.os,
        status: value.status.code(),
        msg: translate_message_with_params(locale, &value.message_key, &params),
        event_type: value.event_type.code().into(),
        login_time: timestamp(value.login_time)?,
    })
}

fn timestamp(value: time::OffsetDateTime) -> AuditResult<String> {
    format_utc_rfc3339_millis(value).map_err(|error| AuditError::Infrastructure(error.to_string()))
}

fn nonempty(value: String) -> Option<String> {
    if value.is_empty() { None } else { Some(value) }
}

pub(crate) fn render_location(location: &AuditLocation, locale: Locale) -> String {
    match location {
        AuditLocation::Resolved(value) => value.clone(),
        AuditLocation::Internal => translate_message(locale, "messages.client_info.location.internal"),
        AuditLocation::Unknown => translate_message(locale, "messages.client_info.location.unknown"),
    }
}

#[cfg(test)]
mod tests {
    use types::http::Locale;

    use crate::domain::AuditLocation;

    use super::{render_location, timestamp};

    #[test]
    fn api_timestamps_use_utc_rfc3339_with_fixed_milliseconds() {
        assert_eq!(timestamp(time::OffsetDateTime::UNIX_EPOCH).unwrap(), "1970-01-01T00:00:00.000Z");
    }

    #[test]
    fn semantic_locations_are_rendered_in_each_wire_locale() {
        let cases = [
            (Locale::ZhCn, "内网IP", "未知"),
            (Locale::En, "Intranet IP", "Unknown"),
            (Locale::ZhTw, "內網IP", "未知"),
        ];
        for (locale, internal, unknown) in cases {
            assert_eq!(render_location(&AuditLocation::Internal, locale), internal);
            assert_eq!(render_location(&AuditLocation::Unknown, locale), unknown);
            assert_eq!(render_location(&AuditLocation::Resolved("provider text".into()), locale), "provider text");
        }
    }
}
