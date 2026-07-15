use kernel::pagination::{
    CursorContext, CursorDecodeError, CursorDirection, CursorEncodeError, CursorPageRequest, DecodedCursor, cursor_fingerprint, decode_cursor,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use time::OffsetDateTime;

use crate::domain::DataScopeFilter;

use super::{MenuListFilter, RbacError, RbacResult, RoleListFilter, RoleUserListFilter};

const ROLE_RESOURCE: &str = "rbac.roles";
const ROLE_SORT: &str = "role_sort:asc,role_id:asc";
const MENU_RESOURCE: &str = "rbac.menus";
const MENU_SORT: &str = "parent_id:asc,order_num:asc,menu_id:asc";
const ROLE_USER_RESOURCE: &str = "rbac.role_users";
const ROLE_USER_SORT: &str = "create_time:asc,user_id:asc";
const NANOS_PER_MICRO: i128 = 1_000;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct TimeIdPoint {
    pub(crate) time_micros: i64,
    pub(crate) id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct RoleBoundary {
    pub(crate) role_sort: i64,
    pub(crate) role_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct MenuBoundary {
    pub(crate) parent_id: String,
    pub(crate) order_num: i64,
    pub(crate) menu_id: String,
}

pub(crate) type RoleCursor = DecodedCursor<RoleBoundary, TimeIdPoint>;
pub(crate) type MenuCursor = DecodedCursor<MenuBoundary, TimeIdPoint>;
pub(crate) type RoleUserCursor = DecodedCursor<TimeIdPoint, TimeIdPoint>;

pub(crate) struct RoleCursorCodec(CursorCodec);
pub(crate) struct MenuCursorCodec(CursorCodec);
pub(crate) struct RoleUserCursorCodec(CursorCodec);

struct CursorCodec {
    resource: &'static str,
    sort: &'static str,
    filter_fingerprint: String,
    scope_fingerprint: String,
    limit: u64,
}

impl RoleCursorCodec {
    pub(crate) fn new(filter: &RoleListFilter, scope: Option<&DataScopeFilter>) -> RbacResult<Self> {
        CursorCodec::new(CursorCodecSpec {
            resource: ROLE_RESOURCE,
            sort: ROLE_SORT,
            filter: role_filter_value(filter),
            scope,
            limit: filter.page.limit,
        })
        .map(Self)
    }

    pub(crate) fn decode(&self, request: &CursorPageRequest) -> RbacResult<Option<RoleCursor>> {
        self.0.decode(request)?.map(validate_role_cursor).transpose()
    }

    pub(crate) fn encode(&self, direction: CursorDirection, boundary: &RoleBoundary, snapshot: &TimeIdPoint) -> RbacResult<String> {
        self.0.encode(direction, boundary, snapshot)
    }
}

impl MenuCursorCodec {
    pub(crate) fn new(filter: &MenuListFilter) -> RbacResult<Self> {
        CursorCodec::new(CursorCodecSpec {
            resource: MENU_RESOURCE,
            sort: MENU_SORT,
            filter: menu_filter_value(filter),
            scope: None,
            limit: filter.page.limit,
        })
        .map(Self)
    }

    pub(crate) fn decode(&self, request: &CursorPageRequest) -> RbacResult<Option<MenuCursor>> {
        self.0.decode(request)?.map(validate_menu_cursor).transpose()
    }

    pub(crate) fn encode(&self, direction: CursorDirection, boundary: &MenuBoundary, snapshot: &TimeIdPoint) -> RbacResult<String> {
        self.0.encode(direction, boundary, snapshot)
    }
}

impl RoleUserCursorCodec {
    pub(crate) fn new(filter: &RoleUserListFilter, scope: Option<&DataScopeFilter>) -> RbacResult<Self> {
        CursorCodec::new(CursorCodecSpec {
            resource: ROLE_USER_RESOURCE,
            sort: ROLE_USER_SORT,
            filter: role_user_filter_value(filter),
            scope,
            limit: filter.page.limit,
        })
        .map(Self)
    }

    pub(crate) fn decode(&self, request: &CursorPageRequest) -> RbacResult<Option<RoleUserCursor>> {
        self.0.decode(request)?.map(validate_role_user_cursor).transpose()
    }

    pub(crate) fn encode(&self, direction: CursorDirection, boundary: &TimeIdPoint, snapshot: &TimeIdPoint) -> RbacResult<String> {
        self.0.encode(direction, boundary, snapshot)
    }
}

struct CursorCodecSpec<'a> {
    resource: &'static str,
    sort: &'static str,
    filter: Value,
    scope: Option<&'a DataScopeFilter>,
    limit: u64,
}

impl CursorCodec {
    fn new(spec: CursorCodecSpec<'_>) -> RbacResult<Self> {
        Ok(Self {
            resource: spec.resource,
            sort: spec.sort,
            filter_fingerprint: fingerprint(spec.filter)?,
            scope_fingerprint: fingerprint(scope_value(spec.scope))?,
            limit: spec.limit,
        })
    }

    fn decode<B, S>(&self, request: &CursorPageRequest) -> RbacResult<Option<DecodedCursor<B, S>>>
    where
        B: DeserializeOwned,
        S: DeserializeOwned,
    {
        request
            .cursor
            .as_deref()
            .map(|value| decode_cursor(value, &self.context()).map_err(cursor_decode_error))
            .transpose()
    }

    fn encode<B: Serialize, S: Serialize>(&self, direction: CursorDirection, boundary: &B, snapshot: &S) -> RbacResult<String> {
        self.context().encode(direction, boundary, snapshot).map_err(cursor_encode_error)
    }

    fn context(&self) -> CursorContext<'_> {
        CursorContext {
            resource: self.resource,
            sort: self.sort,
            filter_fingerprint: &self.filter_fingerprint,
            scope_fingerprint: &self.scope_fingerprint,
            limit: self.limit,
        }
    }
}

pub(crate) fn validate_cursor_request(request: &CursorPageRequest) -> RbacResult<()> {
    request.validate().map_err(|_| {
        RbacError::InvalidInput(
            kernel::error::LocalizedError::new("errors.validation.cursor_limit_range")
                .with_param("min", kernel::pagination::MIN_CURSOR_LIMIT.to_string())
                .with_param("max", kernel::pagination::MAX_CURSOR_LIMIT.to_string()),
        )
    })
}

pub(crate) fn point(time: OffsetDateTime, id: String) -> RbacResult<TimeIdPoint> {
    let micros = time.unix_timestamp_nanos().div_euclid(NANOS_PER_MICRO);
    Ok(TimeIdPoint {
        time_micros: i64::try_from(micros).map_err(numeric_error)?,
        id,
    })
}

pub(crate) fn point_time(point: &TimeIdPoint) -> RbacResult<OffsetDateTime> {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(point.time_micros) * NANOS_PER_MICRO).map_err(|_| RbacError::InvalidCursor)
}

