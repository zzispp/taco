use std::future::Future;

use hook_tracing::db_query_metric;
use sqlx::{Postgres, QueryBuilder, query_as, query_scalar};
use storage::{StorageError, StorageResult};
use time::OffsetDateTime;

use super::{
    RbacStore, RoleApiBindingRecordInput, RoleApiPermissionRecord, RoleMenuBindingRecordInput, RoleMenuPermissionRecord,
    repository::{ROLE_API_BINDING_COLUMNS, ROLE_MENU_BINDING_COLUMNS},
};

impl RbacStore {
    pub async fn replace_role_apis(&self, role_code: &str, inputs: Vec<RoleApiBindingRecordInput>) -> StorageResult<()> {
        record_query("replace_role_apis", async {
            let mut tx = self.database.pool().begin().await?;
            sqlx::query("DELETE FROM role_api_permissions WHERE role_code = $1")
                .bind(role_code)
                .execute(&mut *tx)
                .await?;
            insert_role_api_bindings(&mut tx, inputs).await?;
            tx.commit().await.map_err(StorageError::from)
        })
        .await
    }

    pub async fn replace_role_menus(&self, role_code: &str, inputs: Vec<RoleMenuBindingRecordInput>) -> StorageResult<()> {
        let mut tx = self.database.pool().begin().await?;
        sqlx::query("DELETE FROM role_menu_permissions WHERE role_code = $1")
            .bind(role_code)
            .execute(&mut *tx)
            .await?;
        insert_role_menu_bindings(&mut tx, inputs).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub async fn list_role_api_bindings(&self) -> StorageResult<Vec<RoleApiBindingRecordInput>> {
        query_as::<_, RoleApiPermissionRecord>(&format!(
            r#"
            SELECT {ROLE_API_BINDING_COLUMNS}
            FROM role_api_permissions
            "#
        ))
        .fetch_all(self.database.pool())
        .await
        .map(role_api_binding_records)
        .map_err(StorageError::from)
    }

    pub async fn role_api_ids(&self, role_code: &str) -> StorageResult<Vec<String>> {
        record_query("role_api_ids", async {
            query_scalar::<_, String>("SELECT api_permission_id FROM role_api_permissions WHERE role_code = $1")
                .bind(role_code)
                .fetch_all(self.database.pool())
                .await
                .map_err(StorageError::from)
        })
        .await
    }

    pub async fn role_has_api_bindings(&self, role_code: &str) -> StorageResult<bool> {
        binding_exists("role_api_permissions", "role_code", role_code, self.database.pool()).await
    }

    pub async fn api_has_role_bindings(&self, api_permission_id: &str) -> StorageResult<bool> {
        binding_exists("role_api_permissions", "api_permission_id", api_permission_id, self.database.pool()).await
    }

    pub async fn list_role_menu_bindings(&self) -> StorageResult<Vec<RoleMenuBindingRecordInput>> {
        query_as::<_, RoleMenuPermissionRecord>(&format!(
            r#"
            SELECT {ROLE_MENU_BINDING_COLUMNS}
            FROM role_menu_permissions
            "#
        ))
        .fetch_all(self.database.pool())
        .await
        .map(role_menu_binding_records)
        .map_err(StorageError::from)
    }

    pub async fn role_menu_item_ids(&self, role_code: &str) -> StorageResult<Vec<String>> {
        query_scalar::<_, String>("SELECT menu_item_id FROM role_menu_permissions WHERE role_code = $1")
            .bind(role_code)
            .fetch_all(self.database.pool())
            .await
            .map_err(StorageError::from)
    }

    pub async fn role_has_menu_bindings(&self, role_code: &str) -> StorageResult<bool> {
        binding_exists("role_menu_permissions", "role_code", role_code, self.database.pool()).await
    }

    pub async fn menu_item_has_role_bindings(&self, menu_item_id: &str) -> StorageResult<bool> {
        binding_exists("role_menu_permissions", "menu_item_id", menu_item_id, self.database.pool()).await
    }
}

async fn record_query<T, F>(operation: &'static str, action: F) -> StorageResult<T>
where
    F: Future<Output = StorageResult<T>>,
{
    db_query_metric("rbac_store.bindings", operation, action).await
}

async fn insert_role_api_bindings(tx: &mut sqlx::Transaction<'_, Postgres>, inputs: Vec<RoleApiBindingRecordInput>) -> StorageResult<()> {
    if inputs.is_empty() {
        return Ok(());
    }

    let now = OffsetDateTime::now_utc();
    let mut builder = QueryBuilder::<Postgres>::new("INSERT INTO role_api_permissions (role_code, api_permission_id, created_at, updated_at) ");
    builder.push_values(inputs, |mut row, input| {
        row.push_bind(input.role_code).push_bind(input.api_permission_id).push_bind(now).push_bind(now);
    });
    builder.build().execute(&mut **tx).await?;
    Ok(())
}

async fn insert_role_menu_bindings(tx: &mut sqlx::Transaction<'_, Postgres>, inputs: Vec<RoleMenuBindingRecordInput>) -> StorageResult<()> {
    if inputs.is_empty() {
        return Ok(());
    }

    let now = OffsetDateTime::now_utc();
    let mut builder = QueryBuilder::<Postgres>::new("INSERT INTO role_menu_permissions (role_code, menu_item_id, created_at, updated_at) ");
    builder.push_values(inputs, |mut row, input| {
        row.push_bind(input.role_code).push_bind(input.menu_item_id).push_bind(now).push_bind(now);
    });
    builder.build().execute(&mut **tx).await?;
    Ok(())
}

async fn binding_exists(table: &str, column: &str, value: &str, pool: &sqlx::PgPool) -> StorageResult<bool> {
    query_scalar::<_, bool>(&format!("SELECT EXISTS(SELECT 1 FROM {table} WHERE {column} = $1)"))
        .bind(value)
        .fetch_one(pool)
        .await
        .map_err(StorageError::from)
}

fn role_api_binding_records(records: Vec<RoleApiPermissionRecord>) -> Vec<RoleApiBindingRecordInput> {
    records
        .into_iter()
        .map(|record| RoleApiBindingRecordInput {
            role_code: record.role_code,
            api_permission_id: record.api_permission_id,
        })
        .collect()
}

fn role_menu_binding_records(records: Vec<RoleMenuPermissionRecord>) -> Vec<RoleMenuBindingRecordInput> {
    records
        .into_iter()
        .map(|record| RoleMenuBindingRecordInput {
            role_code: record.role_code,
            menu_item_id: record.menu_item_id,
        })
        .collect()
}
