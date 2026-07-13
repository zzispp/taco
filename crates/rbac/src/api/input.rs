use kernel::{error::LocalizedError, pagination::PageRequest};
use serde::Deserialize;
use time::OffsetDateTime;
use types::http::{DATE_OR_RFC3339_FORMAT, DateTimeRange, DateTimeRangeError, parse_date_time_range};

use crate::application::{MenuListFilter, RbacError, RbacResult, RoleListFilter};

const INVALID_DATE_FILTER_KEY: &str = "errors.rbac.invalid_date_filter";
const INVALID_DATE_RANGE_KEY: &str = "errors.rbac.invalid_date_range";

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct RoleFilterQuery {
    role_name: Option<String>,
    role_key: Option<String>,
    status: Option<String>,
    system: Option<bool>,
    begin_time: Option<String>,
    end_time: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct RoleListQuery {
    page: u64,
    page_size: u64,
    #[serde(flatten)]
    filter: RoleFilterQuery,
}

pub(super) type RoleExportQuery = RoleFilterQuery;

pub(super) struct RoleExportFilter {
    role_name: Option<String>,
    role_key: Option<String>,
    status: Option<String>,
    system: Option<bool>,
    begin_time: Option<OffsetDateTime>,
    end_time: Option<OffsetDateTime>,
}

impl RoleExportFilter {
    pub(super) fn page_filter(&self, page: PageRequest) -> RoleListFilter {
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

#[derive(Debug, Deserialize)]
pub(super) struct MenuListQuery {
    page: u64,
    page_size: u64,
    menu_name: Option<String>,
    status: Option<String>,
    begin_time: Option<String>,
    end_time: Option<String>,
}

pub(super) fn role_list_filter(query: RoleListQuery) -> RbacResult<RoleListFilter> {
    let page = PageRequest {
        page: query.page,
        page_size: query.page_size,
    };
    role_export_filter(query.filter).map(|filter| filter.page_filter(page))
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
        page: PageRequest {
            page: query.page,
            page_size: query.page_size,
        },
        menu_name: query.menu_name,
        status: query.status,
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
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
    use super::*;
    use time::format_description::well_known::Rfc3339;

    const PAGE: PageRequest = PageRequest { page: 1, page_size: 10 };

    #[test]
    fn role_filter_parses_rfc3339_as_exact_instants() {
        let query = RoleFilterQuery {
            begin_time: Some("2026-07-11T16:30:00.123+08:00".into()),
            end_time: Some("2026-07-11T08:30:00.123Z".into()),
            ..Default::default()
        };

        let filter = role_export_filter(query).unwrap().page_filter(PAGE);

        assert_eq!(filter.begin_time, filter.end_time);
        assert_eq!(filter.begin_time.unwrap().unix_timestamp_nanos(), 1_783_758_600_123_000_000);
    }

    #[test]
    fn role_filter_accepts_legacy_date_as_a_complete_utc_day() {
        let query = RoleFilterQuery {
            begin_time: Some("2026-07-11".into()),
            end_time: Some("2026-07-11".into()),
            ..Default::default()
        };

        let filter = role_export_filter(query).unwrap().page_filter(PAGE);

        assert_eq!(filter.begin_time, Some(OffsetDateTime::parse("2026-07-11T00:00:00Z", &Rfc3339).unwrap()));
        assert_eq!(
            filter.end_time,
            Some(OffsetDateTime::parse("2026-07-11T23:59:59.999999999Z", &Rfc3339).unwrap())
        );
    }

    #[test]
    fn role_filter_invalid_date_reports_the_field_and_supported_format() {
        let query = RoleFilterQuery {
            begin_time: Some("invalid".into()),
            ..Default::default()
        };

        let Err(RbacError::InvalidInput(error)) = role_export_filter(query) else {
            panic!("expected invalid date filter");
        };
        assert_eq!(error.key(), INVALID_DATE_FILTER_KEY);
        assert_eq!(error.params()[0].key(), "field");
        assert_eq!(error.params()[0].value(), "begin_time");
        assert_eq!(error.params()[1].key(), "format");
        assert_eq!(error.params()[1].value(), "YYYY-MM-DD / RFC3339");
    }

    #[test]
    fn menu_filter_rejects_reversed_time_range() {
        let query = MenuListQuery {
            page: 1,
            page_size: 10,
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
}
