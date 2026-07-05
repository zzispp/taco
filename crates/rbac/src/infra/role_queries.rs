use kernel::pagination::Page;
use sqlx::{Postgres, query, query_as, query_scalar};
use storage::{
    Database, StorageError, StorageResult,
    database::{to_i64, to_u64},
};
use time::OffsetDateTime;

use crate::{
    application::{RoleListFilter, RoleUserListFilter},
    domain::{DataScopeFilter, Role, RoleDataScopeInput, RoleDeptBindingInput, RoleInput, RoleMenuBindingInput, RoleOption, RoleUser, RoleUserBindingInput},
};

use super::{
    mapping::{role, role_option, role_user},
    records::{RoleDeptRecord, RoleOptionRecord, RolePermissionRecord, RoleRecord, RoleUserRecord},
};

const ROLE_COLUMNS: &str = r#"
    r.role_id, r.role_name, r.role_key, r.role_sort, r.data_scope, r.menu_check_strictly,
    r.dept_check_strictly, r.status, r.system, r.remark, r.create_time::text AS create_time
"#;

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
        let mut tx = self.database.pool().begin().await?;
        let result = query("UPDATE sys_role SET data_scope=$2,dept_check_strictly=$3,update_time=CURRENT_TIMESTAMP WHERE role_id=$1 AND del_flag='0'")
            .bind(id)
            .bind(input.data_scope)
            .bind(input.dept_check_strictly)
            .execute(&mut *tx)
            .await?;
        ensure_rows_affected(result.rows_affected())?;
        query("DELETE FROM sys_role_dept WHERE role_id=$1").bind(id).execute(&mut *tx).await?;
        insert_ids(&mut tx, "sys_role_dept", "dept_id", id, input.dept_ids).await?;
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
        query_as::<_, RoleRecord>(&format!("SELECT {ROLE_COLUMNS} FROM sys_role r WHERE r.role_id = $1 AND r.del_flag = '0'"))
            .bind(id)
            .fetch_optional(self.database.pool())
            .await
            .map(|record| record.map(role))
            .map_err(StorageError::from)
    }

    pub async fn role_name_exists(&self, name: &str, current_id: Option<&str>) -> StorageResult<bool> {
        unique_exists(self.database.pool(), "role_name", name, current_id).await
    }

    pub async fn role_key_exists(&self, key: &str, current_id: Option<&str>) -> StorageResult<bool> {
        unique_exists(self.database.pool(), "role_key", key, current_id).await
    }

    pub async fn has_users(&self, id: &str) -> StorageResult<bool> {
        query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_user_role WHERE role_id = $1)")
            .bind(id)
            .fetch_one(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub async fn page(&self, filter: RoleListFilter) -> StorageResult<Page<Role>> {
        let total = query_scalar::<_, i64>(&role_total_sql())
            .bind(&filter.role_name)
            .bind(&filter.role_key)
            .bind(&filter.status)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .fetch_one(self.database.pool())
            .await?;
        let items = query_as::<_, RoleRecord>(&role_page_sql())
            .bind(&filter.role_name)
            .bind(&filter.role_key)
            .bind(&filter.status)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .bind(to_i64(filter.page.page_size)?)
            .bind(to_i64((filter.page.page - 1) * filter.page.page_size)?)
            .fetch_all(self.database.pool())
            .await?;
        role_page(items, total, filter)
    }

    pub async fn page_scoped(&self, filter: RoleListFilter, scope: DataScopeFilter) -> StorageResult<Page<Role>> {
        let total = query_scalar::<_, i64>(&role_scoped_total_sql())
            .bind(&filter.role_name)
            .bind(&filter.role_key)
            .bind(&filter.status)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .bind(&scope.data_scope)
            .bind(&scope.user_id)
            .bind(&scope.dept_id)
            .bind(&scope.dept_ids)
            .fetch_one(self.database.pool())
            .await?;
        let items = query_as::<_, RoleRecord>(&role_scoped_page_sql())
            .bind(&filter.role_name)
            .bind(&filter.role_key)
            .bind(&filter.status)
            .bind(&filter.begin_time)
            .bind(&filter.end_time)
            .bind(&scope.data_scope)
            .bind(&scope.user_id)
            .bind(&scope.dept_id)
            .bind(&scope.dept_ids)
            .bind(to_i64(filter.page.page_size)?)
            .bind(to_i64((filter.page.page - 1) * filter.page.page_size)?)
            .fetch_all(self.database.pool())
            .await?;
        role_page(items, total, filter)
    }

    pub async fn options(&self) -> StorageResult<Vec<RoleOption>> {
        query_as::<_, RoleOptionRecord>("SELECT role_id,role_name,role_key,status FROM sys_role WHERE del_flag='0' ORDER BY role_sort ASC")
            .fetch_all(self.database.pool())
            .await
            .map(|rows| rows.into_iter().map(role_option).collect())
            .map_err(StorageError::from)
    }

    pub async fn page_users(&self, filter: RoleUserListFilter, scope: Option<DataScopeFilter>) -> StorageResult<Page<RoleUser>> {
        let total = query_scalar::<_, i64>(&role_users_total_sql(scope.is_some()))
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
        let items = query_as::<_, RoleUserRecord>(&role_users_page_sql(scope.is_some()))
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
        replace_ids(self.database.pool(), "sys_role_menu", "menu_id", role_id, input.menu_ids).await
    }

    pub async fn replace_depts(&self, role_id: &str, input: RoleDeptBindingInput) -> StorageResult<()> {
        replace_ids(self.database.pool(), "sys_role_dept", "dept_id", role_id, input.dept_ids).await
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
        binding_ids(self.database.pool(), "sys_role_menu", "menu_id", role_id).await
    }

    pub async fn dept_ids(&self, role_id: &str) -> StorageResult<Vec<String>> {
        binding_ids(self.database.pool(), "sys_role_dept", "dept_id", role_id).await
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

async fn replace_ids(pool: &sqlx::PgPool, table: &str, id_column: &str, role_id: &str, ids: Vec<String>) -> StorageResult<()> {
    let mut tx = pool.begin().await?;
    query(&format!("DELETE FROM {table} WHERE role_id = $1"))
        .bind(role_id)
        .execute(&mut *tx)
        .await?;
    insert_ids(&mut tx, table, id_column, role_id, ids).await?;
    tx.commit().await.map_err(StorageError::from)
}

async fn insert_ids(tx: &mut sqlx::Transaction<'_, Postgres>, table: &str, column: &str, role_id: &str, ids: Vec<String>) -> StorageResult<()> {
    for id in ids {
        query(&format!("INSERT INTO {table} (role_id, {column}) VALUES ($1, $2)"))
            .bind(role_id)
            .bind(id)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

async fn binding_ids(pool: &sqlx::PgPool, table: &str, column: &str, role_id: &str) -> StorageResult<Vec<String>> {
    query_scalar::<_, String>(&format!("SELECT {column} FROM {table} WHERE role_id = $1 ORDER BY {column} ASC"))
        .bind(role_id)
        .fetch_all(pool)
        .await
        .map_err(StorageError::from)
}

async fn unique_exists(pool: &sqlx::PgPool, column: &str, value: &str, current_id: Option<&str>) -> StorageResult<bool> {
    query_scalar::<_, bool>(&format!(
        "SELECT EXISTS(SELECT 1 FROM sys_role WHERE del_flag='0' AND {column}=$1 AND ($2::text IS NULL OR role_id<>$2))"
    ))
    .bind(value)
    .bind(current_id)
    .fetch_one(pool)
    .await
    .map_err(StorageError::from)
}

fn insert_role_sql() -> &'static str {
    "INSERT INTO sys_role (role_id, role_name, role_key, role_sort, data_scope, menu_check_strictly, dept_check_strictly, status, del_flag, system, remark, create_time) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'0',FALSE,$9,$10)"
}

fn update_role_sql() -> &'static str {
    "UPDATE sys_role SET role_name=$2, role_key=$3, role_sort=$4, data_scope=$5, menu_check_strictly=$6, dept_check_strictly=$7, status=$8, remark=$9, update_time=CURRENT_TIMESTAMP WHERE role_id=$1 AND del_flag='0'"
}

fn permission_query() -> &'static str {
    r#"
    SELECT r.role_key, r.status, r.data_scope, m.perms
    FROM sys_role r
    CROSS JOIN sys_menu m
    WHERE r.role_key = 'admin' AND r.del_flag = '0'
    UNION
    SELECT r.role_key, r.status, r.data_scope, m.perms
    FROM sys_role r
    LEFT JOIN sys_role_menu rm ON rm.role_id = r.role_id
    LEFT JOIN sys_menu m ON m.menu_id = rm.menu_id
    WHERE r.role_key <> 'admin' AND r.del_flag = '0'
    "#
}

fn dept_query() -> &'static str {
    "SELECT r.role_key, rd.dept_id FROM sys_role r INNER JOIN sys_role_dept rd ON rd.role_id = r.role_id WHERE r.del_flag = '0'"
}

