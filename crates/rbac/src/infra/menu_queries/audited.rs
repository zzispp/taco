use audit_contract::AuditOutboxRecord;
use sqlx::query;
use storage::{StorageError, StorageResult};
use time::OffsetDateTime;
use types::system::SortBatchInput;

use crate::{
    domain::{Menu, MenuInput},
    infra::audited_transaction::commit_audited_write,
};

use super::{MenuQueries, ensure_rows_affected, insert_menu_sql, update_menu_sql};

impl MenuQueries {
    pub(in crate::infra) async fn create_with_audit(&self, input: MenuInput, audit: &AuditOutboxRecord) -> StorageResult<Menu> {
        let id = self.database.next_id();
        let mut transaction = self.database.pool().begin().await?;
        insert_menu(&mut transaction, &id, input).await?;
        commit_audited_write(transaction, audit).await?;
        self.find(&id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn replace_with_audit(&self, id: &str, input: MenuInput, audit: &AuditOutboxRecord) -> StorageResult<Menu> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query(update_menu_sql())
            .bind(id)
            .bind(input.menu_name)
            .bind(input.parent_id)
            .bind(input.order_num)
            .bind(input.path)
            .bind(input.component)
            .bind(input.query)
            .bind(input.route_name)
            .bind(input.is_frame)
            .bind(input.is_cache)
            .bind(input.menu_type)
            .bind(input.visible)
            .bind(input.status)
            .bind(input.perms)
            .bind(input.icon)
            .bind(input.remark)
            .execute(&mut *transaction)
            .await?;
        ensure_rows_affected(result.rows_affected())?;
        commit_audited_write(transaction, audit).await?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn update_sort_with_audit(&self, id: &str, order_num: i64, audit: &AuditOutboxRecord) -> StorageResult<Menu> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("UPDATE sys_menu SET order_num=$2, update_time=CURRENT_TIMESTAMP WHERE menu_id=$1")
            .bind(id)
            .bind(order_num)
            .execute(&mut *transaction)
            .await?;
        ensure_rows_affected(result.rows_affected())?;
        commit_audited_write(transaction, audit).await?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn update_sorts_with_audit(&self, input: SortBatchInput, audit: &AuditOutboxRecord) -> StorageResult<Vec<Menu>> {
        let ids = input.items.iter().map(|item| item.id.clone()).collect::<Vec<_>>();
        let mut transaction = self.database.pool().begin().await?;
        for item in input.items {
            let result = query("UPDATE sys_menu SET order_num=$2, update_time=CURRENT_TIMESTAMP WHERE menu_id=$1")
                .bind(item.id)
                .bind(item.order_num)
                .execute(&mut *transaction)
                .await?;
            ensure_rows_affected(result.rows_affected())?;
        }
        commit_audited_write(transaction, audit).await?;
        self.find_many(ids).await
    }

    pub(in crate::infra) async fn delete_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_menu WHERE menu_id = $1").bind(id).execute(&mut *transaction).await?;
        ensure_rows_affected(result.rows_affected())?;
        commit_audited_write(transaction, audit).await
    }
}

async fn insert_menu(transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>, id: &str, input: MenuInput) -> StorageResult<()> {
    query(insert_menu_sql())
        .bind(id)
        .bind(input.menu_name)
        .bind(input.parent_id)
        .bind(input.order_num)
        .bind(input.path)
        .bind(input.component)
        .bind(input.query)
        .bind(input.route_name)
        .bind(input.is_frame)
        .bind(input.is_cache)
        .bind(input.menu_type)
        .bind(input.visible)
        .bind(input.status)
        .bind(input.perms)
        .bind(input.icon)
        .bind(input.remark)
        .bind(OffsetDateTime::now_utc())
        .execute(&mut **transaction)
        .await?;
    Ok(())
}
