use kernel::pagination::{CursorDirection, CursorPage};
use rbac::domain::DataScopeFilter;
use sqlx::{AssertSqlSafe, query_as};
use storage::{StorageError, StorageResult, database::to_i64};
use time::OffsetDateTime;
use types::user::User;

use crate::application::{
    AppResult, UserListFilter,
    cursor::{UserCursorCodec, UserCursorPoint, UserDecodedCursor, timestamp_from_nanos, timestamp_nanos},
};
use crate::infra::user_repository::{filter_sql, mapping, record::UserRecord};

use super::UserQueries;

struct UserPageWindow {
    snapshot_time: OffsetDateTime,
    snapshot_id: String,
    boundary_time: Option<OffsetDateTime>,
    boundary_id: Option<String>,
    boundary: Option<UserCursorPoint>,
    direction: CursorDirection,
    limit: u64,
    from_cursor: bool,
}

#[derive(Clone, Copy)]
struct CursorPageBuild<'a> {
    codec: &'a UserCursorCodec,
    snapshot: &'a UserCursorPoint,
    window: &'a UserPageWindow,
}

#[derive(Clone, Copy)]
struct ScopedRecordsSpec<'a> {
    filter: &'a UserListFilter,
    scope: &'a DataScopeFilter,
    window: &'a UserPageWindow,
}

impl UserQueries {
    pub async fn list_page(&self, filter: UserListFilter, scope: Option<DataScopeFilter>) -> AppResult<CursorPage<User>> {
        let codec = UserCursorCodec::new(&filter, scope.as_ref())?;
        let decoded = codec.decode(&filter.page)?;
        let snapshot = self.resolve_snapshot(&filter, scope.as_ref(), decoded.as_ref()).await?;
        let Some(snapshot) = snapshot else {
            return Ok(CursorPage::new(Vec::new(), None, None));
        };
        let window = page_window(&filter, decoded.as_ref(), &snapshot)?;
        let records = self.fetch_cursor_records(&filter, scope.as_ref(), &window).await?;
        self.hydrate_cursor_page(
            records,
            CursorPageBuild {
                codec: &codec,
                snapshot: &snapshot,
                window: &window,
            },
        )
        .await
    }

    async fn resolve_snapshot(
        &self,
        filter: &UserListFilter,
        scope: Option<&DataScopeFilter>,
        decoded: Option<&UserDecodedCursor>,
    ) -> AppResult<Option<UserCursorPoint>> {
        if let Some(decoded) = decoded {
            return Ok(Some(decoded.snapshot.clone()));
        }
        let record = match scope {
            Some(scope) => scoped_snapshot(self, filter, scope).await,
            None => unscoped_snapshot(self, filter).await,
        }
        .map_err(mapping::storage_error)?;
        record.map(|record| record_point(&record)).transpose()
    }

    async fn fetch_cursor_records(&self, filter: &UserListFilter, scope: Option<&DataScopeFilter>, window: &UserPageWindow) -> AppResult<Vec<UserRecord>> {
        let records = match scope {
            Some(scope) => scoped_records(self, ScopedRecordsSpec { filter, scope, window }).await,
            None => unscoped_records(self, filter, window).await,
        };
        records.map_err(mapping::storage_error)
    }

    async fn hydrate_cursor_page(&self, mut records: Vec<UserRecord>, build: CursorPageBuild<'_>) -> AppResult<CursorPage<User>> {
        let requested = usize::try_from(build.window.limit).map_err(numeric_error)?;
        let has_extra = records.len() > requested;
        records.truncate(requested);
        if build.window.direction == CursorDirection::Previous {
            records.reverse();
        }
        let cursors = page_cursors(&records, build, has_extra)?;
        let items = self.users(records).await.map_err(mapping::storage_error)?;
        Ok(CursorPage::new(items, cursors.next, cursors.previous))
    }
}
struct PageCursors {
    next: Option<String>,
    previous: Option<String>,
}

