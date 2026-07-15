use chrono::{DateTime, Utc};
use kernel::pagination::{CursorContext, CursorDirection, CursorPage, CursorPageRequest, cursor_fingerprint, decode_cursor};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::domain::{Job, JobListFilter, JobLogListFilter};

use super::{ExecutionLogSummary, SchedulerError, SchedulerResult, localized};

const JOB_RESOURCE: &str = "scheduler.jobs";
const LOG_RESOURCE: &str = "scheduler.execution_logs";
const GLOBAL_SCOPE: &str = "scheduler.global";
const JOB_SORT: &str = "create_time:desc,job_id:desc";
const LOG_SORT: &str = "create_time:desc,execution_id:desc";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct SchedulerCursorPoint {
    pub created_at_nanos: String,
    pub id: String,
}

impl SchedulerCursorPoint {
    pub fn new(created_at: DateTime<Utc>, id: String) -> Self {
        Self {
            created_at_nanos: datetime_nanos(created_at).to_string(),
            id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchedulerCursorQuery {
    pub limit: u64,
    pub direction: CursorDirection,
    pub boundary: Option<SchedulerCursorPoint>,
    pub snapshot: Option<SchedulerCursorPoint>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SchedulerCursorSlice<T> {
    pub items: Vec<T>,
    pub snapshot: Option<SchedulerCursorPoint>,
    pub has_next: bool,
    pub has_previous: bool,
}

impl<T> SchedulerCursorSlice<T> {
    pub fn map<U>(self, mapper: impl FnMut(T) -> U) -> SchedulerCursorSlice<U> {
        SchedulerCursorSlice {
            items: self.items.into_iter().map(mapper).collect(),
            snapshot: self.snapshot,
            has_next: self.has_next,
            has_previous: self.has_previous,
        }
    }
}

pub fn job_cursor_query(filter: &JobListFilter, page: &CursorPageRequest) -> SchedulerResult<SchedulerCursorQuery> {
    validate_page(page)?;
    decode_query(
        page,
        context(SchedulerContextSpec {
            resource: JOB_RESOURCE,
            sort: JOB_SORT,
            fingerprint: &job_fingerprint(filter)?,
            limit: page.limit,
        }),
    )
}

pub fn log_cursor_query(filter: &JobLogListFilter, page: &CursorPageRequest) -> SchedulerResult<SchedulerCursorQuery> {
    validate_page(page)?;
    decode_query(
        page,
        context(SchedulerContextSpec {
            resource: LOG_RESOURCE,
            sort: LOG_SORT,
            fingerprint: &log_fingerprint(filter)?,
            limit: page.limit,
        }),
    )
}

pub fn job_cursor_page(filter: &JobListFilter, query: &SchedulerCursorQuery, slice: SchedulerCursorSlice<Job>) -> SchedulerResult<CursorPage<Job>> {
    encode_page(
        slice,
        EncodePageOptions {
            query,
            context: context(SchedulerContextSpec {
                resource: JOB_RESOURCE,
                sort: JOB_SORT,
                fingerprint: &job_fingerprint(filter)?,
                limit: query.limit,
            }),
            point: job_point,
        },
    )
}

pub fn log_cursor_page(
    filter: &JobLogListFilter,
    query: &SchedulerCursorQuery,
    slice: SchedulerCursorSlice<ExecutionLogSummary>,
) -> SchedulerResult<CursorPage<ExecutionLogSummary>> {
    encode_page(
        slice,
        EncodePageOptions {
            query,
            context: context(SchedulerContextSpec {
                resource: LOG_RESOURCE,
                sort: LOG_SORT,
                fingerprint: &log_fingerprint(filter)?,
                limit: query.limit,
            }),
            point: log_point,
        },
    )
}

pub fn job_point(job: &Job) -> SchedulerCursorPoint {
    SchedulerCursorPoint::new(job.create_time, job.id.clone())
}

pub fn log_point(log: &ExecutionLogSummary) -> SchedulerCursorPoint {
    SchedulerCursorPoint::new(log.create_time, log.id.clone())
}

pub fn point_time(point: &SchedulerCursorPoint) -> SchedulerResult<DateTime<Utc>> {
    let nanos = point.created_at_nanos.parse::<i128>().map_err(|_| SchedulerError::InvalidCursor)?;
    let seconds = nanos.div_euclid(1_000_000_000);
    let subsecond = nanos.rem_euclid(1_000_000_000);
    let seconds = i64::try_from(seconds).map_err(|_| SchedulerError::InvalidCursor)?;
    let subsecond = u32::try_from(subsecond).map_err(|_| SchedulerError::InvalidCursor)?;
    DateTime::from_timestamp(seconds, subsecond).ok_or(SchedulerError::InvalidCursor)
}

fn decode_query(page: &CursorPageRequest, context: CursorContext<'_>) -> SchedulerResult<SchedulerCursorQuery> {
    let Some(value) = page.cursor.as_deref() else {
        return Ok(SchedulerCursorQuery {
            limit: page.limit,
            direction: CursorDirection::Next,
            boundary: None,
            snapshot: None,
        });
    };
    let decoded = decode_cursor::<SchedulerCursorPoint, SchedulerCursorPoint>(value, &context).map_err(|_| SchedulerError::InvalidCursor)?;
    Ok(SchedulerCursorQuery {
        limit: page.limit,
        direction: decoded.direction,
        boundary: Some(decoded.boundary),
        snapshot: Some(decoded.snapshot),
    })
}

struct EncodePageOptions<'a, F> {
    query: &'a SchedulerCursorQuery,
    context: CursorContext<'a>,
    point: F,
}

fn encode_page<T, F>(slice: SchedulerCursorSlice<T>, options: EncodePageOptions<'_, F>) -> SchedulerResult<CursorPage<T>>
where
    F: Fn(&T) -> SchedulerCursorPoint,
{
    let EncodePageOptions { query, context, point } = options;
    if slice.items.is_empty() {
        return empty_page(query, slice.snapshot.as_ref(), &context);
    }
    let snapshot = slice
        .snapshot
        .as_ref()
        .ok_or_else(|| SchedulerError::Infrastructure("scheduler cursor page is missing its snapshot".into()))?;
    let first = slice
        .items
        .first()
        .ok_or_else(|| SchedulerError::Infrastructure("scheduler cursor page lost its first row".into()))?;
    let last = slice
        .items
        .last()
        .ok_or_else(|| SchedulerError::Infrastructure("scheduler cursor page lost its last row".into()))?;
    let next = slice
        .has_next
        .then(|| context.encode(CursorDirection::Next, &point(last), snapshot))
        .transpose()
        .map_err(cursor_encode_error)?;
    let previous = slice
        .has_previous
        .then(|| context.encode(CursorDirection::Previous, &point(first), snapshot))
        .transpose()
        .map_err(cursor_encode_error)?;
    Ok(CursorPage::new(slice.items, next, previous))
}

fn empty_page<T>(query: &SchedulerCursorQuery, snapshot: Option<&SchedulerCursorPoint>, context: &CursorContext<'_>) -> SchedulerResult<CursorPage<T>> {
    let Some(boundary) = query.boundary.as_ref() else {
        return Ok(CursorPage::new(Vec::new(), None, None));
    };
    let snapshot = snapshot
        .or(query.snapshot.as_ref())
        .ok_or_else(|| SchedulerError::Infrastructure("empty scheduler cursor page is missing its snapshot".into()))?;
    let (next, previous) = match query.direction {
        CursorDirection::Next => (
            None,
            Some(context.encode(CursorDirection::Previous, boundary, snapshot).map_err(cursor_encode_error)?),
        ),
        CursorDirection::Previous => (
            Some(context.encode(CursorDirection::Next, boundary, snapshot).map_err(cursor_encode_error)?),
            None,
        ),
    };
    Ok(CursorPage::new(Vec::new(), next, previous))
}

fn job_fingerprint(filter: &JobListFilter) -> SchedulerResult<String> {
    fingerprint(&json!({
        "name": filter.name.as_deref(),
        "group": filter.group.as_deref(),
        "status": filter.status.map(|value| value.code()),
        "begin_time": filter.begin_time.map(|value| datetime_nanos(value).to_string()),
        "end_time": filter.end_time.map(|value| datetime_nanos(value).to_string()),
    }))
}

fn log_fingerprint(filter: &JobLogListFilter) -> SchedulerResult<String> {
    fingerprint(&json!({
        "name": filter.name.as_deref(),
        "group": filter.group.as_deref(),
        "outcome": filter.outcome.map(|value| value.code()),
        "trigger": filter.trigger.map(|value| value.code()),
        "begin_time": filter.begin_time.map(|value| datetime_nanos(value).to_string()),
        "end_time": filter.end_time.map(|value| datetime_nanos(value).to_string()),
    }))
}

fn fingerprint(value: &serde_json::Value) -> SchedulerResult<String> {
    cursor_fingerprint(value).map_err(cursor_encode_error)
}

struct SchedulerContextSpec<'a> {
    resource: &'a str,
    sort: &'a str,
    fingerprint: &'a str,
    limit: u64,
}

fn context(spec: SchedulerContextSpec<'_>) -> CursorContext<'_> {
    CursorContext {
        resource: spec.resource,
        sort: spec.sort,
        filter_fingerprint: spec.fingerprint,
        scope_fingerprint: GLOBAL_SCOPE,
        limit: spec.limit,
    }
}

fn validate_page(page: &CursorPageRequest) -> SchedulerResult<()> {
    page.validate().map_err(|_| {
        SchedulerError::InvalidInput(
            localized("errors.validation.cursor_limit_range")
                .with_param("min", kernel::pagination::MIN_CURSOR_LIMIT.to_string())
                .with_param("max", kernel::pagination::MAX_CURSOR_LIMIT.to_string()),
        )
    })
}

fn datetime_nanos(value: DateTime<Utc>) -> i128 {
    i128::from(value.timestamp()) * 1_000_000_000 + i128::from(value.timestamp_subsec_nanos())
}

fn cursor_encode_error(error: impl std::fmt::Display) -> SchedulerError {
    SchedulerError::Infrastructure(format!("scheduler cursor encoding failed: {error}"))
}

#[cfg(test)]
#[path = "cursor_tests.rs"]
mod tests;
