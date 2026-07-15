use kernel::pagination::{CursorDirection, CursorPage};
use rbac::domain::{DataScope, DataScopeFilter};
use sqlx::{Postgres, QueryBuilder};
use storage::database::to_i64;
use time::OffsetDateTime;

use crate::application::{
    AppError, AppResult, OnlineSession, OnlineSessionPageRequest,
    cursor::{OnlineCursorCodec, OnlineCursorPoint, OnlineCursorSnapshot},
};

use super::{
    ACTIVE_USER_STATUS, NOT_DELETED_FLAG, OnlineSessionRecord, StorageOnlineSessionStore, from_epoch_millis, storage_error, storage_mapping_error,
    to_epoch_millis,
};

struct OnlinePageWindow {
    as_of: OffsetDateTime,
    head_time: OffsetDateTime,
    head_id: String,
    boundary_time: Option<OffsetDateTime>,
    boundary_id: Option<String>,
    boundary: Option<OnlineCursorPoint>,
    direction: CursorDirection,
    limit: u64,
    from_cursor: bool,
}

#[derive(Clone, Copy)]
struct OnlinePageBuild<'a> {
    codec: &'a OnlineCursorCodec,
    snapshot: &'a OnlineCursorSnapshot,
    window: &'a OnlinePageWindow,
}

#[derive(Clone, Copy)]
struct OnlineCursorBuild<'a> {
    page: OnlinePageBuild<'a>,
    has_extra: bool,
}

pub(super) async fn page_active(store: &StorageOnlineSessionStore, request: OnlineSessionPageRequest) -> AppResult<CursorPage<OnlineSession>> {
    crate::application::cursor::validate_cursor_request(&request.page)?;
    let codec = OnlineCursorCodec::new(&request)?;
    let decoded = codec.decode(&request.page)?;
    let snapshot = resolve_snapshot(store, &request, decoded.as_ref().map(|value| &value.snapshot)).await?;
    let Some(snapshot) = snapshot else {
        return Ok(CursorPage::new(Vec::new(), None, None));
    };
    let window = page_window(&request, decoded.as_ref(), &snapshot)?;
    let records = fetch_records(store, &request, &window).await?;
    build_page(
        records,
        OnlinePageBuild {
            codec: &codec,
            snapshot: &snapshot,
            window: &window,
        },
    )
}

async fn resolve_snapshot(
    store: &StorageOnlineSessionStore,
    request: &OnlineSessionPageRequest,
    decoded: Option<&OnlineCursorSnapshot>,
) -> AppResult<Option<OnlineCursorSnapshot>> {
    if let Some(snapshot) = decoded {
        return Ok(Some(snapshot.clone()));
    }
    let as_of_millis = to_epoch_millis(OffsetDateTime::now_utc())?;
    let as_of = from_epoch_millis(as_of_millis)?;
    let mut query = active_query(request, as_of)?;
    query.push(" ORDER BY session.login_time DESC,session.token_id ASC LIMIT 1");
    let record = query
        .build_query_as::<OnlineSessionRecord>()
        .fetch_optional(store.database.pool())
        .await
        .map_err(storage_error)?;
    record
        .map(|record| {
            Ok(OnlineCursorSnapshot {
                as_of_millis,
                head: record_point(&record)?,
            })
        })
        .transpose()
}

async fn fetch_records(
    store: &StorageOnlineSessionStore,
    request: &OnlineSessionPageRequest,
    window: &OnlinePageWindow,
) -> AppResult<Vec<OnlineSessionRecord>> {
    let mut query = active_query(request, window.as_of)?;
    push_snapshot_bound(&mut query, window);
    push_cursor_bound(&mut query, window);
    push_order(&mut query, window.direction);
    query.push(" LIMIT ").push_bind(to_i64(window.limit + 1).map_err(storage_mapping_error)?);
    query
        .build_query_as::<OnlineSessionRecord>()
        .fetch_all(store.database.pool())
        .await
        .map_err(storage_error)
}

