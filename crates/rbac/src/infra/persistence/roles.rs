use std::future::Future;

use hook_tracing::db_query_metric;
use kernel::pagination::{Page, PageSliceRequest};
use sqlx::{query, query_as, query_scalar};
use storage::{
    StorageError, StorageResult,
    database::{to_i64, to_u64},
};
use time::OffsetDateTime;
use types::rbac::Role;

use super::{
    RbacStore, RoleRecord, RoleRecordInput,
    repository::{ROLE_COLUMNS, ensure_rows_affected, rbac_page},
};

impl RbacStore {
    pub async fn create_role(&self, input: RoleRecordInput) -> StorageResult<Role> {
        record_query("create_role", async {
            let now = OffsetDateTime::now_utc();
            let record = query_as::<_, RoleRecord>(&format!(
                r#"
                INSERT INTO roles (code, name, description, enabled, system, sort_order, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
                RETURNING {ROLE_COLUMNS}
                "#
            ))
            .bind(input.code)
            .bind(input.name)
            .bind(input.description)
            .bind(input.enabled)
            .bind(input.system)
            .bind(input.sort_order)
            .bind(now)
            .fetch_one(self.database.pool())
            .await?;
            Ok(record.into())
        })
        .await
    }

    pub async fn replace_role(&self, code: &str, input: RoleRecordInput) -> StorageResult<Role> {
        let now = OffsetDateTime::now_utc();
        let record = query_as::<_, RoleRecord>(&format!(
            r#"
            UPDATE roles
            SET name = $2,
                description = $3,
                enabled = $4,
                system = $5,
                sort_order = $6,
                updated_at = $7
            WHERE code = $1
            RETURNING {ROLE_COLUMNS}
            "#
        ))
        .bind(code)
        .bind(input.name)
        .bind(input.description)
        .bind(input.enabled)
        .bind(input.system)
        .bind(input.sort_order)
        .bind(now)
        .fetch_optional(self.database.pool())
        .await?;
        record.map(Role::from).ok_or(StorageError::NotFound)
    }

    pub async fn delete_role(&self, code: &str) -> StorageResult<()> {
        let result = query("DELETE FROM roles WHERE code = $1").bind(code).execute(self.database.pool()).await?;
        ensure_rows_affected(result.rows_affected())
    }

    pub async fn find_role(&self, code: &str) -> StorageResult<Option<Role>> {
        self.find_role_record(code).await.map(|record| record.map(Role::from))
    }

    pub async fn list_roles(&self) -> StorageResult<Vec<Role>> {
        record_query("list_roles", async {
            query_as::<_, RoleRecord>(&format!(
                r#"
                SELECT {ROLE_COLUMNS}
                FROM roles
                ORDER BY sort_order ASC
                "#
            ))
            .fetch_all(self.database.pool())
            .await
            .map(|records| records.into_iter().map(Role::from).collect())
            .map_err(StorageError::from)
        })
        .await
    }

    pub async fn page_roles(&self, request: PageSliceRequest) -> StorageResult<Page<Role>> {
        record_query("page_roles", async {
            let total = query_scalar::<_, i64>("SELECT COUNT(*) FROM roles").fetch_one(self.database.pool()).await?;
            let items = query_as::<_, RoleRecord>(&format!(
                r#"
                SELECT {ROLE_COLUMNS}
                FROM roles
                ORDER BY sort_order ASC
                LIMIT $1 OFFSET $2
                "#
            ))
            .bind(to_i64(request.limit)?)
            .bind(to_i64(request.offset)?)
            .fetch_all(self.database.pool())
            .await?;
            Ok(rbac_page(items.into_iter().map(Role::from).collect(), to_u64(total)?, request))
        })
        .await
    }

    pub async fn role_has_users(&self, code: &str) -> StorageResult<bool> {
        record_query("role_has_users", async {
            query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE role = $1 AND is_deleted = FALSE)")
                .bind(code)
                .fetch_one(self.database.pool())
                .await
                .map_err(StorageError::from)
        })
        .await
    }

    pub(super) async fn find_role_record(&self, code: &str) -> StorageResult<Option<RoleRecord>> {
        query_as::<_, RoleRecord>(&format!(
            r#"
            SELECT {ROLE_COLUMNS}
            FROM roles
            WHERE code = $1
            "#
        ))
        .bind(code)
        .fetch_optional(self.database.pool())
        .await
        .map_err(StorageError::from)
    }
}

async fn record_query<T, F>(operation: &'static str, action: F) -> StorageResult<T>
where
    F: Future<Output = StorageResult<T>>,
{
    db_query_metric("rbac_store.roles", operation, action).await
}
