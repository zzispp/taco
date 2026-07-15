use audit_contract::AuditOutboxRecord;
use sqlx::{Postgres, Transaction, query};
use storage::{StorageError, StorageResult};
use time::OffsetDateTime;

use crate::domain::{Dept, DeptInput, SortBatchInput};

use super::{
    audited_transaction::commit_audited_write,
    dept::{ChildAncestorsUpdate, DeptQueries, ancestors, ensure_rows},
    dept_sql,
};

impl DeptQueries {
    pub(in crate::infra) async fn create_with_audit(&self, input: DeptInput, audit: &AuditOutboxRecord) -> StorageResult<Dept> {
        let id = self.database.next_id();
        let parent_ancestors = ancestors(self.database.pool(), &input.parent_id).await?;
        let mut transaction = self.database.pool().begin().await?;
        query(dept_sql::insert_sql())
            .bind(&id)
            .bind(input.parent_id)
            .bind(parent_ancestors)
            .bind(input.dept_name)
            .bind(input.order_num)
            .bind(input.leader)
            .bind(input.phone)
            .bind(input.email)
            .bind(input.status)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut *transaction)
            .await?;
        commit_audited_write(transaction, audit).await?;
        self.find(&id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn replace_with_audit(&self, id: &str, input: DeptInput, audit: &AuditOutboxRecord) -> StorageResult<Dept> {
        let current = self.find(id).await?.ok_or(StorageError::NotFound)?;
        let next_ancestors = ancestors(self.database.pool(), &input.parent_id).await?;
        let parent_changed = current.parent_id != input.parent_id;
        let old_prefix = current.ancestors;
        let new_prefix = next_ancestors.clone();
        let mut transaction = self.database.pool().begin().await?;
        let result = query(dept_sql::update_sql())
            .bind(id)
            .bind(input.parent_id)
            .bind(&next_ancestors)
            .bind(input.dept_name)
            .bind(input.order_num)
            .bind(input.leader)
            .bind(input.phone)
            .bind(input.email)
            .bind(input.status)
            .execute(&mut *transaction)
            .await?;
        ensure_rows(result.rows_affected())?;
        if parent_changed {
            update_child_ancestors_in_transaction(
                &mut transaction,
                ChildAncestorsUpdate {
                    id,
                    old_prefix: &old_prefix,
                    new_prefix: &new_prefix,
                },
            )
            .await?;
        }
        commit_audited_write(transaction, audit).await?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn update_sort_with_audit(&self, id: &str, order_num: i64, audit: &AuditOutboxRecord) -> StorageResult<Dept> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("UPDATE sys_dept SET order_num=$2,update_time=CURRENT_TIMESTAMP WHERE dept_id=$1 AND del_flag='0'")
            .bind(id)
            .bind(order_num)
            .execute(&mut *transaction)
            .await?;
        ensure_rows(result.rows_affected())?;
        commit_audited_write(transaction, audit).await?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn update_sorts_with_audit(&self, input: SortBatchInput, audit: &AuditOutboxRecord) -> StorageResult<Vec<Dept>> {
        let ids = input.items.iter().map(|item| item.id.clone()).collect::<Vec<_>>();
        let mut transaction = self.database.pool().begin().await?;
        for item in input.items {
            let result = query("UPDATE sys_dept SET order_num=$2,update_time=CURRENT_TIMESTAMP WHERE dept_id=$1 AND del_flag='0'")
                .bind(item.id)
                .bind(item.order_num)
                .execute(&mut *transaction)
                .await?;
            ensure_rows(result.rows_affected())?;
        }
        commit_audited_write(transaction, audit).await?;
        let mut departments = Vec::with_capacity(ids.len());
        for id in ids {
            departments.push(self.find(&id).await?.ok_or(StorageError::NotFound)?);
        }
        Ok(departments)
    }

    pub(in crate::infra) async fn delete_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("UPDATE sys_dept SET del_flag = '2', update_time = CURRENT_TIMESTAMP WHERE dept_id = $1 AND del_flag = '0'")
            .bind(id)
            .execute(&mut *transaction)
            .await?;
        ensure_rows(result.rows_affected())?;
        commit_audited_write(transaction, audit).await
    }
}

async fn update_child_ancestors_in_transaction(transaction: &mut Transaction<'_, Postgres>, update: ChildAncestorsUpdate<'_>) -> StorageResult<()> {
    query(
        r#"
        UPDATE sys_dept
        SET ancestors = $3 || substring(ancestors from length($2) + 1),
            update_time = CURRENT_TIMESTAMP
        WHERE del_flag = '0' AND (',' || ancestors || ',') LIKE '%,' || $1 || ',%'
        "#,
    )
    .bind(update.id)
    .bind(format!("{},{}", update.old_prefix, update.id))
    .bind(format!("{},{}", update.new_prefix, update.id))
    .execute(&mut **transaction)
    .await
    .map(|_| ())
    .map_err(StorageError::from)
}
