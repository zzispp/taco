use kernel::pagination::{CursorPageRequest, DEFAULT_CURSOR_LIMIT};
use types::http::{DateTimeRangeError, parse_date_time_range};

use crate::{
    application::{ObservabilityError, ObservabilityResult, localized, localized_param},
    domain::{SystemLogFilter, SystemLogLevel},
};

use super::dto::{SystemLogCleanupQuery, SystemLogExportQuery, SystemLogListQuery};

pub fn list_filter(query: &SystemLogListQuery) -> ObservabilityResult<SystemLogFilter> {
    filter(FilterInput::from_list(query))
}

pub fn export_filter(query: &SystemLogExportQuery) -> ObservabilityResult<SystemLogFilter> {
    required_range(filter(FilterInput::from_export(query))?)
}

pub fn cleanup_filter(query: &SystemLogCleanupQuery) -> ObservabilityResult<SystemLogFilter> {
    required_range(filter(FilterInput::from_cleanup(query))?)
}

pub fn page(limit: Option<u64>, cursor: Option<String>) -> CursorPageRequest {
    CursorPageRequest {
        limit: limit.unwrap_or(DEFAULT_CURSOR_LIMIT),
        cursor,
    }
}

struct FilterInput<'a> {
    keyword: Option<&'a str>,
    levels: Option<&'a str>,
    target: Option<&'a str>,
    begin_time: Option<&'a str>,
    end_time: Option<&'a str>,
}

impl<'a> FilterInput<'a> {
    fn from_list(value: &'a SystemLogListQuery) -> Self {
        Self {
            keyword: value.keyword.as_deref(),
            levels: value.levels.as_deref(),
            target: value.target.as_deref(),
            begin_time: value.begin_time.as_deref(),
            end_time: value.end_time.as_deref(),
        }
    }

    fn from_export(value: &'a SystemLogExportQuery) -> Self {
        Self {
            keyword: value.keyword.as_deref(),
            levels: value.levels.as_deref(),
            target: value.target.as_deref(),
            begin_time: value.begin_time.as_deref(),
            end_time: value.end_time.as_deref(),
        }
    }

    fn from_cleanup(value: &'a SystemLogCleanupQuery) -> Self {
        Self {
            keyword: value.keyword.as_deref(),
            levels: value.levels.as_deref(),
            target: value.target.as_deref(),
            begin_time: value.begin_time.as_deref(),
            end_time: value.end_time.as_deref(),
        }
    }
}

fn filter(input: FilterInput<'_>) -> ObservabilityResult<SystemLogFilter> {
    let range = parse_range(input.begin_time, input.end_time)?;
    Ok(SystemLogFilter {
        keyword: clean(input.keyword),
        levels: parse_levels(input.levels)?,
        target: clean(input.target),
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}

fn parse_levels(value: Option<&str>) -> ObservabilityResult<Vec<SystemLogLevel>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(Vec::new());
    };
    let mut levels = value
        .split(',')
        .map(str::trim)
        .map(|code| SystemLogLevel::parse(code).ok_or_else(|| ObservabilityError::InvalidInput(localized("errors.observability.invalid_level"))))
        .collect::<ObservabilityResult<Vec<_>>>()?;
    levels.sort_unstable_by_key(|level| level.code());
    levels.dedup_by_key(|level| level.code());
    Ok(levels)
}

fn parse_range(begin: Option<&str>, end: Option<&str>) -> ObservabilityResult<types::http::DateTimeRange> {
    parse_date_time_range(begin, end).map_err(|error| match error {
        DateTimeRangeError::InvalidBoundary(field) => {
            ObservabilityError::InvalidInput(localized_param("errors.observability.invalid_date", "field", field.as_str()))
        }
        DateTimeRangeError::Reversed => ObservabilityError::InvalidInput(localized("errors.observability.invalid_date_range")),
    })
}

fn required_range(filter: SystemLogFilter) -> ObservabilityResult<SystemLogFilter> {
    if filter.begin_time.is_none() || filter.end_time.is_none() {
        return Err(ObservabilityError::InvalidInput(localized("errors.observability.time_range_required")));
    }
    Ok(filter)
}

fn clean(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|value| !value.is_empty()).map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use crate::api::dto::{SystemLogExportQuery, SystemLogListQuery};
    use crate::domain::SystemLogLevel;

    use super::{export_filter, list_filter, page};

    #[test]
    fn list_filter_deduplicates_levels_and_preserves_optional_time_range() {
        let query = SystemLogListQuery {
            limit: None,
            cursor: None,
            keyword: Some(" request ".into()),
            levels: Some("error,info,error".into()),
            target: Some(" api ".into()),
            begin_time: None,
            end_time: None,
        };

        let filter = list_filter(&query).unwrap();

        assert_eq!(filter.keyword.as_deref(), Some("request"));
        assert_eq!(filter.levels, [SystemLogLevel::Error, SystemLogLevel::Info]);
        assert_eq!(filter.target.as_deref(), Some("api"));
        assert!(filter.begin_time.is_none());
    }

    #[test]
    fn export_requires_an_explicit_time_range() {
        let query = SystemLogExportQuery {
            keyword: None,
            levels: None,
            target: None,
            begin_time: None,
            end_time: None,
        };

        assert!(export_filter(&query).is_err());
        assert_eq!(page(None, Some("cursor".into())).limit, kernel::pagination::DEFAULT_CURSOR_LIMIT);
    }
}
