use kernel::pagination::{
    CursorContext, CursorDecodeError, CursorDirection, CursorEncodeError, CursorPageRequest, DecodedCursor, cursor_fingerprint, decode_cursor,
};
use rbac::domain::DataScopeFilter;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use time::OffsetDateTime;

use super::{AppError, AppResult, OnlineSessionPageRequest, UserListFilter};

const USER_RESOURCE: &str = "user.users";
const USER_SORT: &str = "create_time:asc,user_id:asc";
const ONLINE_RESOURCE: &str = "user.online_sessions";
const ONLINE_SORT: &str = "login_time:desc,token_id:asc";
const NANOS_PER_MILLISECOND: i128 = 1_000_000;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct UserCursorPoint {
    pub(crate) create_time_nanos: i64,
    pub(crate) user_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct OnlineCursorPoint {
    pub(crate) login_time_millis: i64,
    pub(crate) token_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct OnlineCursorSnapshot {
    pub(crate) as_of_millis: i64,
    pub(crate) head: OnlineCursorPoint,
}

pub(crate) type UserDecodedCursor = DecodedCursor<UserCursorPoint, UserCursorPoint>;
pub(crate) type OnlineDecodedCursor = DecodedCursor<OnlineCursorPoint, OnlineCursorSnapshot>;

pub(crate) struct UserCursorCodec {
    filter_fingerprint: String,
    scope_fingerprint: String,
    limit: u64,
}

pub(crate) struct OnlineCursorCodec {
    filter_fingerprint: String,
    scope_fingerprint: String,
    limit: u64,
}

impl UserCursorCodec {
    pub(crate) fn new(filter: &UserListFilter, scope: Option<&DataScopeFilter>) -> AppResult<Self> {
        Ok(Self {
            filter_fingerprint: fingerprint(user_filter_value(filter))?,
            scope_fingerprint: fingerprint(scope_value(scope))?,
            limit: filter.page.limit,
        })
    }

    pub(crate) fn decode(&self, request: &CursorPageRequest) -> AppResult<Option<UserDecodedCursor>> {
        decode_optional(request.cursor.as_deref(), &self.context())?
            .map(validate_user_cursor)
            .transpose()
    }

    pub(crate) fn encode(&self, direction: CursorDirection, boundary: &UserCursorPoint, snapshot: &UserCursorPoint) -> AppResult<String> {
        self.context().encode(direction, boundary, snapshot).map_err(cursor_encode_error)
    }

    fn context(&self) -> CursorContext<'_> {
        CursorContext {
            resource: USER_RESOURCE,
            sort: USER_SORT,
            filter_fingerprint: &self.filter_fingerprint,
            scope_fingerprint: &self.scope_fingerprint,
            limit: self.limit,
        }
    }
}

impl OnlineCursorCodec {
    pub(crate) fn new(request: &OnlineSessionPageRequest) -> AppResult<Self> {
        Ok(Self {
            filter_fingerprint: fingerprint(online_filter_value(request))?,
            scope_fingerprint: fingerprint(scope_value(request.scope.as_ref()))?,
            limit: request.page.limit,
        })
    }

    pub(crate) fn decode(&self, request: &CursorPageRequest) -> AppResult<Option<OnlineDecodedCursor>> {
        decode_optional(request.cursor.as_deref(), &self.context())?
            .map(validate_online_cursor)
            .transpose()
    }

    pub(crate) fn encode(&self, direction: CursorDirection, boundary: &OnlineCursorPoint, snapshot: &OnlineCursorSnapshot) -> AppResult<String> {
        self.context().encode(direction, boundary, snapshot).map_err(cursor_encode_error)
    }

    fn context(&self) -> CursorContext<'_> {
        CursorContext {
            resource: ONLINE_RESOURCE,
            sort: ONLINE_SORT,
            filter_fingerprint: &self.filter_fingerprint,
            scope_fingerprint: &self.scope_fingerprint,
            limit: self.limit,
        }
    }
}

pub(crate) fn timestamp_nanos(value: OffsetDateTime) -> AppResult<i64> {
    i64::try_from(value.unix_timestamp_nanos()).map_err(|error| AppError::Infrastructure(format!("cursor timestamp overflow: {error}")))
}