fn build_page(mut records: Vec<OnlineSessionRecord>, page: OnlinePageBuild<'_>) -> AppResult<CursorPage<OnlineSession>> {
    let requested = usize::try_from(page.window.limit).map_err(numeric_error)?;
    let has_extra = records.len() > requested;
    records.truncate(requested);
    if page.window.direction == CursorDirection::Previous {
        records.reverse();
    }
    let (next, previous) = page_cursors(&records, OnlineCursorBuild { page, has_extra })?;
    let items = records.into_iter().map(OnlineSession::try_from).collect::<AppResult<Vec<_>>>()?;
    Ok(CursorPage::new(items, next, previous))
}

fn page_cursors(records: &[OnlineSessionRecord], context: OnlineCursorBuild<'_>) -> AppResult<(Option<String>, Option<String>)> {
    let OnlineCursorBuild { page, has_extra } = context;
    let Some(first) = records.first() else {
        return empty_page_cursors(page.codec, page.snapshot, page.window);
    };
    let last = records.last().expect("a non-empty page has a last record");
    let has_previous = page.window.from_cursor && (page.window.direction == CursorDirection::Next || has_extra);
    let has_next = has_extra || (page.window.from_cursor && page.window.direction == CursorDirection::Previous);
    let next = has_next
        .then(|| page.codec.encode(CursorDirection::Next, &record_point(last)?, page.snapshot))
        .transpose()?;
    let previous = has_previous
        .then(|| page.codec.encode(CursorDirection::Previous, &record_point(first)?, page.snapshot))
        .transpose()?;
    Ok((next, previous))
}

fn empty_page_cursors(codec: &OnlineCursorCodec, snapshot: &OnlineCursorSnapshot, window: &OnlinePageWindow) -> AppResult<(Option<String>, Option<String>)> {
    let Some(boundary) = &window.boundary else { return Ok((None, None)) };
    match window.direction {
        CursorDirection::Next => Ok((None, Some(codec.encode(CursorDirection::Previous, boundary, snapshot)?))),
        CursorDirection::Previous => Ok((Some(codec.encode(CursorDirection::Next, boundary, snapshot)?), None)),
    }
}

fn page_window(
    request: &OnlineSessionPageRequest,
    decoded: Option<&crate::application::cursor::OnlineDecodedCursor>,
    snapshot: &OnlineCursorSnapshot,
) -> AppResult<OnlinePageWindow> {
    let boundary = decoded.map(|value| &value.boundary);
    Ok(OnlinePageWindow {
        as_of: from_epoch_millis(snapshot.as_of_millis)?,
        head_time: from_epoch_millis(snapshot.head.login_time_millis)?,
        head_id: snapshot.head.token_id.clone(),
        boundary_time: boundary.map(|value| from_epoch_millis(value.login_time_millis)).transpose()?,
        boundary_id: boundary.map(|value| value.token_id.clone()),
        boundary: boundary.cloned(),
        direction: decoded.map_or(CursorDirection::Next, |value| value.direction),
        limit: request.page.limit,
        from_cursor: decoded.is_some(),
    })
}

fn active_query(request: &OnlineSessionPageRequest, as_of: OffsetDateTime) -> AppResult<QueryBuilder<Postgres>> {
    let mut query = QueryBuilder::new(active_session_select());
    query.push("FROM sys_user_session session JOIN sys_user users ON users.user_id=session.user_id ");
    query.push("WHERE session.expires_at>").push_bind(as_of);
    query.push(" AND session.login_time<=").push_bind(as_of);
    query.push(" AND users.status=").push_bind(ACTIVE_USER_STATUS);
    query.push(" AND users.del_flag=").push_bind(NOT_DELETED_FLAG);
    push_text_filters(&mut query, request);
    push_time_filters(&mut query, request)?;
    push_scope_filter(&mut query, request.scope.as_ref());
    Ok(query)
}

