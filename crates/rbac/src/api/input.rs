use kernel::{error::LocalizedError, pagination::CursorPageRequest};
use serde::Deserialize;
use time::OffsetDateTime;
use types::http::{DATE_OR_RFC3339_FORMAT, DateTimeRange, DateTimeRangeError, parse_date_time_range};
use utoipa::IntoParams;

use crate::application::{MenuListFilter, RbacError, RbacResult, RoleListFilter};

const INVALID_DATE_FILTER_KEY: &str = "errors.rbac.invalid_date_filter";
const INVALID_DATE_RANGE_KEY: &str = "errors.rbac.invalid_date_range";

#[derive(Clone, Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub(super) struct RoleExportQuery {
    role_name: Option<String>,
    role_key: Option<String>,
    status: Option<String>,
    system: Option<bool>,
    begin_time: Option<String>,
    end_time: Option<String>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub(super) struct RoleListQuery {
    #[serde(default = "default_limit")]
    #[param(default = 20, minimum = 1, maximum = 100)]
    limit: u64,
    #[serde(default)]
    cursor: Option<String>,
    role_name: Option<String>,
    role_key: Option<String>,
    status: Option<String>,
    system: Option<bool>,
    begin_time: Option<String>,
    end_time: Option<String>,
}

pub(super) struct RoleExportFilter {
    role_name: Option<String>,
    role_key: Option<String>,
    status: Option<String>,
    system: Option<bool>,
    begin_time: Option<OffsetDateTime>,
    end_time: Option<OffsetDateTime>,
}

impl RoleExportFilter {
    pub(super) fn page_filter(&self, page: CursorPageRequest) -> RoleListFilter {
        RoleListFilter {
            page,
            role_name: self.role_name.clone(),
            role_key: self.role_key.clone(),
            status: self.status.clone(),
            system: self.system,
            begin_time: self.begin_time,
            end_time: self.end_time,
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub(super) struct MenuListQuery {
    #[serde(default = "default_limit")]
    #[param(default = 20, minimum = 1, maximum = 100)]
    limit: u64,
    #[serde(default)]
    cursor: Option<String>,
    menu_name: Option<String>,
    status: Option<String>,
    begin_time: Option<String>,
    end_time: Option<String>,
}

pub(super) fn role_list_filter(query: RoleListQuery) -> RbacResult<RoleListFilter> {
    let page = CursorPageRequest {
        limit: query.limit,
        cursor: query.cursor.clone(),
    };
    role_export_filter(RoleExportQuery {
        role_name: query.role_name,
        role_key: query.role_key,
        status: query.status,
        system: query.system,
        begin_time: query.begin_time,
        end_time: query.end_time,
    })
    .map(|filter| filter.page_filter(page))
}

pub(super) fn role_export_filter(query: RoleExportQuery) -> RbacResult<RoleExportFilter> {
    let range = created_time_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    Ok(RoleExportFilter {
        role_name: query.role_name,
        role_key: query.role_key,
        status: query.status,
        system: query.system,
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}

pub(super) fn menu_list_filter(query: MenuListQuery) -> RbacResult<MenuListFilter> {
    let range = created_time_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    Ok(MenuListFilter {
        page: CursorPageRequest {
            limit: query.limit,
            cursor: query.cursor,
        },
        menu_name: query.menu_name,
        status: query.status,
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}

const fn default_limit() -> u64 {
    kernel::pagination::DEFAULT_CURSOR_LIMIT
}

fn created_time_range(begin: Option<&str>, end: Option<&str>) -> RbacResult<DateTimeRange> {
    parse_date_time_range(begin, end).map_err(map_date_range_error)
}

fn map_date_range_error(error: DateTimeRangeError) -> RbacError {
    let message = match error {
        DateTimeRangeError::InvalidBoundary(field) => LocalizedError::new(INVALID_DATE_FILTER_KEY)
            .with_param("field", field.as_str())
            .with_param("format", DATE_OR_RFC3339_FORMAT),
        DateTimeRangeError::Reversed => LocalizedError::new(INVALID_DATE_RANGE_KEY),
    };
    RbacError::InvalidInput(message)
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use serde_json::Value;
    use time::format_description::well_known::Rfc3339;
    use tower::ServiceExt;
    use types::http::RequestQuery;

    use super::*;

    fn page() -> CursorPageRequest {
        CursorPageRequest { limit: 10, cursor: None }
    }

    #[test]
    fn role_filter_parses_rfc3339_as_exact_instants() {
        let query = RoleExportQuery {
            begin_time: Some("2026-07-11T16:30:00.123+08:00".into()),
            end_time: Some("2026-07-11T08:30:00.123Z".into()),
            ..Default::default()
        };

        let filter = role_export_filter(query).unwrap().page_filter(page());

        assert_eq!(filter.begin_time, filter.end_time);
        assert_eq!(filter.begin_time.unwrap().unix_timestamp_nanos(), 1_783_758_600_123_000_000);
    }

    #[test]
    fn role_filter_accepts_legacy_date_as_a_complete_utc_day() {
        let query = RoleExportQuery {
            begin_time: Some("2026-07-11".into()),
            end_time: Some("2026-07-11".into()),
            ..Default::default()
        };

        let filter = role_export_filter(query).unwrap().page_filter(page());

        assert_eq!(filter.begin_time, Some(OffsetDateTime::parse("2026-07-11T00:00:00Z", &Rfc3339).unwrap()));
        assert_eq!(
            filter.end_time,
            Some(OffsetDateTime::parse("2026-07-11T23:59:59.999999999Z", &Rfc3339).unwrap())
        );
    }

    #[test]
    fn role_filter_invalid_date_reports_field_and_format() {
        let query = RoleExportQuery {
            begin_time: Some("invalid".into()),
            ..Default::default()
        };

        let Err(RbacError::InvalidInput(error)) = role_export_filter(query) else {
            panic!("expected invalid date filter");
        };
        assert_eq!(error.key(), INVALID_DATE_FILTER_KEY);
        assert_eq!(error.params()[0].value(), "begin_time");
        assert_eq!(error.params()[1].value(), "YYYY-MM-DD / RFC3339");
    }

    #[test]
    fn menu_filter_rejects_reversed_time_range() {
        let query = MenuListQuery {
            limit: 10,
            cursor: None,
            menu_name: None,
            status: None,
            begin_time: Some("2026-07-11T08:30:00.001Z".into()),
            end_time: Some("2026-07-11T08:30:00.000Z".into()),
        };

        let Err(RbacError::InvalidInput(error)) = menu_list_filter(query) else {
            panic!("expected invalid date range");
        };
        assert_eq!(error.key(), INVALID_DATE_RANGE_KEY);
    }

    #[tokio::test]
    async fn legacy_role_and_menu_page_parameters_are_rejected_by_routes() {
        let app = Router::new()
            .route("/roles", get(|RequestQuery(_): RequestQuery<RoleListQuery>| async {}))
            .route("/menus", get(|RequestQuery(_): RequestQuery<MenuListQuery>| async {}));

        for uri in ["/roles?page=1", "/menus?page_size=20"] {
            let response = app.clone().oneshot(Request::get(uri).body(Body::empty()).unwrap()).await.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let body = serde_json::from_slice::<Value>(&bytes).unwrap();
            assert_eq!(body["code"], "invalid_input");
        }
    }
}