fn validate_role_cursor(cursor: RoleCursor) -> RbacResult<RoleCursor> {
    if cursor.boundary.role_id.trim().is_empty() {
        return Err(RbacError::InvalidCursor);
    }
    validate_snapshot(&cursor.snapshot)?;
    Ok(cursor)
}

fn validate_menu_cursor(cursor: MenuCursor) -> RbacResult<MenuCursor> {
    if cursor.boundary.parent_id.trim().is_empty() || cursor.boundary.menu_id.trim().is_empty() {
        return Err(RbacError::InvalidCursor);
    }
    validate_snapshot(&cursor.snapshot)?;
    Ok(cursor)
}

fn validate_role_user_cursor(cursor: RoleUserCursor) -> RbacResult<RoleUserCursor> {
    validate_snapshot(&cursor.boundary)?;
    validate_snapshot(&cursor.snapshot)?;
    Ok(cursor)
}

fn validate_snapshot(point: &TimeIdPoint) -> RbacResult<()> {
    if point.id.trim().is_empty() {
        return Err(RbacError::InvalidCursor);
    }
    point_time(point).map(|_| ())
}

fn role_filter_value(filter: &RoleListFilter) -> Value {
    json!({
        "role_name": filter.role_name,
        "role_key": filter.role_key,
        "status": filter.status,
        "system": filter.system,
        "begin_time": filter.begin_time.map(timestamp_fingerprint),
        "end_time": filter.end_time.map(timestamp_fingerprint),
    })
}

fn menu_filter_value(filter: &MenuListFilter) -> Value {
    json!({
        "menu_name": filter.menu_name,
        "status": filter.status,
        "begin_time": filter.begin_time.map(timestamp_fingerprint),
        "end_time": filter.end_time.map(timestamp_fingerprint),
    })
}

fn role_user_filter_value(filter: &RoleUserListFilter) -> Value {
    json!({
        "role_id": filter.role_id,
        "username": filter.username,
        "phonenumber": filter.phonenumber,
        "allocated": filter.allocated,
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

fn timestamp_fingerprint(value: OffsetDateTime) -> String {
    value.unix_timestamp_nanos().to_string()
}

fn sorted(values: &[String]) -> Vec<&str> {
    let mut values = values.iter().map(String::as_str).collect::<Vec<_>>();
    values.sort_unstable();
    values
}

fn fingerprint(value: Value) -> RbacResult<String> {
    cursor_fingerprint(&value).map_err(cursor_encode_error)
}

fn cursor_encode_error(error: CursorEncodeError) -> RbacError {
    RbacError::Infrastructure(error.to_string())
}

fn cursor_decode_error(_error: CursorDecodeError) -> RbacError {
    RbacError::InvalidCursor
}

fn numeric_error(error: impl std::fmt::Display) -> RbacError {
    RbacError::Infrastructure(format!("RBAC cursor timestamp overflow: {error}"))
}

#[cfg(test)]
#[path = "cursor_tests.rs"]
mod tests;
