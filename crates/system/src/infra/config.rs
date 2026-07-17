use audit_contract::AuditOutboxRecord;
use constants::system_config::SYSTEM_CONFIG_CHANGED_CHANNEL;
use kernel::pagination::CursorPage;
use sqlx::{AssertSqlSafe, query, query_as, query_scalar};
use storage::{Database, StorageError, StorageResult};
use time::OffsetDateTime;

use crate::{
    application::{ConfigListFilter, SystemResult},
    domain::{ConfigInput, ConfigItem},
};

use super::{audited_transaction::commit_audited_write, mapping::config, record::ConfigRecord};

pub(super) const COLUMNS: &str = "config_id,config_name,config_key,config_value,config_type,public_read,remark,create_time";

#[path = "config_pages.rs"]
mod pages;
pub(super) use pages::filtered_query;

#[derive(Clone)]
pub struct ConfigQueries {
    database: Database,
}

impl ConfigQueries {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn page(&self, filter: ConfigListFilter) -> SystemResult<CursorPage<ConfigItem>> {
        pages::page(&self.database, filter).await
    }

    pub async fn list(&self, filter: ConfigListFilter) -> StorageResult<Vec<ConfigItem>> {
        let mut query = pages::filtered_query(&filter);
        query.push(" ORDER BY config_id ASC");
        query
            .build_query_as::<ConfigRecord>()
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)?
            .into_iter()
            .map(config)
            .collect()
    }

    pub async fn create(&self, input: ConfigInput) -> StorageResult<ConfigItem> {
        let id = self.database.next_id();
        let config_key = input.config_key.clone();
        let mut transaction = self.database.pool().begin().await?;
        query(insert_sql())
            .bind(&id)
            .bind(input.config_name)
            .bind(input.config_key)
            .bind(input.config_value)
            .bind(input.config_type)
            .bind(input.public_read)
            .bind(input.remark)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut *transaction)
            .await?;
        notify_config_changed(&mut transaction, &config_key).await?;
        transaction.commit().await?;
        self.find(&id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn create_with_audit(&self, input: ConfigInput, audit: &AuditOutboxRecord) -> StorageResult<ConfigItem> {
        let id = self.database.next_id();
        let config_key = input.config_key.clone();
        let mut transaction = self.database.pool().begin().await?;
        query(insert_sql())
            .bind(&id)
            .bind(input.config_name)
            .bind(input.config_key)
            .bind(input.config_value)
            .bind(input.config_type)
            .bind(input.public_read)
            .bind(input.remark)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut *transaction)
            .await?;
        notify_config_changed(&mut transaction, &config_key).await?;
        commit_audited_write(transaction, audit).await?;
        self.find(&id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn replace(&self, id: &str, input: ConfigInput) -> StorageResult<ConfigItem> {
        let config_key = input.config_key.clone();
        let mut transaction = self.database.pool().begin().await?;
        let result = query(update_sql())
            .bind(id)
            .bind(input.config_name)
            .bind(input.config_key)
            .bind(input.config_value)
            .bind(input.config_type)
            .bind(input.public_read)
            .bind(input.remark)
            .execute(&mut *transaction)
            .await?;
        ensure_rows(result.rows_affected())?;
        notify_config_changed(&mut transaction, &config_key).await?;
        transaction.commit().await?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn replace_with_audit(&self, id: &str, input: ConfigInput, audit: &AuditOutboxRecord) -> StorageResult<ConfigItem> {
        let config_key = input.config_key.clone();
        let mut transaction = self.database.pool().begin().await?;
        let result = query(update_sql())
            .bind(id)
            .bind(input.config_name)
            .bind(input.config_key)
            .bind(input.config_value)
            .bind(input.config_type)
            .bind(input.public_read)
            .bind(input.remark)
            .execute(&mut *transaction)
            .await?;
        ensure_rows(result.rows_affected())?;
        notify_config_changed(&mut transaction, &config_key).await?;
        commit_audited_write(transaction, audit).await?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete(&self, id: &str) -> StorageResult<()> {
        let result = query("DELETE FROM sys_config WHERE config_id = $1")
            .bind(id)
            .execute(self.database.pool())
            .await?;
        ensure_rows(result.rows_affected())
    }

    pub(in crate::infra) async fn delete_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_config WHERE config_id = $1").bind(id).execute(&mut *transaction).await?;
        ensure_rows(result.rows_affected())?;
        commit_audited_write(transaction, audit).await
    }

    pub async fn delete_many(&self, ids: &[String]) -> StorageResult<()> {
        let mut tx = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_config WHERE config_id = ANY($1)").bind(ids).execute(&mut *tx).await?;
        ensure_batch_rows(result.rows_affected(), ids.len())?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub(in crate::infra) async fn delete_many_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_config WHERE config_id = ANY($1)")
            .bind(ids)
            .execute(&mut *transaction)
            .await?;
        ensure_batch_rows(result.rows_affected(), ids.len())?;
        commit_audited_write(transaction, audit).await
    }

    pub async fn find(&self, id: &str) -> StorageResult<Option<ConfigItem>> {
        query_as::<_, ConfigRecord>(AssertSqlSafe(format!("SELECT {COLUMNS} FROM sys_config WHERE config_id = $1")))
            .bind(id)
            .fetch_optional(self.database.pool())
            .await
            .map_err(StorageError::from)?
            .map(config)
            .transpose()
    }

    pub async fn find_by_key(&self, key: &str) -> StorageResult<Option<ConfigItem>> {
        query_as::<_, ConfigRecord>(AssertSqlSafe(format!("SELECT {COLUMNS} FROM sys_config WHERE config_key = $1")))
            .bind(key)
            .fetch_optional(self.database.pool())
            .await
            .map_err(StorageError::from)?
            .map(config)
            .transpose()
    }

    pub async fn value_by_key(&self, key: &str) -> StorageResult<Option<String>> {
        query_scalar::<_, String>("SELECT config_value FROM sys_config WHERE config_key=$1")
            .bind(key)
            .fetch_optional(self.database.pool())
            .await
            .map_err(StorageError::from)
    }
}

fn insert_sql() -> &'static str {
    "INSERT INTO sys_config (config_id,config_name,config_key,config_value,config_type,public_read,remark,create_time) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)"
}
fn update_sql() -> &'static str {
    "UPDATE sys_config SET config_name=$2,config_key=$3,config_value=$4,config_type=$5,public_read=$6,remark=$7,update_time=CURRENT_TIMESTAMP WHERE config_id=$1"
}

async fn notify_config_changed(transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>, config_key: &str) -> StorageResult<()> {
    query("SELECT pg_notify($1, $2)")
        .bind(SYSTEM_CONFIG_CHANGED_CHANNEL)
        .bind(config_key)
        .execute(&mut **transaction)
        .await?;
    Ok(())
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
