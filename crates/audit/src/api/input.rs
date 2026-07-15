use kernel::pagination::{CursorPageRequest, DEFAULT_CURSOR_LIMIT};
use types::http::{DateTimeRangeError, Locale, parse_date_time_range, translate_message};

use crate::{
    application::{AuditError, AuditResult, localized, localized_param},
    domain::{AuditStatus, BusinessType, LoginEventType, LoginLogFilter, LoginSortField, OperationLogFilter, OperationSortField, SortDirection},
};

use super::dto::{LoginLogExportQuery, LoginLogListQuery, OperationLogExportQuery, OperationLogListQuery};

pub fn operation_filter(query: &OperationLogListQuery, locale: Locale) -> AuditResult<OperationLogFilter> {
    let range = parse_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    let title = clean(query.title.as_deref());
    Ok(OperationLogFilter {
        title_keys: matching_title_keys(title.as_deref(), locale),
        title,
        business_types: parse_business_types(query.business_type.as_deref())?,
        status: parse_status(query.status)?,
        operator_name: clean(query.oper_name.as_deref()),
        operation_ip: clean(query.oper_ip.as_deref()),
        begin_time: range.begin_time,
        end_time: range.end_time,
        sort_field: parse_operation_sort(query.sort_by.as_deref())?,
        sort_direction: parse_direction(query.sort_order.as_deref())?,
    })
}

pub fn login_filter(query: &LoginLogListQuery) -> AuditResult<LoginLogFilter> {
    let range = parse_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    Ok(LoginLogFilter {
        username: clean(query.user_name.as_deref()),
        ip_address: clean(query.ipaddr.as_deref()),
        status: parse_status(query.status)?,
        event_type: parse_event_type(query.event_type.as_deref())?,
        begin_time: range.begin_time,
        end_time: range.end_time,
        sort_field: parse_login_sort(query.sort_by.as_deref())?,
        sort_direction: parse_direction(query.sort_order.as_deref())?,
    })
}

pub fn operation_export_filter(query: OperationLogExportQuery, locale: Locale) -> AuditResult<OperationLogFilter> {
    operation_filter(
        &OperationLogListQuery {
            limit: None,
            cursor: None,
            title: query.title,
            oper_name: query.oper_name,
            oper_ip: query.oper_ip,
            business_type: query.business_type,
            status: query.status,
            begin_time: query.begin_time,
            end_time: query.end_time,
            sort_by: query.sort_by,
            sort_order: query.sort_order,
        },
        locale,
    )
}

pub fn login_export_filter(query: LoginLogExportQuery) -> AuditResult<LoginLogFilter> {
    login_filter(&LoginLogListQuery {
        limit: None,
        cursor: None,
        ipaddr: query.ipaddr,
        user_name: query.user_name,
        status: query.status,
        event_type: query.event_type,
        begin_time: query.begin_time,
        end_time: query.end_time,
        sort_by: query.sort_by,
        sort_order: query.sort_order,
    })
}

pub fn page(limit: Option<u64>, cursor: Option<String>) -> CursorPageRequest {
    CursorPageRequest {
        limit: limit.unwrap_or(DEFAULT_CURSOR_LIMIT),
        cursor,
    }
}

fn parse_business_types(value: Option<&str>) -> AuditResult<Vec<BusinessType>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(Vec::new());
    };
    value
        .split(',')
        .map(str::trim)
        .map(|code| code.parse::<i16>().ok().and_then(BusinessType::parse).ok_or_else(invalid_business_type))
        .collect()
}

fn parse_status(value: Option<i16>) -> AuditResult<Option<AuditStatus>> {
    value
        .map(|code| AuditStatus::parse(code).ok_or_else(|| invalid("errors.audit.invalid_status")))
        .transpose()
}

fn parse_event_type(value: Option<&str>) -> AuditResult<Option<LoginEventType>> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|code| LoginEventType::parse(code).ok_or_else(|| invalid("errors.audit.invalid_event_type")))
        .transpose()
}

fn parse_operation_sort(value: Option<&str>) -> AuditResult<OperationSortField> {
    parse_sort(value, OperationSortField::parse)
}

fn parse_login_sort(value: Option<&str>) -> AuditResult<LoginSortField> {
    parse_sort(value, LoginSortField::parse)
}

fn parse_sort<T: Default>(value: Option<&str>, parser: impl FnOnce(&str) -> Option<T>) -> AuditResult<T> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => parser(value).ok_or_else(|| invalid("errors.audit.invalid_sort_field")),
        None => Ok(T::default()),
    }
}

fn parse_direction(value: Option<&str>) -> AuditResult<SortDirection> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        Some(value) => SortDirection::parse(value).ok_or_else(|| invalid("errors.audit.invalid_sort_order")),
        None => Ok(SortDirection::default()),
    }
}

fn parse_range(begin: Option<&str>, end: Option<&str>) -> AuditResult<types::http::DateTimeRange> {
    parse_date_time_range(begin, end).map_err(|error| match error {
        DateTimeRangeError::InvalidBoundary(field) => AuditError::InvalidInput(localized_param("errors.audit.invalid_date", "field", field.as_str())),
        DateTimeRangeError::Reversed => invalid("errors.audit.invalid_date_range"),
    })
}

fn clean(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|value| !value.is_empty()).map(str::to_owned)
}

