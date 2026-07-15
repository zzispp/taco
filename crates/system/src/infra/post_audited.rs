use audit_contract::AuditOutboxRecord;
use sqlx::query;
use storage::{StorageError, StorageResult};
use time::OffsetDateTime;

use crate::domain::{Post, PostInput};

use super::{
    audited_transaction::commit_audited_write,
    post::{PostQueries, ensure_batch_rows, ensure_rows, insert_sql, update_sql},
};

impl PostQueries {
    pub(in crate::infra) async fn create_with_audit(&self, input: PostInput, audit: &AuditOutboxRecord) -> StorageResult<Post> {
        let id = self.database.next_id();
        let mut transaction = self.database.pool().begin().await?;
        query(insert_sql())
            .bind(&id)
            .bind(input.post_code)
            .bind(input.post_name)
            .bind(input.post_sort)
            .bind(input.status)
            .bind(input.remark)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut *transaction)
            .await?;
        commit_audited_write(transaction, audit).await?;
        self.find(&id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn replace_with_audit(&self, id: &str, input: PostInput, audit: &AuditOutboxRecord) -> StorageResult<Post> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query(update_sql())
            .bind(id)
            .bind(input.post_code)
            .bind(input.post_name)
            .bind(input.post_sort)
            .bind(input.status)
            .bind(input.remark)
            .execute(&mut *transaction)
            .await?;
        ensure_rows(result.rows_affected())?;
        commit_audited_write(transaction, audit).await?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn delete_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_post WHERE post_id = $1").bind(id).execute(&mut *transaction).await?;
        ensure_rows(result.rows_affected())?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra) async fn delete_many_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_post WHERE post_id = ANY($1)")
            .bind(ids)
            .execute(&mut *transaction)
            .await?;
        ensure_batch_rows(result.rows_affected(), ids.len())?;
        commit_audited_write(transaction, audit).await
    }
}