pub(crate) fn validate_cursor_request(request: &CursorPageRequest) -> AppResult<()> {
    request.validate().map_err(|_| {
        AppError::InvalidInput(
            kernel::error::LocalizedError::new("errors.validation.cursor_limit_range")
                .with_param("min", kernel::pagination::MIN_CURSOR_LIMIT.to_string())
                .with_param("max", kernel::pagination::MAX_CURSOR_LIMIT.to_string()),
        )
    })
}

pub(crate) fn timestamp_from_nanos(value: i64) -> AppResult<OffsetDateTime> {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(value)).map_err(|error| AppError::Infrastructure(format!("cursor timestamp is out of range: {error}")))
}

fn decode_optional<B, S>(value: Option<&str>, context: &CursorContext<'_>) -> AppResult<Option<DecodedCursor<B, S>>>
where
    B: for<'de> Deserialize<'de>,
    S: for<'de> Deserialize<'de>,
{
    value.map(|value| decode_cursor(value, context).map_err(cursor_decode_error)).transpose()
}

fn user_filter_value(filter: &UserListFilter) -> Value {
    json!({
        "username": filter.username,
        "nick_name": filter.nick_name,
        "phonenumber": filter.phonenumber,
        "email": filter.email,
        "sex": filter.sex,
        "status": filter.status,
        "dept_id": filter.dept_id,
        "dept_name": filter.dept_name,
        "post_ids": sorted(&filter.post_ids),
        "role_ids": sorted(&filter.role_ids),
        "begin_time": filter.begin_time.map(|value| value.unix_timestamp_nanos().to_string()),
        "end_time": filter.end_time.map(|value| value.unix_timestamp_nanos().to_string()),
    })
}

fn online_filter_value(request: &OnlineSessionPageRequest) -> Value {
    json!({
        "ipaddr": request.search.ipaddr,
        "user_name": request.search.user_name,
        "login_location": request.search.login_location,
        "browser": request.search.browser,
        "os": request.search.os,
        "begin_time": request.search.begin_time,
        "end_time": request.search.end_time,
    })
}

fn scope_value(scope: Option<&DataScopeFilter>) -> Value {
    match scope {
        None => json!({"mode": "unscoped"}),
        Some(scope) => json!({
            "data_scope": scope.data_scope.code(),
            "user_id": scope.user_id,
            "dept_id": scope.dept_id,
            "dept_ids": sorted(&scope.dept_ids),
        }),
    }
}

fn sorted(values: &[String]) -> Vec<&str> {
    let mut values = values.iter().map(String::as_str).collect::<Vec<_>>();
    values.sort_unstable();
    values
}

fn fingerprint(value: Value) -> AppResult<String> {
    cursor_fingerprint(&value).map_err(cursor_encode_error)
}

fn cursor_encode_error(error: CursorEncodeError) -> AppError {
    AppError::Infrastructure(error.to_string())
}

fn cursor_decode_error(_error: CursorDecodeError) -> AppError {
    AppError::InvalidCursor
}

fn validate_user_cursor(cursor: UserDecodedCursor) -> AppResult<UserDecodedCursor> {
    if !is_nonempty_id(&cursor.boundary.user_id) || !is_nonempty_id(&cursor.snapshot.user_id) {
        return Err(AppError::InvalidCursor);
    }
    Ok(cursor)
}

fn validate_online_cursor(cursor: OnlineDecodedCursor) -> AppResult<OnlineDecodedCursor> {
    let valid_ids = is_nonempty_id(&cursor.boundary.token_id) && is_nonempty_id(&cursor.snapshot.head.token_id);
    let valid_times =
        valid_millis(cursor.boundary.login_time_millis) && valid_millis(cursor.snapshot.as_of_millis) && valid_millis(cursor.snapshot.head.login_time_millis);
    if !valid_ids || !valid_times {
        return Err(AppError::InvalidCursor);
    }
    Ok(cursor)
}

fn valid_millis(value: i64) -> bool {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(value) * NANOS_PER_MILLISECOND).is_ok()
}

fn is_nonempty_id(value: &str) -> bool {
    !value.trim().is_empty()
}

#[cfg(test)]
#[path = "cursor_tests.rs"]
mod tests;
