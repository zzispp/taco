use kernel::pagination::Page;
use sqlx::{AssertSqlSafe, query, query_as, query_scalar};
use storage::{Database, StorageError, StorageResult};
use time::OffsetDateTime;

use crate::{
    application::ConfigListFilter,
    domain::{ConfigInput, ConfigItem},
};

use super::{mapping::config, page, record::ConfigRecord};

const COLUMNS: &str = "config_id,config_name,config_key,config_value,config_type,public_read,remark,create_time::text AS create_time";

#[derive(Clone)]
pub struct ConfigQueries {
    database: Database,
}

impl ConfigQueries {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn page(&self, filter: ConfigListFilter) -> StorageResult<Page<ConfigItem>> {
        let total = query_scalar::<_, i64>(AssertSqlSafe(total_sql()))
            .bind(&filter.config_name)
            .bind(&filter.config_key)
            .bind(&filter.config_type)
            .bind(filter.public_read)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .fetch_one(self.database.pool())
            .await?;
        let rows = query_as::<_, ConfigRecord>(AssertSqlSafe(page_sql()))
            .bind(&filter.config_name)
            .bind(&filter.config_key)
            .bind(&filter.config_type)
            .bind(filter.public_read)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .bind(page::limit(filter.page)?)
            .bind(page::offset(filter.page)?)
            .fetch_all(self.database.pool())
            .await?;
        page::page(rows.into_iter().map(config).collect(), total, filter.page)
    }

    pub async fn list(&self, filter: ConfigListFilter) -> StorageResult<Vec<ConfigItem>> {
        query_as::<_, ConfigRecord>(AssertSqlSafe(list_sql()))
            .bind(&filter.config_name)
            .bind(&filter.config_key)
            .bind(&filter.config_type)
            .bind(filter.public_read)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .fetch_all(self.database.pool())
            .await
            .map(|rows| rows.into_iter().map(config).collect())
            .map_err(StorageError::from)
    }

    pub async fn create(&self, input: ConfigInput) -> StorageResult<ConfigItem> {
        let id = self.database.next_id();
        query(insert_sql())
            .bind(&id)
            .bind(input.config_name)
            .bind(input.config_key)
            .bind(input.config_value)
            .bind(input.config_type)
            .bind(input.public_read)
            .bind(input.remark)
            .bind(OffsetDateTime::now_utc())
            .execute(self.database.pool())
            .await?;
        self.find(&id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn replace(&self, id: &str, input: ConfigInput) -> StorageResult<ConfigItem> {
        let result = query(update_sql())
            .bind(id)
            .bind(input.config_name)
            .bind(input.config_key)
            .bind(input.config_value)
            .bind(input.config_type)
            .bind(input.public_read)
            .bind(input.remark)
            .execute(self.database.pool())
            .await?;
        ensure_rows(result.rows_affected())?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete(&self, id: &str) -> StorageResult<()> {
        let result = query("DELETE FROM sys_config WHERE config_id = $1")
            .bind(id)
            .execute(self.database.pool())
            .await?;
        ensure_rows(result.rows_affected())
    }

    pub async fn delete_many(&self, ids: &[String]) -> StorageResult<()> {
        let mut tx = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_config WHERE config_id = ANY($1)").bind(ids).execute(&mut *tx).await?;
        ensure_batch_rows(result.rows_affected(), ids.len())?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn find(&self, id: &str) -> StorageResult<Option<ConfigItem>> {
        query_as::<_, ConfigRecord>(AssertSqlSafe(format!("SELECT {COLUMNS} FROM sys_config WHERE config_id = $1")))
            .bind(id)
            .fetch_optional(self.database.pool())
            .await
            .map(|row| row.map(config))
            .map_err(StorageError::from)
    }

    pub async fn find_by_key(&self, key: &str) -> StorageResult<Option<ConfigItem>> {
        query_as::<_, ConfigRecord>(AssertSqlSafe(format!("SELECT {COLUMNS} FROM sys_config WHERE config_key = $1")))
            .bind(key)
            .fetch_optional(self.database.pool())
            .await
            .map(|row| row.map(config))
            .map_err(StorageError::from)
    }

    pub async fn value_by_key(&self, key: &str) -> StorageResult<Option<String>> {
        query_scalar::<_, String>("SELECT config_value FROM sys_config WHERE config_key=$1")
            .bind(key)
            .fetch_optional(self.database.pool())
            .await
            .map_err(StorageError::from)
    }
}

fn predicate() -> &'static str {
    "($1::text IS NULL OR config_name ILIKE '%' || $1 || '%') AND ($2::text IS NULL OR config_key ILIKE '%' || $2 || '%') AND ($3::text IS NULL OR config_type=$3) AND ($4::boolean IS NULL OR public_read=$4::boolean) AND ($5::text IS NULL OR create_time::date >= $5::date) AND ($6::text IS NULL OR create_time::date <= $6::date)"
}
fn list_sql() -> String {
    format!("SELECT {COLUMNS} FROM sys_config WHERE {} ORDER BY config_id ASC", predicate())
}
fn page_sql() -> String {
    format!(
        "SELECT {COLUMNS} FROM sys_config WHERE {} ORDER BY config_id ASC LIMIT $7 OFFSET $8",
        predicate()
    )
}
fn total_sql() -> String {
    format!("SELECT COUNT(*) FROM sys_config WHERE {}", predicate())
}
fn insert_sql() -> &'static str {
    "INSERT INTO sys_config (config_id,config_name,config_key,config_value,config_type,public_read,remark,create_time) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)"
}
fn update_sql() -> &'static str {
    "UPDATE sys_config SET config_name=$2,config_key=$3,config_value=$4,config_type=$5,public_read=$6,remark=$7,update_time=CURRENT_TIMESTAMP WHERE config_id=$1"
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
    fn config_text_filters_use_case_insensitive_search() {
        let sql = page_sql();

        assert!(sql.contains("config_name ILIKE"));
        assert!(sql.contains("config_key ILIKE"));
    }
}