fn page_window(filter: &UserListFilter, decoded: Option<&UserDecodedCursor>, snapshot: &UserCursorPoint) -> AppResult<UserPageWindow> {
    let boundary = decoded.map(|cursor| &cursor.boundary);
    Ok(UserPageWindow {
        snapshot_time: timestamp_from_nanos(snapshot.create_time_nanos)?,
        snapshot_id: snapshot.user_id.clone(),
        boundary_time: boundary.map(|point| timestamp_from_nanos(point.create_time_nanos)).transpose()?,
        boundary_id: boundary.map(|point| point.user_id.clone()),
        boundary: boundary.cloned(),
        direction: decoded.map_or(CursorDirection::Next, |cursor| cursor.direction),
        limit: filter.page.limit,
        from_cursor: decoded.is_some(),
    })
}

fn page_cursors(records: &[UserRecord], build: CursorPageBuild<'_>, has_extra: bool) -> AppResult<PageCursors> {
    let Some(first) = records.first() else {
        return empty_page_cursors(build.codec, build.snapshot, build.window);
    };
    let last = records.last().expect("a non-empty page has a last record");
    let has_previous = build.window.from_cursor && (build.window.direction == CursorDirection::Next || has_extra);
    let has_next = has_extra || (build.window.from_cursor && build.window.direction == CursorDirection::Previous);
    Ok(PageCursors {
        next: has_next
            .then(|| build.codec.encode(CursorDirection::Next, &record_point(last)?, build.snapshot))
            .transpose()?,
        previous: has_previous
            .then(|| build.codec.encode(CursorDirection::Previous, &record_point(first)?, build.snapshot))
            .transpose()?,
    })
}

fn empty_page_cursors(codec: &UserCursorCodec, snapshot: &UserCursorPoint, window: &UserPageWindow) -> AppResult<PageCursors> {
    let Some(boundary) = &window.boundary else {
        return Ok(PageCursors { next: None, previous: None });
    };
    match window.direction {
        CursorDirection::Next => Ok(PageCursors {
            next: None,
            previous: Some(codec.encode(CursorDirection::Previous, boundary, snapshot)?),
        }),
        CursorDirection::Previous => Ok(PageCursors {
            next: Some(codec.encode(CursorDirection::Next, boundary, snapshot)?),
            previous: None,
        }),
    }
}

fn record_point(record: &UserRecord) -> AppResult<UserCursorPoint> {
    Ok(UserCursorPoint {
        create_time_nanos: timestamp_nanos(record.create_time)?,
        user_id: record.user_id.clone(),
    })
}

async fn unscoped_snapshot(queries: &UserQueries, filter: &UserListFilter) -> StorageResult<Option<UserRecord>> {
    let sql = filter_sql::filtered_users("ORDER BY u.create_time DESC,u.user_id DESC LIMIT 1");
    bind_unscoped(query_as::<_, UserRecord>(AssertSqlSafe(sql)), filter)
        .fetch_optional(queries.database.pool())
        .await
        .map_err(StorageError::from)
}

async fn scoped_snapshot(queries: &UserQueries, filter: &UserListFilter, scope: &DataScopeFilter) -> StorageResult<Option<UserRecord>> {
    let sql = filter_sql::scoped_users("ORDER BY u.create_time DESC,u.user_id DESC LIMIT 1");
    bind_scoped(query_as::<_, UserRecord>(AssertSqlSafe(sql)), filter, scope)
        .fetch_optional(queries.database.pool())
        .await
        .map_err(StorageError::from)
}

