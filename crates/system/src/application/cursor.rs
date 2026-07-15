use kernel::pagination::{
    CursorContext, CursorDecodeError, CursorDirection, CursorEncodeError, CursorPageRequest, DecodedCursor, cursor_fingerprint, decode_cursor,
};
use rbac::domain::DataScopeFilter;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use time::OffsetDateTime;

use super::{ConfigListFilter, DeptListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter, SystemError, SystemResult};

const NANOS_PER_MICRO: i128 = 1_000;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SystemBoundary {
    Dept { parent_id: String, order_num: i64, dept_id: String },
    Post { post_sort: i64, post_id: String },
    DictType { dict_id: String },
    DictData { dict_sort: i64, dict_code: String },
    Config { config_id: String },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct TimeIdPoint {
    pub(crate) time_micros: i64,
    pub(crate) id: String,
}

pub(crate) type SystemDecodedCursor = DecodedCursor<SystemBoundary, TimeIdPoint>;

#[derive(Clone, Copy)]
enum BoundaryKind {
    Dept,
    Post,
    DictType,
    DictData,
    Config,
}

pub(crate) struct SystemCursorCodec {
    resource: &'static str,
    sort: &'static str,
    filter_fingerprint: String,
    scope_fingerprint: String,
    limit: u64,
    boundary_kind: BoundaryKind,
}

struct SystemCursorSpec {
    resource: &'static str,
    sort: &'static str,
    filter: Value,
    scope: Value,
    limit: u64,
    boundary_kind: BoundaryKind,
}

impl SystemCursorCodec {
    pub(crate) fn dept(filter: &DeptListFilter, scope: Option<&DataScopeFilter>) -> SystemResult<Self> {
        Self::new(SystemCursorSpec {
            resource: "system.depts",
            sort: "parent_id:asc,order_num:asc,dept_id:asc",
            filter: dept_filter(filter),
            scope: scope_value(scope),
            limit: filter.page.limit,
            boundary_kind: BoundaryKind::Dept,
        })
    }

    pub(crate) fn post(filter: &PostListFilter) -> SystemResult<Self> {
        Self::new(SystemCursorSpec {
            resource: "system.posts",
            sort: "post_sort:asc,post_id:asc",
            filter: post_filter(filter),
            scope: unscoped(),
            limit: filter.page.limit,
            boundary_kind: BoundaryKind::Post,
        })
    }

    pub(crate) fn dict_type(filter: &DictTypeListFilter) -> SystemResult<Self> {
        Self::new(SystemCursorSpec {
            resource: "system.dict_types",
            sort: "dict_id:asc",
            filter: dict_type_filter(filter),
            scope: unscoped(),
            limit: filter.page.limit,
            boundary_kind: BoundaryKind::DictType,
        })
    }

    pub(crate) fn dict_data(filter: &DictDataListFilter) -> SystemResult<Self> {
        Self::new(SystemCursorSpec {
            resource: "system.dict_data",
            sort: "dict_sort:asc,dict_code:asc",
            filter: dict_data_filter(filter),
            scope: unscoped(),
            limit: filter.page.limit,
            boundary_kind: BoundaryKind::DictData,
        })
    }

    pub(crate) fn config(filter: &ConfigListFilter) -> SystemResult<Self> {
        Self::new(SystemCursorSpec {
            resource: "system.configs",
            sort: "config_id:asc",
            filter: config_filter(filter),
            scope: unscoped(),
            limit: filter.page.limit,
            boundary_kind: BoundaryKind::Config,
        })
    }

    pub(crate) fn decode(&self, request: &CursorPageRequest) -> SystemResult<Option<SystemDecodedCursor>> {
        let cursor = request
            .cursor
            .as_deref()
            .map(|value| decode_cursor(value, &self.context()).map_err(cursor_decode_error))
            .transpose()?;
        cursor.map(|cursor| validate_cursor(cursor, self.boundary_kind)).transpose()
    }

    pub(crate) fn encode(&self, direction: CursorDirection, boundary: &SystemBoundary, snapshot: &TimeIdPoint) -> SystemResult<String> {
        self.context().encode(direction, boundary, snapshot).map_err(cursor_encode_error)
    }

    fn new(spec: SystemCursorSpec) -> SystemResult<Self> {
        Ok(Self {
            resource: spec.resource,
            sort: spec.sort,
            filter_fingerprint: fingerprint(spec.filter)?,
            scope_fingerprint: fingerprint(spec.scope)?,
            limit: spec.limit,
            boundary_kind: spec.boundary_kind,
        })
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

pub(crate) fn validate_cursor_request(request: &CursorPageRequest) -> SystemResult<()> {
    request.validate().map_err(|_| {
        SystemError::InvalidInput(
            kernel::error::LocalizedError::new("errors.validation.cursor_limit_range")
                .with_param("min", kernel::pagination::MIN_CURSOR_LIMIT.to_string())
                .with_param("max", kernel::pagination::MAX_CURSOR_LIMIT.to_string()),
        )
    })
}

pub(crate) fn point(time: OffsetDateTime, id: String) -> SystemResult<TimeIdPoint> {
    let micros = time.unix_timestamp_nanos().div_euclid(NANOS_PER_MICRO);
    Ok(TimeIdPoint {
        time_micros: i64::try_from(micros).map_err(numeric_error)?,
        id,
    })
}

pub(crate) fn point_time(point: &TimeIdPoint) -> SystemResult<OffsetDateTime> {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(point.time_micros) * NANOS_PER_MICRO).map_err(|_| SystemError::InvalidCursor)
}

fn validate_cursor(cursor: SystemDecodedCursor, kind: BoundaryKind) -> SystemResult<SystemDecodedCursor> {
    if !boundary_matches(&cursor.boundary, kind) || !boundary_is_valid(&cursor.boundary) || cursor.snapshot.id.trim().is_empty() {
        return Err(SystemError::InvalidCursor);
    }
    point_time(&cursor.snapshot)?;
    Ok(cursor)
}

fn boundary_matches(boundary: &SystemBoundary, kind: BoundaryKind) -> bool {
    matches!(
        (boundary, kind),
        (SystemBoundary::Dept { .. }, BoundaryKind::Dept)
            | (SystemBoundary::Post { .. }, BoundaryKind::Post)
            | (SystemBoundary::DictType { .. }, BoundaryKind::DictType)
            | (SystemBoundary::DictData { .. }, BoundaryKind::DictData)
            | (SystemBoundary::Config { .. }, BoundaryKind::Config)
    )
}

fn boundary_is_valid(boundary: &SystemBoundary) -> bool {
    match boundary {
        SystemBoundary::Dept { parent_id, dept_id, .. } => is_nonblank(parent_id) && is_nonblank(dept_id),
        SystemBoundary::Post { post_id, .. } => is_nonblank(post_id),
        SystemBoundary::DictType { dict_id } => is_nonblank(dict_id),
        SystemBoundary::DictData { dict_code, .. } => is_nonblank(dict_code),
        SystemBoundary::Config { config_id } => is_nonblank(config_id),
    }
}

fn is_nonblank(value: &str) -> bool {
    !value.trim().is_empty()
}

fn dept_filter(filter: &DeptListFilter) -> Value {
    json!({"dept_name":filter.dept_name,"leader":filter.leader,"phone":filter.phone,"email":filter.email,"status":filter.status,"begin_time":time_value(filter.begin_time),"end_time":time_value(filter.end_time)})
}

fn post_filter(filter: &PostListFilter) -> Value {
    json!({"post_code":filter.post_code,"post_name":filter.post_name,"status":filter.status,"remark":filter.remark,"begin_time":time_value(filter.begin_time),"end_time":time_value(filter.end_time)})
}

fn dict_type_filter(filter: &DictTypeListFilter) -> Value {
    json!({"dict_name":filter.dict_name,"dict_type":filter.dict_type,"status":filter.status,"begin_time":time_value(filter.begin_time),"end_time":time_value(filter.end_time)})
}

fn dict_data_filter(filter: &DictDataListFilter) -> Value {
    json!({"dict_type":filter.dict_type,"dict_label":filter.dict_label,"status":filter.status,"begin_time":time_value(filter.begin_time),"end_time":time_value(filter.end_time)})
}

fn config_filter(filter: &ConfigListFilter) -> Value {
    json!({"config_name":filter.config_name,"config_key":filter.config_key,"config_type":filter.config_type,"public_read":filter.public_read,"begin_time":time_value(filter.begin_time),"end_time":time_value(filter.end_time)})
}

fn scope_value(scope: Option<&DataScopeFilter>) -> Value {
    match scope {
        None => unscoped(),
        Some(scope) => json!({"data_scope":scope.data_scope.code(),"user_id":scope.user_id,"dept_id":scope.dept_id,"dept_ids":sorted(&scope.dept_ids)}),
    }
}

fn unscoped() -> Value {
    json!({"mode":"unscoped"})
}

fn time_value(value: Option<OffsetDateTime>) -> Option<String> {
    value.map(|value| value.unix_timestamp_nanos().to_string())
}

fn sorted(values: &[String]) -> Vec<&str> {
    let mut values = values.iter().map(String::as_str).collect::<Vec<_>>();
    values.sort_unstable();
    values
}

fn fingerprint(value: Value) -> SystemResult<String> {
    cursor_fingerprint(&value).map_err(cursor_encode_error)
}

fn cursor_encode_error(error: CursorEncodeError) -> SystemError {
    SystemError::Infrastructure(error.to_string())
}

fn cursor_decode_error(_error: CursorDecodeError) -> SystemError {
    SystemError::InvalidCursor
}

fn numeric_error(error: impl std::fmt::Display) -> SystemError {
    SystemError::Infrastructure(format!("system cursor timestamp overflow: {error}"))
}
