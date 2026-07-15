use audit_contract::AuditOutboxRecord;
use sqlx::{Postgres, Transaction, query};
use storage::{StorageError, StorageResult};
use time::OffsetDateTime;

use crate::{
    domain::{Role, RoleDataScopeInput, RoleDeptBindingInput, RoleInput, RoleMenuBindingInput, RoleUserBindingInput},
    infra::audited_transaction::commit_audited_write,
};

use super::{
    RoleQueries,
    cleanup::delete_role_relations,
    relations::{
        insert_dept_ids, insert_user_ids, normalize_data_scope_dept_ids, normalize_menu_ids, replace_dept_ids_in_transaction, replace_menu_ids_in_transaction,
    },
    sql::{insert_role_sql, update_role_sql},
    support::{ensure_batch_rows, ensure_rows_affected},
};

impl RoleQueries {
    pub(in crate::infra) async fn create_with_audit(&self, input: RoleInput, audit: &AuditOutboxRecord) -> StorageResult<Role> {
        let id = self.database.next_id();
        let mut transaction = self.database.pool().begin().await?;
        insert_role(&mut transaction, &id, input).await?;
        commit_audited_write(transaction, audit).await?;
        self.find(&id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn replace_with_audit(&self, id: &str, input: RoleInput, audit: &AuditOutboxRecord) -> StorageResult<Role> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query(update_role_sql())
            .bind(id)
            .bind(input.role_name)
            .bind(input.role_key)
            .bind(input.role_sort)
            .bind(input.data_scope)
            .bind(input.menu_check_strictly)
            .bind(input.dept_check_strictly)
            .bind(input.status)
            .bind(input.remark)
            .execute(&mut *transaction)
            .await?;
        ensure_rows_affected(result.rows_affected())?;
        commit_audited_write(transaction, audit).await?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn update_status_with_audit(&self, id: &str, status: String, audit: &AuditOutboxRecord) -> StorageResult<Role> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("UPDATE sys_role SET status=$2, update_time=CURRENT_TIMESTAMP WHERE role_id=$1 AND del_flag='0'")
            .bind(id)
            .bind(status)
            .execute(&mut *transaction)
            .await?;
        ensure_rows_affected(result.rows_affected())?;
        commit_audited_write(transaction, audit).await?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn update_data_scope_with_audit(&self, id: &str, input: RoleDataScopeInput, audit: &AuditOutboxRecord) -> StorageResult<Role> {
        let dept_ids = normalize_data_scope_dept_ids(self.database.pool(), &input).await?;
        let mut transaction = self.database.pool().begin().await?;
        let result = query("UPDATE sys_role SET data_scope=$2,dept_check_strictly=$3,update_time=CURRENT_TIMESTAMP WHERE role_id=$1 AND del_flag='0'")
            .bind(id)
            .bind(input.data_scope)
            .bind(input.dept_check_strictly)
            .execute(&mut *transaction)
            .await?;
        ensure_rows_affected(result.rows_affected())?;
        query("DELETE FROM sys_role_dept WHERE role_id=$1").bind(id).execute(&mut *transaction).await?;
        insert_dept_ids(&mut transaction, id, dept_ids).await?;
        commit_audited_write(transaction, audit).await?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra) async fn delete_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("UPDATE sys_role SET del_flag = '2', update_time = CURRENT_TIMESTAMP WHERE role_id = $1 AND del_flag = '0'")
            .bind(id)
            .execute(&mut *transaction)
            .await?;
        ensure_rows_affected(result.rows_affected())?;
        delete_role_relations(&mut transaction, &[id.to_owned()]).await?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra) async fn delete_many_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("UPDATE sys_role SET del_flag = '2', update_time = $2 WHERE role_id = ANY($1) AND del_flag = '0'")
            .bind(ids)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut *transaction)
            .await?;
        ensure_batch_rows(result.rows_affected(), ids.len())?;
        delete_role_relations(&mut transaction, ids).await?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra) async fn replace_users_with_audit(&self, role_id: &str, input: RoleUserBindingInput, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        insert_user_ids(&mut transaction, role_id, input.user_ids).await?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra) async fn delete_user_with_audit(&self, role_id: &str, user_id: &str, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_user_role WHERE role_id=$1 AND user_id=$2")
            .bind(role_id)
            .bind(user_id)
            .execute(&mut *transaction)
            .await?;
        ensure_rows_affected(result.rows_affected())?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra) async fn delete_users_with_audit(&self, role_id: &str, user_ids: &[String], audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_user_role WHERE role_id=$1 AND user_id = ANY($2)")
            .bind(role_id)
            .bind(user_ids)
            .execute(&mut *transaction)
            .await?;
        ensure_batch_rows(result.rows_affected(), user_ids.len())?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra) async fn replace_menus_with_audit(&self, role_id: &str, input: RoleMenuBindingInput, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let menu_ids = normalize_menu_ids(self.database.pool(), &input.menu_ids).await?;
        let mut transaction = self.database.pool().begin().await?;
        replace_menu_ids_in_transaction(&mut transaction, role_id, menu_ids).await?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra) async fn replace_depts_with_audit(&self, role_id: &str, input: RoleDeptBindingInput, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let mut transaction = self.database.pool().begin().await?;
        replace_dept_ids_in_transaction(&mut transaction, role_id, input.dept_ids).await?;
        commit_audited_write(transaction, audit).await
    }
}

async fn insert_role(transaction: &mut Transaction<'_, Postgres>, id: &str, input: RoleInput) -> StorageResult<()> {
    query(insert_role_sql())
        .bind(id)
        .bind(input.role_name)
        .bind(input.role_key)
        .bind(input.role_sort)
        .bind(input.data_scope)
        .bind(input.menu_check_strictly)
        .bind(input.dept_check_strictly)
        .bind(input.status)
        .bind(input.remark)
        .bind(OffsetDateTime::now_utc())
        .execute(&mut **transaction)
        .await?;
    Ok(())
}
