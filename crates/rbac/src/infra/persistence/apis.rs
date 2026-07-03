use kernel::pagination::{Page, PageSliceRequest};
use sqlx::{query, query_as, query_scalar};
use storage::{
    StorageError, StorageResult,
    database::{to_i64, to_u64},
};
use time::OffsetDateTime;
use types::rbac::ApiPermission;

use super::{
    ApiPermissionRecord, ApiPermissionRecordInput, RbacStore,
    repository::{API_PERMISSION_COLUMNS, ensure_rows_affected, rbac_page},
};

impl RbacStore {
    pub async fn create_api(&self, input: ApiPermissionRecordInput) -> StorageResult<ApiPermission> {
        let now = OffsetDateTime::now_utc();
        let record = query_as::<_, ApiPermissionRecord>(&format!(
            r#"
            INSERT INTO api_permissions (
                id, code, method, path_pattern, name, "group", enabled, system, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
            RETURNING {API_PERMISSION_COLUMNS}
            "#
        ))
        .bind(self.database.next_id())
        .bind(input.code)
        .bind(input.method)
        .bind(input.path_pattern)
        .bind(input.name)
        .bind(input.group)
        .bind(input.enabled)
        .bind(input.system)
        .bind(now)
        .fetch_one(self.database.pool())
        .await?;
        Ok(record.into())
    }

    pub async fn replace_api(&self, id: &str, input: ApiPermissionRecordInput) -> StorageResult<ApiPermission> {
        let now = OffsetDateTime::now_utc();
        let record = query_as::<_, ApiPermissionRecord>(&format!(
            r#"
            UPDATE api_permissions
            SET code = $2,
                method = $3,
                path_pattern = $4,
                name = $5,
                "group" = $6,
                enabled = $7,
                system = $8,
                updated_at = $9
            WHERE id = $1
            RETURNING {API_PERMISSION_COLUMNS}
            "#
        ))
        .bind(id)
        .bind(input.code)
        .bind(input.method)
        .bind(input.path_pattern)
        .bind(input.name)
        .bind(input.group)
        .bind(input.enabled)
        .bind(input.system)
        .bind(now)
        .fetch_optional(self.database.pool())
        .await?;
        record.map(ApiPermission::from).ok_or(StorageError::NotFound)
    }

    pub async fn delete_api(&self, id: &str) -> StorageResult<()> {
        let result = query("DELETE FROM api_permissions WHERE id = $1")
            .bind(id)
            .execute(self.database.pool())
            .await?;
        ensure_rows_affected(result.rows_affected())
    }

    pub async fn find_api(&self, id: &str) -> StorageResult<Option<ApiPermission>> {
        self.find_api_record(id).await.map(|record| record.map(ApiPermission::from))
    }

    pub async fn list_apis(&self) -> StorageResult<Vec<ApiPermission>> {
        query_as::<_, ApiPermissionRecord>(&format!(
            r#"
            SELECT {API_PERMISSION_COLUMNS}
            FROM api_permissions
            ORDER BY id ASC
            "#
        ))
        .fetch_all(self.database.pool())
        .await
        .map(|records| records.into_iter().map(ApiPermission::from).collect())
        .map_err(StorageError::from)
    }

    pub async fn page_apis(&self, request: PageSliceRequest) -> StorageResult<Page<ApiPermission>> {
        let total = query_scalar::<_, i64>("SELECT COUNT(*) FROM api_permissions")
            .fetch_one(self.database.pool())
            .await?;
        let items = query_as::<_, ApiPermissionRecord>(&format!(
            r#"
            SELECT {API_PERMISSION_COLUMNS}
            FROM api_permissions
            ORDER BY id ASC
            LIMIT $1 OFFSET $2
            "#
        ))
        .bind(to_i64(request.limit)?)
        .bind(to_i64(request.offset)?)
        .fetch_all(self.database.pool())
        .await?;
        Ok(rbac_page(items.into_iter().map(ApiPermission::from).collect(), to_u64(total)?, request))
    }

    async fn find_api_record(&self, id: &str) -> StorageResult<Option<ApiPermissionRecord>> {
        query_as::<_, ApiPermissionRecord>(&format!(
            r#"
            SELECT {API_PERMISSION_COLUMNS}
            FROM api_permissions
            WHERE id = $1
            "#
        ))
        .bind(id)
        .fetch_optional(self.database.pool())
        .await
        .map_err(StorageError::from)
    }
}