fn push_text_filters(query: &mut QueryBuilder<Postgres>, request: &OnlineSessionPageRequest) {
    let fields = [
        ("session.ipaddr", &request.search.ipaddr),
        ("session.user_name", &request.search.user_name),
        ("session.login_location", &request.search.login_location),
        ("session.browser", &request.search.browser),
        ("session.os", &request.search.os),
    ];
    for (column, value) in fields {
        if let Some(value) = value {
            query.push(" AND ").push(column).push(" ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
        }
    }
}

fn push_time_filters(query: &mut QueryBuilder<Postgres>, request: &OnlineSessionPageRequest) -> AppResult<()> {
    if let Some(begin) = request.search.begin_time {
        query.push(" AND session.login_time>=").push_bind(from_epoch_millis(begin)?);
    }
    if let Some(end) = request.search.end_time {
        query.push(" AND session.login_time<=").push_bind(from_epoch_millis(end)?);
    }
    Ok(())
}

fn push_scope_filter(query: &mut QueryBuilder<Postgres>, scope: Option<&DataScopeFilter>) {
    let Some(scope) = scope else { return };
    match scope.data_scope {
        DataScope::All => {}
        DataScope::Custom => {
            query.push(" AND users.dept_id=ANY(").push_bind(scope.dept_ids.clone()).push(")");
        }
        DataScope::Department => {
            query.push(" AND users.dept_id=").push_bind(scope.dept_id.clone());
        }
        DataScope::DepartmentAndChildren => push_department_tree_scope(query, scope.dept_id.clone()),
        DataScope::SelfOnly => {
            query.push(" AND users.user_id=").push_bind(scope.user_id.clone());
        }
    }
}

fn push_department_tree_scope(query: &mut QueryBuilder<Postgres>, dept_id: Option<String>) {
    query.push(" AND EXISTS (SELECT 1 FROM sys_dept d WHERE d.dept_id=users.dept_id AND d.del_flag='0' AND (d.dept_id=");
    query.push_bind(dept_id.clone()).push(" OR (',' || d.ancestors || ',') LIKE '%,' || ");
    query.push_bind(dept_id).push(" || ',%'))");
}

fn push_snapshot_bound(query: &mut QueryBuilder<Postgres>, window: &OnlinePageWindow) {
    query.push(" AND (session.login_time<").push_bind(window.head_time);
    query.push(" OR (session.login_time=").push_bind(window.head_time);
    query.push(" AND session.token_id>=").push_bind(window.head_id.clone()).push("))");
}

fn push_cursor_bound(query: &mut QueryBuilder<Postgres>, window: &OnlinePageWindow) {
    let (Some(time), Some(token_id)) = (window.boundary_time, window.boundary_id.clone()) else {
        return;
    };
    let (time_operator, id_operator) = match window.direction {
        CursorDirection::Next => ("<", ">"),
        CursorDirection::Previous => (">", "<"),
    };
    query.push(" AND (session.login_time").push(time_operator).push_bind(time);
    query.push(" OR (session.login_time=").push_bind(time);
    query.push(" AND session.token_id").push(id_operator).push_bind(token_id).push("))");
}

fn push_order(query: &mut QueryBuilder<Postgres>, direction: CursorDirection) {
    match direction {
        CursorDirection::Next => query.push(" ORDER BY session.login_time DESC,session.token_id ASC"),
        CursorDirection::Previous => query.push(" ORDER BY session.login_time ASC,session.token_id DESC"),
    };
}

fn record_point(record: &OnlineSessionRecord) -> AppResult<OnlineCursorPoint> {
    Ok(OnlineCursorPoint {
        login_time_millis: to_epoch_millis(record.login_time)?,
        token_id: record.token_id.clone(),
    })
}

fn active_session_select() -> &'static str {
    "SELECT session.token_id,session.user_id,users.dept_id,session.dept_name,session.user_name,session.ipaddr,\
     session.login_location,session.browser,session.os,session.login_time,session.expires_at "
}

fn numeric_error(error: impl std::fmt::Display) -> AppError {
    AppError::Infrastructure(format!("online session cursor numeric conversion failed: {error}"))
}

#[cfg(test)]
#[path = "pagination_tests.rs"]
mod tests;