async fn unscoped_records(queries: &UserQueries, filter: &UserListFilter, window: &UserPageWindow) -> StorageResult<Vec<UserRecord>> {
    let suffix = cursor_suffix(13, window.direction);
    let sql = filter_sql::filtered_users(&suffix);
    bind_unscoped(query_as::<_, UserRecord>(AssertSqlSafe(sql)), filter)
        .bind(window.snapshot_time)
        .bind(&window.snapshot_id)
        .bind(window.boundary_time)
        .bind(&window.boundary_id)
        .bind(to_i64(window.limit + 1)?)
        .fetch_all(queries.database.pool())
        .await
        .map_err(StorageError::from)
}

async fn scoped_records(queries: &UserQueries, spec: ScopedRecordsSpec<'_>) -> StorageResult<Vec<UserRecord>> {
    let suffix = cursor_suffix(17, spec.window.direction);
    let sql = filter_sql::scoped_users(&suffix);
    bind_scoped(query_as::<_, UserRecord>(AssertSqlSafe(sql)), spec.filter, spec.scope)
        .bind(spec.window.snapshot_time)
        .bind(&spec.window.snapshot_id)
        .bind(spec.window.boundary_time)
        .bind(&spec.window.boundary_id)
        .bind(to_i64(spec.window.limit + 1)?)
        .fetch_all(queries.database.pool())
        .await
        .map_err(StorageError::from)
}

fn cursor_suffix(first: usize, direction: CursorDirection) -> String {
    let snapshot_time = first;
    let snapshot_id = first + 1;
    let boundary_time = first + 2;
    let boundary_id = first + 3;
    let limit = first + 4;
    let (operator, order) = match direction {
        CursorDirection::Next => (">", "ASC"),
        CursorDirection::Previous => ("<", "DESC"),
    };
    format!(
        "AND (u.create_time,u.user_id) <= (${snapshot_time},${snapshot_id}) \
         AND (${boundary_time}::timestamptz IS NULL OR (u.create_time,u.user_id) {operator} (${boundary_time},${boundary_id})) \
         ORDER BY u.create_time {order},u.user_id {order} LIMIT ${limit}"
    )
}

fn bind_unscoped<'q>(
    query: sqlx::query::QueryAs<'q, sqlx::Postgres, UserRecord, sqlx::postgres::PgArguments>,
    filter: &'q UserListFilter,
) -> sqlx::query::QueryAs<'q, sqlx::Postgres, UserRecord, sqlx::postgres::PgArguments> {
    query
        .bind(&filter.username)
        .bind(&filter.phonenumber)
        .bind(&filter.status)
        .bind(&filter.dept_id)
        .bind(filter.begin_time)
        .bind(filter.end_time)
        .bind(&filter.nick_name)
        .bind(&filter.dept_name)
        .bind(&filter.email)
        .bind(&filter.sex)
        .bind(&filter.post_ids)
        .bind(&filter.role_ids)
}

fn bind_scoped<'q>(
    query: sqlx::query::QueryAs<'q, sqlx::Postgres, UserRecord, sqlx::postgres::PgArguments>,
    filter: &'q UserListFilter,
    scope: &'q DataScopeFilter,
) -> sqlx::query::QueryAs<'q, sqlx::Postgres, UserRecord, sqlx::postgres::PgArguments> {
    query
        .bind(scope.data_scope.code())
        .bind(&scope.user_id)
        .bind(&scope.dept_id)
        .bind(&scope.dept_ids)
        .bind(&filter.username)
        .bind(&filter.phonenumber)
        .bind(&filter.status)
        .bind(&filter.dept_id)
        .bind(filter.begin_time)
        .bind(filter.end_time)
        .bind(&filter.nick_name)
        .bind(&filter.dept_name)
        .bind(&filter.email)
        .bind(&filter.sex)
        .bind(&filter.post_ids)
        .bind(&filter.role_ids)
}

fn numeric_error(error: impl std::fmt::Display) -> crate::application::AppError {
    crate::application::AppError::Infrastructure(format!("user cursor numeric conversion failed: {error}"))
}

#[cfg(test)]
#[path = "cursor_page_tests.rs"]
mod tests;
