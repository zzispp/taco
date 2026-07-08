use kernel::pagination::Page;
use sqlx::{AssertSqlSafe, query, query_as, query_scalar};
use storage::{
    Database, StorageError, StorageResult,
    database::{to_i64, to_u64},
};
use time::OffsetDateTime;

use super::{
    mapping::{role, role_option, role_user},
    records::{RoleDeptRecord, RoleOptionRecord, RolePermissionRecord, RoleRecord, RoleUserRecord},
};
use crate::{
    application::{RoleListFilter, RoleUserListFilter},
    domain::{DataScopeFilter, Role, RoleDataScopeInput, RoleDeptBindingInput, RoleInput, RoleMenuBindingInput, RoleOption, RoleUser, RoleUserBindingInput},
};

mod pages;
mod relations;
mod scoped_users;
mod sql;
mod support;

use self::{
    relations::{
        dept_ids, insert_dept_ids, menu_ids, normalize_data_scope_dept_ids, normalize_menu_ids, replace_dept_ids, replace_menu_ids, replace_user_ids,
        role_key_exists, role_name_exists,
    },
    sql::{ROLE_COLUMNS, dept_query, insert_role_sql, permission_query, role_users_page_sql, role_users_total_sql, update_role_sql},
    support::{ensure_batch_rows, ensure_rows_affected},
};

#[derive(Clone)]
pub struct RoleQueries {
    database: Database,
}

