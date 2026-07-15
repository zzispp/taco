use kernel::pagination::{CursorDirection, CursorPage};
use sqlx::{AssertSqlSafe, Postgres, QueryBuilder, query, query_as, query_scalar};
use storage::{Database, StorageError, StorageResult};
use time::OffsetDateTime;

use crate::{
    application::{PostListFilter, SystemBoundary, SystemCursorCodec, SystemResult, TimeIdPoint, point_time},
    domain::{Post, PostInput},
};

use super::{
    cursor_page::{CursorRecord, PageBuildContext, PageNavigation, build_page, navigation},
    mapping::{post, storage_error},
    record::PostRecord,
};

pub(super) const COLUMNS: &str = "post_id,post_code,post_name,post_sort,status,remark,create_time";

struct PostPageWindow {
    snapshot_time: OffsetDateTime,
    snapshot_id: String,
    boundary_sort: Option<i64>,
    boundary_id: Option<String>,
    navigation: PageNavigation,
}

#[derive(Clone)]
pub struct PostQueries {
    pub(super) database: Database,
}

impl PostQueries {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn page(&self, filter: PostListFilter) -> SystemResult<CursorPage<Post>> {
        let codec = SystemCursorCodec::post(&filter)?;
        let decoded = codec.decode(&filter.page)?;
        let snapshot = resolve_snapshot(&self.database, &filter, decoded.as_ref().map(|cursor| &cursor.snapshot)).await?;
        let Some(snapshot) = snapshot else {
            return Ok(CursorPage::new(Vec::new(), None, None));
        };
        let window = post_window(decoded.as_ref(), &snapshot, filter.page.limit)?;
        let records = fetch_records(&self.database, &filter, &window).await?;
        build_page(
            records,
            PageBuildContext {
                codec: &codec,
                snapshot: &snapshot,
                navigation: &window.navigation,
            },
        )
    }

    pub async fn options(&self) -> StorageResult<Vec<Post>> {
        query_as::<_, PostRecord>(AssertSqlSafe(format!(
            "SELECT {COLUMNS} FROM sys_post WHERE status='0' ORDER BY post_sort ASC,post_id ASC"
        )))
        .fetch_all(self.database.pool())
        .await
        .map_err(StorageError::from)?
        .into_iter()
        .map(post)
        .collect()
    }