const MODULE_TITLE_KEYS: &[&str] = &[
    "audit.module.profile",
    "audit.module.online",
    "audit.module.user",
    "audit.module.role",
    "audit.module.menu",
    "audit.module.department",
    "audit.module.post",
    "audit.module.dict_type",
    "audit.module.dict_data",
    "audit.module.config",
    "audit.module.notice",
    "audit.module.job",
    "audit.module.job_log",
    "audit.module.operation_log",
    "audit.module.login_log",
];

fn matching_title_keys(title: Option<&str>, locale: Locale) -> Vec<String> {
    let Some(title) = title else { return Vec::new() };
    let needle = title.to_lowercase();
    MODULE_TITLE_KEYS
        .iter()
        .filter(|key| translate_message(locale, key).to_lowercase().contains(&needle))
        .map(|key| (*key).into())
        .collect()
}

fn invalid(key: &'static str) -> AuditError {
    AuditError::InvalidInput(localized(key))
}

fn invalid_business_type() -> AuditError {
    invalid("errors.audit.invalid_business_type")
}

#[cfg(test)]
mod tests {
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use super::{login_filter, operation_filter, page};
    use crate::{
        api::dto::{LoginLogExportQuery, LoginLogListQuery, OperationLogExportQuery, OperationLogListQuery},
        domain::{BusinessType, LoginSortField, OperationSortField, SortDirection},
    };
    use types::http::Locale;

    #[test]
    fn operation_query_parses_filters_closed_dates_and_sort() {
        let mut query = operation_query();
        query.business_type = Some("1, 3".into());
        query.begin_time = Some("2026-07-13".into());
        query.end_time = Some("2026-07-13".into());
        query.sort_by = Some("cost_time".into());
        query.sort_order = Some("asc".into());

        let filter = operation_filter(&query, Locale::En).unwrap();

        assert_eq!(filter.business_types, [BusinessType::Insert, BusinessType::Delete]);
        assert_eq!(filter.sort_field, OperationSortField::CostTime);
        assert_eq!(filter.sort_direction, SortDirection::Asc);
        assert_eq!(filter.begin_time, Some(timestamp("2026-07-13T00:00:00Z")));
        assert_eq!(filter.end_time, Some(timestamp("2026-07-13T23:59:59.999999999Z")));
    }

    #[test]
    fn illegal_sort_and_codes_fail_explicitly() {
        let mut operation = operation_query();
        operation.sort_by = Some("oper_time desc".into());
        assert!(operation_filter(&operation, Locale::En).is_err());
        operation.sort_by = None;
        operation.business_type = Some("10".into());
        assert!(operation_filter(&operation, Locale::En).is_err());

        let mut login = login_query();
        login.sort_by = Some("event_type".into());
        assert!(login_filter(&login).is_err());
    }

    #[test]
    fn title_sort_is_rejected_with_the_stable_localization_key() {
        let mut query = operation_query();
        query.sort_by = Some("title".into());

        let error = operation_filter(&query, Locale::En).unwrap_err();

        assert!(matches!(
            error,
            crate::application::AuditError::InvalidInput(error) if error.key() == "errors.audit.invalid_sort_field"
        ));
    }

    fn operation_query() -> OperationLogListQuery {
        OperationLogListQuery {
            limit: Some(20),
            cursor: None,
            title: None,
            oper_name: None,
            oper_ip: None,
            business_type: None,
            status: None,
            begin_time: None,
            end_time: None,
            sort_by: None,
            sort_order: None,
        }
    }

    fn login_query() -> LoginLogListQuery {
        LoginLogListQuery {
            limit: Some(20),
            cursor: None,
            ipaddr: None,
            user_name: None,
            status: None,
            event_type: None,
            begin_time: None,
            end_time: None,
            sort_by: None,
            sort_order: None,
        }
    }

    fn timestamp(value: &str) -> OffsetDateTime {
        OffsetDateTime::parse(value, &Rfc3339).unwrap()
    }

    #[test]
    fn defaults_are_stable() {
        assert_eq!(
            operation_filter(&operation_query(), Locale::En).unwrap().sort_field,
            OperationSortField::OperationTime
        );
        assert_eq!(login_filter(&login_query()).unwrap().sort_field, LoginSortField::LoginTime);
        assert_eq!(page(None, None).limit, 20);
        assert_eq!(page(Some(50), Some("cursor".into())).cursor.as_deref(), Some("cursor"));
    }

    #[test]
    fn localized_title_filter_maps_to_stable_persistent_keys() {
        let mut query = operation_query();
        query.title = Some("User management".into());
        let filter = operation_filter(&query, Locale::En).unwrap();
        assert_eq!(filter.title_keys, ["audit.module.user"]);
    }

    #[test]
    fn deleted_page_number_parameters_are_rejected() {
        assert!(serde_json::from_value::<OperationLogListQuery>(serde_json::json!({"page": 1})).is_err());
        assert!(serde_json::from_value::<LoginLogListQuery>(serde_json::json!({"page_size": 20})).is_err());
        assert!(serde_json::from_value::<OperationLogExportQuery>(serde_json::json!({"cursor": "opaque"})).is_err());
        assert!(serde_json::from_value::<LoginLogExportQuery>(serde_json::json!({"limit": 20})).is_err());
    }
}
