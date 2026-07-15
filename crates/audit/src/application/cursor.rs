mod context;

use kernel::pagination::{CursorContext, CursorDirection, CursorPage, CursorPageRequest, decode_cursor};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::domain::{LoginLog, LoginLogFilter, LoginSortField, OperationLogFilter, OperationLogSummary, OperationSortField};

use self::context::{login_context, login_fingerprint, operation_context, operation_fingerprint, sort_key, validate_page};
use super::{AuditError, AuditResult};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct AuditSnapshot {
    pub ingested_at_nanos: String,
    pub id: String,
}

impl AuditSnapshot {
    pub fn new(ingested_at: OffsetDateTime, id: String) -> Self {
        Self {
            ingested_at_nanos: ingested_at.unix_timestamp_nanos().to_string(),
            id,
        }
    }

    pub fn ingested_at(&self) -> AuditResult<OffsetDateTime> {
        parse_timestamp(&self.ingested_at_nanos)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum OperationCursorValue {
    Time(String),
    SmallInt(i16),
    Text(String),
    BigInt(i64),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct OperationCursorBoundary {
    pub value: OperationCursorValue,
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum LoginCursorValue {
    Time(String),
    SmallInt(i16),
    Text(String),
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct LoginCursorBoundary {
    pub value: LoginCursorValue,
    pub id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditCursorQuery<B> {
    pub limit: u64,
    pub direction: CursorDirection,
    pub boundary: Option<B>,
    pub snapshot: Option<AuditSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditCursorSlice<T> {
    pub items: Vec<T>,
    pub snapshot: Option<AuditSnapshot>,
    pub has_next: bool,
    pub has_previous: bool,
}

pub fn operation_cursor_query(filter: &OperationLogFilter, page: &CursorPageRequest) -> AuditResult<AuditCursorQuery<OperationCursorBoundary>> {
    validate_page(page)?;
    let fingerprint = operation_fingerprint(filter)?;
    let sort = sort_key(filter.sort_field.code(), filter.sort_direction.code(), "oper_id");
    decode_query(page, operation_context(&sort, &fingerprint, page.limit))
}

pub fn login_cursor_query(filter: &LoginLogFilter, page: &CursorPageRequest) -> AuditResult<AuditCursorQuery<LoginCursorBoundary>> {
    validate_page(page)?;
    let fingerprint = login_fingerprint(filter)?;
    let sort = sort_key(filter.sort_field.code(), filter.sort_direction.code(), "info_id");
    decode_query(page, login_context(&sort, &fingerprint, page.limit))
}

pub fn operation_cursor_page(
    filter: &OperationLogFilter,
    query: &AuditCursorQuery<OperationCursorBoundary>,
    slice: AuditCursorSlice<OperationLogSummary>,
) -> AuditResult<CursorPage<OperationLogSummary>> {
    let fingerprint = operation_fingerprint(filter)?;
    let sort = sort_key(filter.sort_field.code(), filter.sort_direction.code(), "oper_id");
    encode_page(
        slice,
        EncodePageOptions {
            query,
            context: operation_context(&sort, &fingerprint, query.limit),
            boundary: |item: &OperationLogSummary| operation_boundary(item, filter.sort_field),
        },
    )
}

pub fn login_cursor_page(
    filter: &LoginLogFilter,
    query: &AuditCursorQuery<LoginCursorBoundary>,
    slice: AuditCursorSlice<LoginLog>,
) -> AuditResult<CursorPage<LoginLog>> {
    let fingerprint = login_fingerprint(filter)?;
    let sort = sort_key(filter.sort_field.code(), filter.sort_direction.code(), "info_id");
    encode_page(
        slice,
        EncodePageOptions {
            query,
            context: login_context(&sort, &fingerprint, query.limit),
            boundary: |item: &LoginLog| login_boundary(item, filter.sort_field),
        },
    )
}

pub fn operation_boundary(item: &OperationLogSummary, field: OperationSortField) -> OperationCursorBoundary {
    let value = match field {
        OperationSortField::OperationTime => OperationCursorValue::Time(item.operation_time.unix_timestamp_nanos().to_string()),
        OperationSortField::BusinessType => OperationCursorValue::SmallInt(item.business_type.code()),
        OperationSortField::Status => OperationCursorValue::SmallInt(item.status.code()),
        OperationSortField::OperatorName => OperationCursorValue::Text(item.operator_name.clone()),
        OperationSortField::CostTime => OperationCursorValue::BigInt(item.cost_time_ms),
    };
    OperationCursorBoundary { value, id: item.id.clone() }
}

pub fn login_boundary(item: &LoginLog, field: LoginSortField) -> LoginCursorBoundary {
    let value = match field {
        LoginSortField::LoginTime => LoginCursorValue::Time(item.login_time.unix_timestamp_nanos().to_string()),
        LoginSortField::Username => LoginCursorValue::Text(item.username.clone()),
        LoginSortField::IpAddress => LoginCursorValue::Text(item.ip_address.clone()),
        LoginSortField::Status => LoginCursorValue::SmallInt(item.status.code()),
    };
    LoginCursorBoundary { value, id: item.id.clone() }
}

fn decode_query<B>(page: &CursorPageRequest, context: CursorContext<'_>) -> AuditResult<AuditCursorQuery<B>>
where
    B: for<'de> Deserialize<'de>,
{
    let Some(value) = page.cursor.as_deref() else {
        return Ok(AuditCursorQuery {
            limit: page.limit,
            direction: CursorDirection::Next,
            boundary: None,
            snapshot: None,
        });
    };
    let decoded = decode_cursor::<B, AuditSnapshot>(value, &context).map_err(|_| AuditError::InvalidCursor)?;
    Ok(AuditCursorQuery {
        limit: page.limit,
        direction: decoded.direction,
        boundary: Some(decoded.boundary),
        snapshot: Some(decoded.snapshot),
    })
}

struct EncodePageOptions<'a, B, F> {
    query: &'a AuditCursorQuery<B>,
    context: CursorContext<'a>,
    boundary: F,
}

fn encode_page<T, B, F>(slice: AuditCursorSlice<T>, options: EncodePageOptions<'_, B, F>) -> AuditResult<CursorPage<T>>
where
    B: Serialize,
    F: Fn(&T) -> B,
{
    let EncodePageOptions { query, context, boundary } = options;
    if slice.items.is_empty() {
        return empty_page(query, slice.snapshot.as_ref(), &context);
    }
    let Some(snapshot) = slice.snapshot.as_ref() else {
        return Err(AuditError::Infrastructure("audit cursor page is missing its snapshot".into()));
    };
    let first = slice
        .items
        .first()
        .ok_or_else(|| AuditError::Infrastructure("audit cursor page lost its first row".into()))?;
    let last = slice
        .items
        .last()
        .ok_or_else(|| AuditError::Infrastructure("audit cursor page lost its last row".into()))?;
    let next = slice
        .has_next
        .then(|| context.encode(CursorDirection::Next, &boundary(last), snapshot))
        .transpose()
        .map_err(cursor_encode_error)?;
    let previous = slice
        .has_previous
        .then(|| context.encode(CursorDirection::Previous, &boundary(first), snapshot))
        .transpose()
        .map_err(cursor_encode_error)?;
    Ok(CursorPage::new(slice.items, next, previous))
}

fn empty_page<T, B>(query: &AuditCursorQuery<B>, snapshot: Option<&AuditSnapshot>, context: &CursorContext<'_>) -> AuditResult<CursorPage<T>>
where
    B: Serialize,
{
    let Some(boundary) = query.boundary.as_ref() else {
        return Ok(CursorPage::new(Vec::new(), None, None));
    };
    let snapshot = snapshot
        .or(query.snapshot.as_ref())
        .ok_or_else(|| AuditError::Infrastructure("empty audit cursor page is missing its snapshot".into()))?;
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

fn parse_timestamp(value: &str) -> AuditResult<OffsetDateTime> {
    let nanos = value.parse::<i128>().map_err(|_| AuditError::InvalidCursor)?;
    OffsetDateTime::from_unix_timestamp_nanos(nanos).map_err(|_| AuditError::InvalidCursor)
}

fn cursor_encode_error(error: impl std::fmt::Display) -> AuditError {
    AuditError::Infrastructure(format!("audit cursor encoding failed: {error}"))
}

#[cfg(test)]
#[path = "cursor_tests.rs"]
mod tests;
