use kernel::pagination::Page;
use sqlx::{AssertSqlSafe, query, query_as, query_scalar};
use storage::{Database, StorageError, StorageResult};
use time::OffsetDateTime;

use crate::{
    application::PostListFilter,
    domain::{Post, PostInput},
};

use super::{mapping::post, page, record::PostRecord};

const COLUMNS: &str = "post_id,post_code,post_name,post_sort,status,remark,create_time::text AS create_time";

#[derive(Clone)]
pub struct PostQueries {
    database: Database,
}

impl PostQueries {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn page(&self, filter: PostListFilter) -> StorageResult<Page<Post>> {
        let total = query_scalar::<_, i64>(AssertSqlSafe(total_sql()))
            .bind(&filter.post_code)
            .bind(&filter.post_name)
            .bind(&filter.status)
            .bind(&filter.remark)
            .bind(filter.begin_time)
            .bind(filter.end_time)
            .fetch_one(self.database.pool())
            .await?;
        let records = query_as::<_, PostRecord>(AssertSqlSafe(page_sql()))
            .bind(&filter.post_code)
            .bind(&filter.post_name)
            .bind(&filter.status)
            .bind(&filter.remark)
            .bind(filter.begin_time)
            .bind(filter.end_time)
            .bind(page::limit(filter.page)?)
            .bind(page::offset(filter.page)?)
            .fetch_all(self.database.pool())
            .await?;
        page::page(records.into_iter().map(post).collect(), total, filter.page)
    }

    pub async fn options(&self) -> StorageResult<Vec<Post>> {
        query_as::<_, PostRecord>(AssertSqlSafe(format!("SELECT {COLUMNS} FROM sys_post WHERE status='0' ORDER BY post_sort ASC")))
            .fetch_all(self.database.pool())
            .await
            .map(|rows| rows.into_iter().map(post).collect())
            .map_err(StorageError::from)
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
            .map(|record| record.map(post))
            .map_err(StorageError::from)
    }
}

fn predicate() -> &'static str {
    "($1::text IS NULL OR post_code ILIKE '%' || $1 || '%') AND ($2::text IS NULL OR post_name ILIKE '%' || $2 || '%') AND ($3::text IS NULL OR status=$3) AND ($4::text IS NULL OR remark ILIKE '%' || $4 || '%') AND ($5::timestamptz IS NULL OR create_time >= $5) AND ($6::timestamptz IS NULL OR create_time <= $6)"
}

fn page_sql() -> String {
    format!("SELECT {COLUMNS} FROM sys_post WHERE {} ORDER BY post_sort ASC LIMIT $7 OFFSET $8", predicate())
}

fn total_sql() -> String {
    format!("SELECT COUNT(*) FROM sys_post WHERE {}", predicate())
}

fn insert_sql() -> &'static str {
    "INSERT INTO sys_post (post_id,post_code,post_name,post_sort,status,remark,create_time) VALUES ($1,$2,$3,$4,$5,$6,$7)"
}

fn update_sql() -> &'static str {
    "UPDATE sys_post SET post_code=$2,post_name=$3,post_sort=$4,status=$5,remark=$6,update_time=CURRENT_TIMESTAMP WHERE post_id=$1"
}

fn ensure_rows(rows: u64) -> StorageResult<()> {
    if rows == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

fn ensure_batch_rows(rows: u64, expected: usize) -> StorageResult<()> {
    if rows != expected as u64 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::page_sql;

    #[test]
    fn post_text_filters_use_case_insensitive_search() {
        let sql = page_sql();

        assert!(sql.contains("post_code ILIKE"));
        assert!(sql.contains("post_name ILIKE"));
        assert!(sql.contains("remark ILIKE"));
    }

    #[test]
    fn post_time_filters_compare_timestamps_without_date_truncation() {
        let sql = page_sql();

        assert!(sql.contains("create_time >="));
        assert!(sql.contains("create_time <="));
        assert!(!sql.contains("::date"));
    }
}
