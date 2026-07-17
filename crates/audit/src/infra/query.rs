mod keyset;

use sqlx::{AssertSqlSafe, PgConnection, Postgres, QueryBuilder};
use storage::ObservedPgPool;
use time::OffsetDateTime;

use crate::{
    application::{AuditCursorQuery, AuditCursorSlice, AuditResult, AuditSnapshot, LoginCursorBoundary, OperationCursorBoundary},
    domain::{LoginLog, LoginLogFilter, OperationLogDetail, OperationLogFilter, OperationLogSummary},
};

use super::{
    mapping,
    records::{LoginRecord, OperationDetailRecord, OperationSummaryRecord},
};

const OPERATION_COLUMNS: &str = "oper_id,title,business_type,method,request_method,operator_type,oper_name,dept_name,oper_url,oper_ip,oper_location_kind,oper_location,status,oper_time,cost_time";
const LOGIN_COLUMNS: &str =
    "info_id,user_id,user_name,ipaddr,login_location_kind,login_location,browser,os,status,event_type,message_key,message_params,login_time";
const OPERATION_SNAPSHOT_SOURCE: SnapshotSource = SnapshotSource {
    table: "sys_oper_log",
    id_column: "oper_id",
};
const LOGIN_SNAPSHOT_SOURCE: SnapshotSource = SnapshotSource {
    table: "sys_logininfor",
    id_column: "info_id",
};

pub async fn page_operations(
    pool: ObservedPgPool,
    filter: OperationLogFilter,
    page: AuditCursorQuery<OperationCursorBoundary>,
) -> AuditResult<AuditCursorSlice<OperationLogSummary>> {
    let mut connection = pool.acquire().await.map_err(mapping::sqlx_error)?;
    page_operations_on(&mut connection, filter, page).await
}

pub(super) async fn page_operations_on(
    connection: &mut PgConnection,
    filter: OperationLogFilter,
    page: AuditCursorQuery<OperationCursorBoundary>,
) -> AuditResult<AuditCursorSlice<OperationLogSummary>> {
    let snapshot = operation_snapshot(connection, page.snapshot.clone()).await?;
    let Some(snapshot) = snapshot else {
        return Ok(empty_slice());
    };
    let mut builder = QueryBuilder::<Postgres>::new(format!("SELECT {OPERATION_COLUMNS} FROM sys_oper_log WHERE TRUE"));
    push_operation_filters(&mut builder, &filter);
    keyset::push_snapshot(&mut builder, "oper_id", &snapshot)?;
    keyset::push_operation_boundary(&mut builder, &filter, &page)?;
    keyset::push_operation_order(&mut builder, &filter, page.direction);
    keyset::push_limit(&mut builder, page.limit)?;
    let records = builder
        .build_query_as::<OperationSummaryRecord>()
        .fetch_all(&mut *connection)
        .await
        .map_err(mapping::sqlx_error)?;
    let items = records.into_iter().map(mapping::operation_summary).collect::<AuditResult<Vec<_>>>()?;
    keyset::slice(items, snapshot, page)
}

pub async fn find_operation(pool: ObservedPgPool, id: &str) -> AuditResult<Option<OperationLogDetail>> {
    let sql = format!("SELECT {OPERATION_COLUMNS},request_id,operator_id,dept_id,oper_param,json_result,error_msg FROM sys_oper_log WHERE oper_id=$1");
    sqlx::query_as::<_, OperationDetailRecord>(AssertSqlSafe(sql))
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(mapping::sqlx_error)?
        .map(mapping::operation_detail)
        .transpose()
}

pub async fn page_logins(pool: ObservedPgPool, filter: LoginLogFilter, page: AuditCursorQuery<LoginCursorBoundary>) -> AuditResult<AuditCursorSlice<LoginLog>> {
    let mut connection = pool.acquire().await.map_err(mapping::sqlx_error)?;
    page_logins_on(&mut connection, filter, page).await
}

pub(super) async fn page_logins_on(
    connection: &mut PgConnection,
    filter: LoginLogFilter,
    page: AuditCursorQuery<LoginCursorBoundary>,
) -> AuditResult<AuditCursorSlice<LoginLog>> {
    let snapshot = login_snapshot(connection, page.snapshot.clone()).await?;
    let Some(snapshot) = snapshot else {
        return Ok(empty_slice());
    };
    let mut builder = QueryBuilder::<Postgres>::new(format!("SELECT {LOGIN_COLUMNS} FROM sys_logininfor WHERE TRUE"));
    push_login_filters(&mut builder, &filter);
    keyset::push_snapshot(&mut builder, "info_id", &snapshot)?;
    keyset::push_login_boundary(&mut builder, &filter, &page)?;
    keyset::push_login_order(&mut builder, &filter, page.direction);
    keyset::push_limit(&mut builder, page.limit)?;
    let records = builder
        .build_query_as::<LoginRecord>()
        .fetch_all(&mut *connection)
        .await
        .map_err(mapping::sqlx_error)?;
    let items = records.into_iter().map(mapping::login).collect::<AuditResult<Vec<_>>>()?;
    keyset::slice(items, snapshot, page)
}