fn ensure_rows_affected(rows_affected: u64) -> StorageResult<()> {
    if rows_affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

async fn replace_user_ids(pool: &sqlx::PgPool, role_id: &str, user_ids: Vec<String>) -> StorageResult<()> {
    let mut tx = pool.begin().await?;
    for id in user_ids {
        query("INSERT INTO sys_user_role (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(id)
            .bind(role_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await.map_err(StorageError::from)
}

fn role_page(items: Vec<RoleRecord>, total: i64, filter: RoleListFilter) -> StorageResult<Page<Role>> {
    Ok(Page {
        items: items.into_iter().map(role).collect(),
        total: to_u64(total)?,
        page: filter.page.page,
        page_size: filter.page.page_size,
    })
}

fn role_where() -> &'static str {
    "r.del_flag='0' AND ($1::text IS NULL OR r.role_name ILIKE '%' || $1 || '%') AND ($2::text IS NULL OR r.role_key ILIKE '%' || $2 || '%') AND ($3::text IS NULL OR r.status=$3) AND ($4::text IS NULL OR r.create_time::date >= $4::date) AND ($5::text IS NULL OR r.create_time::date <= $5::date)"
}

fn role_page_sql() -> String {
    format!(
        "SELECT {ROLE_COLUMNS} FROM sys_role r WHERE {} ORDER BY r.role_sort ASC LIMIT $6 OFFSET $7",
        role_where()
    )
}

fn role_total_sql() -> String {
    format!("SELECT COUNT(*) FROM sys_role r WHERE {}", role_where())
}

fn role_scoped_page_sql() -> String {
    format!(
        "SELECT DISTINCT {ROLE_COLUMNS} FROM sys_role r LEFT JOIN sys_user_role ur ON ur.role_id=r.role_id LEFT JOIN sys_user u ON u.user_id=ur.user_id LEFT JOIN sys_dept d ON d.dept_id=u.dept_id WHERE {} AND {} ORDER BY r.role_sort ASC LIMIT $10 OFFSET $11",
        role_where(),
        role_scope_where()
    )
}

fn role_scoped_total_sql() -> String {
    format!(
        "SELECT COUNT(DISTINCT r.role_id) FROM sys_role r LEFT JOIN sys_user_role ur ON ur.role_id=r.role_id LEFT JOIN sys_user u ON u.user_id=ur.user_id LEFT JOIN sys_dept d ON d.dept_id=u.dept_id WHERE {} AND {}",
        role_where(),
        role_scope_where()
    )
}

fn role_scope_where() -> &'static str {
    "($6='1' OR ($6='2' AND u.dept_id = ANY($9)) OR ($6='3' AND $8::text IS NOT NULL AND u.dept_id=$8) OR ($6='4' AND $8::text IS NOT NULL AND (u.dept_id=$8 OR (',' || d.ancestors || ',') LIKE '%,' || $8 || ',%')) OR ($6='5' AND u.user_id=$7))"
}

fn role_users_base(scoped: bool) -> String {
    let scope = if scoped { format!(" AND {}", user_scope_where()) } else { String::new() };
    format!(
        "FROM sys_user u LEFT JOIN sys_dept d ON d.dept_id=u.dept_id WHERE u.del_flag='0' AND ($2::text IS NULL OR u.user_name ILIKE '%' || $2 || '%') AND ($3::text IS NULL OR u.phonenumber ILIKE '%' || $3 || '%') AND (($4 AND EXISTS (SELECT 1 FROM sys_user_role ur WHERE ur.user_id=u.user_id AND ur.role_id=$1)) OR (NOT $4 AND NOT EXISTS (SELECT 1 FROM sys_user_role ur WHERE ur.user_id=u.user_id AND ur.role_id=$1))){}",
        scope
    )
}

fn role_users_page_sql(scoped: bool) -> String {
    format!(
        "SELECT u.user_id,u.user_name AS username,u.nick_name,u.dept_id,u.phonenumber,u.email,u.status {} ORDER BY u.create_time ASC LIMIT $9 OFFSET $10",
        role_users_base(scoped)
    )
}

fn role_users_total_sql(scoped: bool) -> String {
    format!("SELECT COUNT(*) {}", role_users_base(scoped))
}

fn user_scope_where() -> &'static str {
    "($5='1' OR ($5='2' AND u.dept_id = ANY($8)) OR ($5='3' AND $7::text IS NOT NULL AND u.dept_id=$7) OR ($5='4' AND $7::text IS NOT NULL AND (u.dept_id=$7 OR (',' || d.ancestors || ',') LIKE '%,' || $7 || ',%')) OR ($5='5' AND u.user_id=$6))"
}

fn ensure_batch_rows(rows: u64, expected: usize) -> StorageResult<()> {
    if rows != expected as u64 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}