impl RoleQueries {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(&self, input: RoleInput) -> StorageResult<Role> {
        let id = self.database.next_id();
        query(insert_role_sql())
            .bind(&id)
            .bind(input.role_name)
            .bind(input.role_key)
            .bind(input.role_sort)
            .bind(input.data_scope)
            .bind(input.menu_check_strictly)
            .bind(input.dept_check_strictly)
            .bind(input.status)
            .bind(input.remark)
            .bind(OffsetDateTime::now_utc())
            .execute(self.database.pool())
            .await?;
        self.find(&id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn replace(&self, id: &str, input: RoleInput) -> StorageResult<Role> {
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
            .execute(self.database.pool())
            .await?;
        ensure_rows_affected(result.rows_affected())?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn update_status(&self, id: &str, status: String) -> StorageResult<Role> {
        let result = query("UPDATE sys_role SET status=$2, update_time=CURRENT_TIMESTAMP WHERE role_id=$1 AND del_flag='0'")
            .bind(id)
            .bind(status)
            .execute(self.database.pool())
            .await?;
        ensure_rows_affected(result.rows_affected())?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn update_data_scope(&self, id: &str, input: RoleDataScopeInput) -> StorageResult<Role> {
        let dept_ids = normalize_data_scope_dept_ids(self.database.pool(), &input).await?;
        let mut tx = self.database.pool().begin().await?;
        let result = query("UPDATE sys_role SET data_scope=$2,dept_check_strictly=$3,update_time=CURRENT_TIMESTAMP WHERE role_id=$1 AND del_flag='0'")
            .bind(id)
            .bind(input.data_scope)
            .bind(input.dept_check_strictly)
            .execute(&mut *tx)
            .await?;
        ensure_rows_affected(result.rows_affected())?;
        query("DELETE FROM sys_role_dept WHERE role_id=$1").bind(id).execute(&mut *tx).await?;
        insert_dept_ids(&mut tx, id, dept_ids).await?;
        tx.commit().await.map_err(StorageError::from)?;
        self.find(id).await?.ok_or(StorageError::NotFound)
    }

    pub async fn delete(&self, id: &str) -> StorageResult<()> {
        let result = query("UPDATE sys_role SET del_flag = '2', update_time = CURRENT_TIMESTAMP WHERE role_id = $1 AND del_flag = '0'")
            .bind(id)
            .execute(self.database.pool())
            .await?;
        ensure_rows_affected(result.rows_affected())
    }

    pub async fn delete_many(&self, ids: &[String]) -> StorageResult<()> {
        let mut tx = self.database.pool().begin().await?;
        let result = query("UPDATE sys_role SET del_flag = '2', update_time = CURRENT_TIMESTAMP WHERE role_id = ANY($1) AND del_flag = '0'")
            .bind(ids)
            .execute(&mut *tx)
            .await?;
        ensure_batch_rows(result.rows_affected(), ids.len())?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn find(&self, id: &str) -> StorageResult<Option<Role>> {
        query_as::<_, RoleRecord>(AssertSqlSafe(format!(
            "SELECT {ROLE_COLUMNS} FROM sys_role r WHERE r.role_id = $1 AND r.del_flag = '0'"
        )))
        .bind(id)
        .fetch_optional(self.database.pool())
        .await
        .map(|record| record.map(role))
        .map_err(StorageError::from)
    }

    pub async fn role_name_exists(&self, name: &str, current_id: Option<&str>) -> StorageResult<bool> {
        role_name_exists(self.database.pool(), name, current_id).await
    }

    pub async fn role_key_exists(&self, key: &str, current_id: Option<&str>) -> StorageResult<bool> {
        role_key_exists(self.database.pool(), key, current_id).await
    }

    pub async fn has_users(&self, id: &str) -> StorageResult<bool> {
        query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_user_role WHERE role_id = $1)")
            .bind(id)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub async fn page(&self, filter: RoleListFilter) -> StorageResult<Page<Role>> {
        pages::page(&self.database, filter).await
    }

    pub async fn page_scoped(&self, filter: RoleListFilter, scope: DataScopeFilter) -> StorageResult<Page<Role>> {
        pages::page_scoped(&self.database, filter, scope).await
    }

    pub async fn options(&self) -> StorageResult<Vec<RoleOption>> {
        query_as::<_, RoleOptionRecord>("SELECT role_id,role_name,role_key,status FROM sys_role WHERE del_flag='0' ORDER BY role_sort ASC")
            .fetch_all(self.database.pool())
            .await
            .map(|rows| rows.into_iter().map(role_option).collect())
            .map_err(StorageError::from)
    }

    pub async fn page_users(&self, filter: RoleUserListFilter, scope: Option<DataScopeFilter>) -> StorageResult<Page<RoleUser>> {
        let total = query_scalar::<_, i64>(AssertSqlSafe(role_users_total_sql(scope.is_some())))
            .bind(&filter.role_id)
            .bind(&filter.username)
            .bind(&filter.phonenumber)
            .bind(filter.allocated)
            .bind(scope.as_ref().map(|s| s.data_scope.as_str()))
            .bind(scope.as_ref().map(|s| s.user_id.as_str()))
            .bind(scope.as_ref().and_then(|s| s.dept_id.as_deref()))
            .bind(scope.as_ref().map(|s| s.dept_ids.as_slice()).unwrap_or(&[]))
            .fetch_one(self.database.pool())
            .await?;
        let items = query_as::<_, RoleUserRecord>(AssertSqlSafe(role_users_page_sql(scope.is_some())))
            .bind(&filter.role_id)
            .bind(&filter.username)
            .bind(&filter.phonenumber)
            .bind(filter.allocated)
            .bind(scope.as_ref().map(|s| s.data_scope.as_str()))
            .bind(scope.as_ref().map(|s| s.user_id.as_str()))
            .bind(scope.as_ref().and_then(|s| s.dept_id.as_deref()))
            .bind(scope.as_ref().map(|s| s.dept_ids.as_slice()).unwrap_or(&[]))
            .bind(to_i64(filter.page.page_size)?)
            .bind(to_i64((filter.page.page - 1) * filter.page.page_size)?)
            .fetch_all(self.database.pool())
            .await?;
        Ok(Page {
            items: items.into_iter().map(role_user).collect(),
            total: to_u64(total)?,
            page: filter.page.page,
            page_size: filter.page.page_size,
        })
    }

    pub async fn replace_menus(&self, role_id: &str, input: RoleMenuBindingInput) -> StorageResult<()> {
        let menu_ids = normalize_menu_ids(self.database.pool(), &input.menu_ids).await?;
        replace_menu_ids(self.database.pool(), role_id, menu_ids).await
    }

    pub async fn replace_depts(&self, role_id: &str, input: RoleDeptBindingInput) -> StorageResult<()> {
        replace_dept_ids(self.database.pool(), role_id, input.dept_ids).await
    }

    pub async fn replace_users(&self, role_id: &str, input: RoleUserBindingInput) -> StorageResult<()> {
        replace_user_ids(self.database.pool(), role_id, input.user_ids).await
    }

    pub async fn delete_user(&self, role_id: &str, user_id: &str) -> StorageResult<()> {
        let result = query("DELETE FROM sys_user_role WHERE role_id=$1 AND user_id=$2")
            .bind(role_id)
            .bind(user_id)
            .execute(self.database.pool())
            .await?;
        ensure_rows_affected(result.rows_affected())
    }

    pub async fn delete_users(&self, role_id: &str, user_ids: &[String]) -> StorageResult<()> {
        let mut tx = self.database.pool().begin().await?;
        let result = query("DELETE FROM sys_user_role WHERE role_id=$1 AND user_id = ANY($2)")
            .bind(role_id)
            .bind(user_ids)
            .execute(&mut *tx)
            .await?;
        ensure_batch_rows(result.rows_affected(), user_ids.len())?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn menu_ids(&self, role_id: &str) -> StorageResult<Vec<String>> {
        menu_ids(self.database.pool(), role_id).await
    }

    pub async fn dept_ids(&self, role_id: &str) -> StorageResult<Vec<String>> {
        dept_ids(self.database.pool(), role_id).await
    }

    pub async fn permission_rows(&self) -> StorageResult<Vec<RolePermissionRecord>> {
        query_as::<_, RolePermissionRecord>(permission_query())
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub async fn dept_rows(&self) -> StorageResult<Vec<RoleDeptRecord>> {
        query_as::<_, RoleDeptRecord>(dept_query())
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
    }
}