async fn operation_snapshot(connection: &mut PgConnection, snapshot: Option<AuditSnapshot>) -> AuditResult<Option<AuditSnapshot>> {
    resolve_snapshot(connection, snapshot, OPERATION_SNAPSHOT_SOURCE).await
}

async fn login_snapshot(connection: &mut PgConnection, snapshot: Option<AuditSnapshot>) -> AuditResult<Option<AuditSnapshot>> {
    resolve_snapshot(connection, snapshot, LOGIN_SNAPSHOT_SOURCE).await
}

#[derive(Clone, Copy)]
struct SnapshotSource {
    table: &'static str,
    id_column: &'static str,
}

async fn resolve_snapshot(connection: &mut PgConnection, snapshot: Option<AuditSnapshot>, source: SnapshotSource) -> AuditResult<Option<AuditSnapshot>> {
    if let Some(snapshot) = snapshot {
        snapshot.ingested_at()?;
        return Ok(Some(snapshot));
    }
    let sql = format!(
        "SELECT ingested_at,{0} FROM {1} ORDER BY ingested_at DESC,{0} DESC LIMIT 1",
        source.id_column, source.table
    );
    sqlx::query_as::<_, (OffsetDateTime, String)>(AssertSqlSafe(sql))
        .fetch_optional(connection)
        .await
        .map_err(mapping::sqlx_error)
        .map(|row| row.map(|(ingested_at, id)| AuditSnapshot::new(ingested_at, id)))
}

fn push_operation_filters(builder: &mut QueryBuilder<Postgres>, filter: &OperationLogFilter) {
    push_title(builder, filter);
    push_business_types(builder, filter);
    if let Some(status) = filter.status {
        builder.push(" AND status=").push_bind(status.code());
    }
    push_like(builder, "oper_name", filter.operator_name.as_deref());
    push_like(builder, "oper_ip", filter.operation_ip.as_deref());
    push_time_range(
        builder,
        "oper_time",
        TimeRange {
            begin: filter.begin_time,
            end: filter.end_time,
        },
    );
}

fn push_login_filters(builder: &mut QueryBuilder<Postgres>, filter: &LoginLogFilter) {
    push_like(builder, "user_name", filter.username.as_deref());
    push_like(builder, "ipaddr", filter.ip_address.as_deref());
    if let Some(status) = filter.status {
        builder.push(" AND status=").push_bind(status.code());
    }
    if let Some(event_type) = filter.event_type {
        builder.push(" AND event_type=").push_bind(event_type.code());
    }
    push_time_range(
        builder,
        "login_time",
        TimeRange {
            begin: filter.begin_time,
            end: filter.end_time,
        },
    );
}

fn push_business_types(builder: &mut QueryBuilder<Postgres>, filter: &OperationLogFilter) {
    if filter.business_types.is_empty() {
        return;
    }
    builder.push(" AND business_type IN (");
    let mut values = builder.separated(",");
    for value in &filter.business_types {
        values.push_bind(value.code());
    }
    values.push_unseparated(")");
}

fn push_title(builder: &mut QueryBuilder<Postgres>, filter: &OperationLogFilter) {
    let Some(title) = filter.title.as_deref() else { return };
    builder.push(" AND (title ILIKE ").push_bind(like_pattern(title)).push(" ESCAPE '\\'");
    if !filter.title_keys.is_empty() {
        builder.push(" OR title=ANY(").push_bind(filter.title_keys.clone()).push(")");
    }
    builder.push(")");
}

fn push_like(builder: &mut QueryBuilder<Postgres>, column: &'static str, value: Option<&str>) {
    let Some(value) = value else { return };
    builder
        .push(" AND ")
        .push(column)
        .push(" ILIKE ")
        .push_bind(like_pattern(value))
        .push(" ESCAPE '\\'");
}

fn like_pattern(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
    format!("%{escaped}%")
}

struct TimeRange {
    begin: Option<OffsetDateTime>,
    end: Option<OffsetDateTime>,
}

fn push_time_range(builder: &mut QueryBuilder<Postgres>, column: &'static str, range: TimeRange) {
    if let Some(begin) = range.begin {
        builder.push(" AND ").push(column).push(">=").push_bind(begin);
    }
    if let Some(end) = range.end {
        builder.push(" AND ").push(column).push("<=").push_bind(end);
    }
}

fn empty_slice<T>() -> AuditCursorSlice<T> {
    AuditCursorSlice {
        items: Vec::new(),
        snapshot: None,
        has_next: false,
        has_previous: false,
    }
}

#[cfg(test)]
mod tests {
    use super::like_pattern;

    #[test]
    fn contains_pattern_escapes_sql_wildcards() {
        assert_eq!(like_pattern(r"a%b_c\d"), r"%a\%b\_c\\d%");
    }
}
