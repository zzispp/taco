use kernel::pagination::CursorDirection;
use sqlx::{AssertSqlSafe, PgConnection, Postgres, QueryBuilder, query_as};

use crate::{
    application::{SchedulerCursorPoint, SchedulerCursorQuery, SchedulerCursorSlice, SchedulerError, SchedulerResult, point_time},
    domain::ExecutionState,
};

use super::super::mapping::map_sqlx_error;

const JOB_SNAPSHOT_SOURCE: SnapshotSource = SnapshotSource {
    table: "sys_job",
    id_column: "job_id",
    state: None,
};
const LOG_SNAPSHOT_SOURCE: SnapshotSource = SnapshotSource {
    table: "sys_job_execution",
    id_column: "execution_id",
    state: Some(ExecutionState::Terminal.code()),
};

#[derive(Clone, Copy)]
struct SnapshotSource {
    table: &'static str,
    id_column: &'static str,
    state: Option<&'static str>,
}

pub(super) struct WindowSpec<'a> {
    pub(super) id_column: &'static str,
    pub(super) snapshot: &'a SchedulerCursorPoint,
    pub(super) page: &'a SchedulerCursorQuery,
}

pub(super) async fn resolve_job_snapshot(
    connection: &mut PgConnection,
    snapshot: Option<SchedulerCursorPoint>,
) -> SchedulerResult<Option<SchedulerCursorPoint>> {
    resolve_snapshot(connection, snapshot, JOB_SNAPSHOT_SOURCE).await
}

pub(super) async fn resolve_log_snapshot(
    connection: &mut PgConnection,
    snapshot: Option<SchedulerCursorPoint>,
) -> SchedulerResult<Option<SchedulerCursorPoint>> {
    resolve_snapshot(connection, snapshot, LOG_SNAPSHOT_SOURCE).await
}

async fn resolve_snapshot(
    connection: &mut PgConnection,
    snapshot: Option<SchedulerCursorPoint>,
    source: SnapshotSource,
) -> SchedulerResult<Option<SchedulerCursorPoint>> {
    if let Some(snapshot) = snapshot {
        point_time(&snapshot)?;
        return Ok(Some(snapshot));
    }
    let predicate = source.state.map(|state| format!(" WHERE state='{state}'")).unwrap_or_default();
    let sql = format!(
        "SELECT create_time,{0} FROM {1}{predicate} ORDER BY create_time DESC,{0} DESC LIMIT 1",
        source.id_column, source.table
    );
    query_as::<_, (chrono::DateTime<chrono::Utc>, String)>(AssertSqlSafe(sql))
        .fetch_optional(connection)
        .await
        .map_err(map_sqlx_error)
        .map(|row| row.map(|(created_at, id)| SchedulerCursorPoint::new(created_at, id)))
}

pub(super) fn push_window(builder: &mut QueryBuilder<Postgres>, spec: WindowSpec<'_>) -> SchedulerResult<()> {
    validate_point(spec.snapshot)?;
    push_snapshot_window(builder, &spec)?;
    push_boundary_window(builder, &spec)?;
    push_order(builder, &spec);
    Ok(())
}

fn validate_point(point: &SchedulerCursorPoint) -> SchedulerResult<()> {
    if point.id.is_empty() {
        return Err(SchedulerError::InvalidCursor);
    }
    Ok(())
}

fn push_snapshot_window(builder: &mut QueryBuilder<Postgres>, spec: &WindowSpec<'_>) -> SchedulerResult<()> {
    builder
        .push(" AND (create_time,")
        .push(spec.id_column)
        .push(") <= (")
        .push_bind(point_time(spec.snapshot)?)
        .push(",")
        .push_bind(spec.snapshot.id.clone())
        .push(")");
    Ok(())
}

fn push_boundary_window(builder: &mut QueryBuilder<Postgres>, spec: &WindowSpec<'_>) -> SchedulerResult<()> {
    let Some(boundary) = spec.page.boundary.as_ref() else {
        return Ok(());
    };
    validate_point(boundary)?;
    let comparison = if spec.page.direction == CursorDirection::Next { "<" } else { ">" };
    builder
        .push(" AND (create_time,")
        .push(spec.id_column)
        .push(") ")
        .push(comparison)
        .push(" (")
        .push_bind(point_time(boundary)?)
        .push(",")
        .push_bind(boundary.id.clone())
        .push(")");
    Ok(())
}

fn push_order(builder: &mut QueryBuilder<Postgres>, spec: &WindowSpec<'_>) {
    let order = if spec.page.direction == CursorDirection::Next { "DESC" } else { "ASC" };
    builder
        .push(" ORDER BY create_time ")
        .push(order)
        .push(",")
        .push(spec.id_column)
        .push(" ")
        .push(order);
}

pub(super) fn push_limit(builder: &mut QueryBuilder<Postgres>, limit: u64) -> SchedulerResult<()> {
    let query_limit = limit
        .checked_add(1)
        .and_then(|value| i64::try_from(value).ok())
        .ok_or_else(|| SchedulerError::Infrastructure("scheduler cursor limit overflow".into()))?;
    builder.push(" LIMIT ").push_bind(query_limit);
    Ok(())
}

pub(super) fn slice<T>(mut items: Vec<T>, snapshot: SchedulerCursorPoint, page: SchedulerCursorQuery) -> SchedulerResult<SchedulerCursorSlice<T>> {
    let limit = usize::try_from(page.limit).map_err(|error| SchedulerError::Infrastructure(format!("scheduler cursor limit conversion failed: {error}")))?;
    let has_extra = items.len() > limit;
    if has_extra {
        items.truncate(limit);
    }
    if page.direction == CursorDirection::Previous {
        items.reverse();
    }
    if items.is_empty() {
        return Ok(empty_slice_with_snapshot(snapshot));
    }
    let from_cursor = page.boundary.is_some();
    let (has_next, has_previous) = match page.direction {
        CursorDirection::Next => (has_extra, from_cursor),
        CursorDirection::Previous => (from_cursor, has_extra),
    };
    Ok(SchedulerCursorSlice {
        items,
        snapshot: Some(snapshot),
        has_next,
        has_previous,
    })
}

fn empty_slice_with_snapshot<T>(snapshot: SchedulerCursorPoint) -> SchedulerCursorSlice<T> {
    SchedulerCursorSlice {
        items: Vec::new(),
        snapshot: Some(snapshot),
        has_next: false,
        has_previous: false,
    }
}

pub(super) fn empty_slice<T>() -> SchedulerCursorSlice<T> {
    SchedulerCursorSlice {
        items: Vec::new(),
        snapshot: None,
        has_next: false,
        has_previous: false,
    }
}
