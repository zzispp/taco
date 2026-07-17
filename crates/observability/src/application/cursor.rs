use kernel::pagination::{CursorContext, CursorDirection, CursorPage, CursorPageRequest, cursor_fingerprint, decode_cursor};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::domain::{SystemLogDetail, SystemLogFilter, SystemLogSummary};

use super::{ObservabilityError, ObservabilityResult, localized_param};

const RESOURCE: &str = "system_logs";
const SORT: &str = "occurred_at_desc_log_id_desc";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct SystemLogSnapshot {
    pub ingested_seq: i64,
}

impl SystemLogSnapshot {
    pub const fn new(ingested_seq: i64) -> Self {
        Self { ingested_seq }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct SystemLogBoundary {
    pub occurred_at_nanos: String,
    pub id: String,
}

impl SystemLogBoundary {
    pub fn from_summary(value: &SystemLogSummary) -> Self {
        Self {
            occurred_at_nanos: value.occurred_at.unix_timestamp_nanos().to_string(),
            id: value.id.clone(),
        }
    }

    pub fn occurred_at(&self) -> ObservabilityResult<OffsetDateTime> {
        parse_time(&self.occurred_at_nanos)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemLogCursorQuery {
    pub limit: u64,
    pub direction: CursorDirection,
    pub boundary: Option<SystemLogBoundary>,
    pub snapshot: Option<SystemLogSnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SystemLogCursorSlice {
    pub items: Vec<SystemLogSummary>,
    pub snapshot: Option<SystemLogSnapshot>,
    pub has_next: bool,
    pub has_previous: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SystemLogExportSlice {
    pub items: Vec<SystemLogDetail>,
    pub snapshot: Option<SystemLogSnapshot>,
    pub has_next: bool,
}

pub fn system_log_cursor_query(filter: &SystemLogFilter, page: &CursorPageRequest) -> ObservabilityResult<SystemLogCursorQuery> {
    page.validate().map_err(|_| {
        ObservabilityError::InvalidInput(
            localized_param(
                "errors.observability.cursor_limit_range",
                "min",
                kernel::pagination::MIN_CURSOR_LIMIT.to_string(),
            )
            .with_param("max", kernel::pagination::MAX_CURSOR_LIMIT.to_string()),
        )
    })?;
    let context = cursor_context(filter, page.limit)?;
    let Some(value) = page.cursor.as_deref() else {
        return Ok(SystemLogCursorQuery {
            limit: page.limit,
            direction: CursorDirection::Next,
            boundary: None,
            snapshot: None,
        });
    };
    let decoded = decode_cursor::<SystemLogBoundary, SystemLogSnapshot>(value, &context.borrow()).map_err(|_| ObservabilityError::InvalidCursor)?;
    Ok(SystemLogCursorQuery {
        limit: page.limit,
        direction: decoded.direction,
        boundary: Some(decoded.boundary),
        snapshot: Some(decoded.snapshot),
    })
}

pub fn system_log_cursor_page(
    filter: &SystemLogFilter,
    query: &SystemLogCursorQuery,
    slice: SystemLogCursorSlice,
) -> ObservabilityResult<CursorPage<SystemLogSummary>> {
    if slice.items.is_empty() {
        return empty_page(filter, query, slice.snapshot.as_ref());
    }
    let snapshot = slice
        .snapshot
        .as_ref()
        .ok_or_else(|| ObservabilityError::Infrastructure("system log cursor page is missing its snapshot".into()))?;
    let first = slice
        .items
        .first()
        .ok_or_else(|| ObservabilityError::Infrastructure("system log cursor page lost its first row".into()))?;
    let last = slice
        .items
        .last()
        .ok_or_else(|| ObservabilityError::Infrastructure("system log cursor page lost its last row".into()))?;
    let context = cursor_context(filter, query.limit)?;
    let next = slice
        .has_next
        .then(|| context.borrow().encode(CursorDirection::Next, &SystemLogBoundary::from_summary(last), snapshot))
        .transpose()
        .map_err(cursor_error)?;
    let previous = slice
        .has_previous
        .then(|| {
            context
                .borrow()
                .encode(CursorDirection::Previous, &SystemLogBoundary::from_summary(first), snapshot)
        })
        .transpose()
        .map_err(cursor_error)?;
    Ok(CursorPage::new(slice.items, next, previous))
}

fn empty_page(
    filter: &SystemLogFilter,
    query: &SystemLogCursorQuery,
    snapshot: Option<&SystemLogSnapshot>,
) -> ObservabilityResult<CursorPage<SystemLogSummary>> {
    let Some(boundary) = query.boundary.as_ref() else {
        return Ok(CursorPage::new(Vec::new(), None, None));
    };
    let snapshot = snapshot
        .or(query.snapshot.as_ref())
        .ok_or_else(|| ObservabilityError::Infrastructure("empty system log cursor page is missing its snapshot".into()))?;
    let context = cursor_context(filter, query.limit)?;
    let (next, previous) = match query.direction {
        CursorDirection::Next => (None, Some(context.borrow().encode(CursorDirection::Previous, boundary, snapshot))),
        CursorDirection::Previous => (Some(context.borrow().encode(CursorDirection::Next, boundary, snapshot)), None),
    };
    Ok(CursorPage::new(
        Vec::new(),
        next.transpose().map_err(cursor_error)?,
        previous.transpose().map_err(cursor_error)?,
    ))
}

struct SystemLogCursorContext {
    filter_fingerprint: String,
    limit: u64,
}

impl SystemLogCursorContext {
    fn borrow(&self) -> CursorContext<'_> {
        CursorContext {
            resource: RESOURCE,
            sort: SORT,
            filter_fingerprint: &self.filter_fingerprint,
            scope_fingerprint: "",
            limit: self.limit,
        }
    }
}

fn cursor_context(filter: &SystemLogFilter, limit: u64) -> ObservabilityResult<SystemLogCursorContext> {
    Ok(SystemLogCursorContext {
        filter_fingerprint: filter_fingerprint(filter)?,
        limit,
    })
}

fn filter_fingerprint(filter: &SystemLogFilter) -> ObservabilityResult<String> {
    let mut levels = filter.levels.iter().map(|value| value.code()).collect::<Vec<_>>();
    levels.sort_unstable();
    levels.dedup();
    let value = (
        filter.keyword.as_deref(),
        levels,
        filter.target.as_deref(),
        filter.begin_time.map(OffsetDateTime::unix_timestamp_nanos),
        filter.end_time.map(OffsetDateTime::unix_timestamp_nanos),
    );
    cursor_fingerprint(&value).map_err(cursor_error)
}

fn parse_time(value: &str) -> ObservabilityResult<OffsetDateTime> {
    let nanos = value.parse::<i128>().map_err(|_| ObservabilityError::InvalidCursor)?;
    OffsetDateTime::from_unix_timestamp_nanos(nanos).map_err(|_| ObservabilityError::InvalidCursor)
}

fn cursor_error(error: impl std::fmt::Display) -> ObservabilityError {
    ObservabilityError::Infrastructure(format!("system log cursor error: {error}"))
}

#[cfg(test)]
mod tests {
    use crate::domain::{SystemLogFilter, SystemLogLevel};

    use super::{SystemLogSnapshot, filter_fingerprint};

    #[test]
    fn level_selection_order_does_not_change_cursor_context() {
        let first = SystemLogFilter {
            levels: vec![SystemLogLevel::Error, SystemLogLevel::Info],
            ..Default::default()
        };
        let second = SystemLogFilter {
            levels: vec![SystemLogLevel::Info, SystemLogLevel::Error, SystemLogLevel::Info],
            ..Default::default()
        };
        assert_eq!(filter_fingerprint(&first).unwrap(), filter_fingerprint(&second).unwrap());
    }

    #[test]
    fn snapshot_uses_the_persisted_ingestion_watermark() {
        let snapshot = SystemLogSnapshot::new(42);

        assert_eq!(snapshot.ingested_seq, 42);
    }
}