    pub async fn create(&self, input: PostInput) -> StorageResult<Post> {
        let id = self.database.next_id();
        query(insert_sql())
            .bind(&id)
            .bind(input.post_code)
            .bind(input.post_name)
            .bind(input.post_sort)
            .bind(input.status)
            .bind(input.remark)
            .bind(OffsetDateTime::now_utc())
            .execute(self.database.pool())
            .await?;
        self.find(&id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn replace(&self, id: &str, input: PostInput) -> StorageResult<Post> {
        let result = query(update_sql())
            .bind(id)
            .bind(input.post_code)
            .bind(input.post_name)
            .bind(input.post_sort)
            .bind(input.status)
            .bind(input.remark)
            .execute(self.database.pool())
            .await?;
        ensure_rows(result.rows_affected())?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete(&self, id: &str) -> StorageResult<()> {
        let result = query("DELETE FROM sys_post WHERE post_id = $1").bind(id).execute(self.database.pool()).await?;
        ensure_rows(result.rows_affected())
    }

    pub async fn delete_many(&self, ids: &[String]) -> StorageResult<()> {
        let mut tx = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_post WHERE post_id = ANY($1)").bind(ids).execute(&mut *tx).await?;
        ensure_batch_rows(result.rows_affected(), ids.len())?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn has_users(&self, id: &str) -> StorageResult<bool> {
        query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_user_post WHERE post_id = $1)")
            .bind(id)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub async fn code_exists(&self, code: &str, current_id: Option<&str>) -> StorageResult<bool> {
        query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_post WHERE post_code=$1 AND ($2::text IS NULL OR post_id<>$2))")
            .bind(code)
            .bind(current_id)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub async fn name_exists(&self, name: &str, current_id: Option<&str>) -> StorageResult<bool> {
        query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_post WHERE post_name=$1 AND ($2::text IS NULL OR post_id<>$2))")
            .bind(name)
            .bind(current_id)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub async fn find(&self, id: &str) -> StorageResult<Option<Post>> {
        query_as::<_, PostRecord>(AssertSqlSafe(format!("SELECT {COLUMNS} FROM sys_post WHERE post_id = $1")))
            .bind(id)
            .fetch_optional(self.database.pool())
            .await
            .map_err(StorageError::from)?
            .map(post)
            .transpose()
    }
}

async fn resolve_snapshot(database: &Database, filter: &PostListFilter, decoded: Option<&TimeIdPoint>) -> SystemResult<Option<TimeIdPoint>> {
    if let Some(snapshot) = decoded {
        return Ok(Some(snapshot.clone()));
    }
    let mut query = filtered_query(filter);
    query.push(" ORDER BY create_time DESC,post_id DESC LIMIT 1");
    let record = query
        .build_query_as::<PostRecord>()
        .fetch_optional(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)?;
    record.map(|record| record.snapshot()).transpose()
}

async fn fetch_records(database: &Database, filter: &PostListFilter, window: &PostPageWindow) -> SystemResult<Vec<PostRecord>> {
    let mut query = filtered_query(filter);
    push_window(&mut query, window)?;
    query
        .build_query_as::<PostRecord>()
        .fetch_all(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)
}

pub(super) fn filtered_query(filter: &PostListFilter) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new(format!("SELECT {COLUMNS} FROM sys_post WHERE TRUE"));
    if let Some(value) = &filter.post_code {
        query.push(" AND post_code ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
    }
    if let Some(value) = &filter.post_name {
        query.push(" AND post_name ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
    }
    if let Some(value) = &filter.status {
        query.push(" AND status=").push_bind(value.clone());
    }
    if let Some(value) = &filter.remark {
        query.push(" AND remark ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
    }
    if let Some(value) = filter.begin_time {
        query.push(" AND create_time>=").push_bind(value);
    }
    if let Some(value) = filter.end_time {
        query.push(" AND create_time<=").push_bind(value);
    }
    query
}

fn post_window(decoded: Option<&crate::application::SystemDecodedCursor>, snapshot: &TimeIdPoint, limit: u64) -> SystemResult<PostPageWindow> {
    let (boundary_sort, boundary_id) = match decoded.map(|cursor| &cursor.boundary) {
        Some(SystemBoundary::Post { post_sort, post_id }) => (Some(*post_sort), Some(post_id.clone())),
        _ => (None, None),
    };
    Ok(PostPageWindow {
        snapshot_time: point_time(snapshot)?,
        snapshot_id: snapshot.id.clone(),
        boundary_sort,
        boundary_id,
        navigation: navigation(decoded, limit),
    })
}

fn push_window(query: &mut QueryBuilder<Postgres>, window: &PostPageWindow) -> SystemResult<()> {
    query.push(" AND (create_time,post_id)<=(").push_bind(window.snapshot_time);
    query.push(",").push_bind(window.snapshot_id.clone()).push(")");
    if let (Some(sort), Some(id)) = (window.boundary_sort, window.boundary_id.clone()) {
        let operator = if window.navigation.direction == CursorDirection::Next { ">" } else { "<" };
        query.push(" AND (post_sort,post_id)").push(operator).push("(").push_bind(sort);
        query.push(",").push_bind(id).push(")");
    }
    let order = if window.navigation.direction == CursorDirection::Next {
        "ASC"
    } else {
        "DESC"
    };
    query.push(" ORDER BY post_sort ").push(order).push(",post_id ").push(order);
    let fetch_limit = window.navigation.limit.checked_add(1).ok_or_else(|| numeric_error("cursor limit overflow"))?;
    query.push(" LIMIT ").push_bind(storage::database::to_i64(fetch_limit).map_err(numeric_error)?);
    Ok(())
}

fn numeric_error(error: impl std::fmt::Display) -> crate::application::SystemError {
    crate::application::SystemError::Infrastructure(format!("post cursor numeric conversion failed: {error}"))
}

pub(super) fn insert_sql() -> &'static str {
    "INSERT INTO sys_post (post_id,post_code,post_name,post_sort,status,remark,create_time) VALUES ($1,$2,$3,$4,$5,$6,$7)"
}

pub(super) fn update_sql() -> &'static str {
    "UPDATE sys_post SET post_code=$2,post_name=$3,post_sort=$4,status=$5,remark=$6,update_time=CURRENT_TIMESTAMP WHERE post_id=$1"
}

pub(super) fn ensure_rows(rows: u64) -> StorageResult<()> {
    if rows == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

pub(super) fn ensure_batch_rows(rows: u64, expected: usize) -> StorageResult<()> {
    if rows != expected as u64 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
#[path = "post_tests.rs"]
mod tests;
