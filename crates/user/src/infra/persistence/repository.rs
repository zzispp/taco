use std::future::Future;

use constants::pagination::PAGE_INDEX_OFFSET;
use hook_tracing::db_query_metric;
use sqlx::{PgPool, query, query_as, query_scalar};
use storage::{
    Database, StorageError, StorageResult,
    database::{to_i64, to_u64},
};
use time::OffsetDateTime;
use types::{
    pagination::{Page, PageRequest, PageSliceRequest},
    user::{User, UserId},
};

use super::{UserAuthRecord, UserRecord, UserRecordInput};

const USER_COLUMNS: &str = r#"
    id,
    username,
    password_hash,
    email,
    role,
    is_active,
    is_deleted,
    created_at,
    updated_at,
    last_login_at,
    auth_source,
    email_verified
"#;

#[derive(Clone)]
pub struct UserStore {
    database: Database,
}

impl UserStore {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn create(&self, user: UserRecordInput) -> StorageResult<User> {
        record_query("create", async {
            ensure_role_exists(self.database.pool(), &user.role).await?;
            let now = OffsetDateTime::now_utc();
            let record = query_as::<_, UserRecord>(&format!(
                r#"
                INSERT INTO users (
                    id, username, password_hash, email, role, is_active, is_deleted,
                    created_at, updated_at, last_login_at, auth_source, email_verified
                )
                VALUES ($1, $2, $3, $4, $5, $6, FALSE, $7, $7, NULL, $8, FALSE)
                RETURNING {USER_COLUMNS}
                "#
            ))
            .bind(self.database.next_id())
            .bind(user.username)
            .bind(user.password_hash)
            .bind(user.email)
            .bind(user.role)
            .bind(user.is_active)
            .bind(now)
            .bind(UserRecord::local_auth_source())
            .fetch_one(self.database.pool())
            .await?;
            Ok(record.into())
        })
        .await
    }

    pub async fn replace(&self, id: UserId, user: UserRecordInput) -> StorageResult<User> {
        ensure_role_exists(self.database.pool(), &user.role).await?;
        let now = OffsetDateTime::now_utc();
        let record = query_as::<_, UserRecord>(&format!(
            r#"
            UPDATE users
            SET username = $2,
                password_hash = $3,
                email = $4,
                role = $5,
                is_active = $6,
                updated_at = $7
            WHERE id = $1 AND is_deleted = FALSE
            RETURNING {USER_COLUMNS}
            "#
        ))
        .bind(id.0.as_str())
        .bind(user.username)
        .bind(user.password_hash)
        .bind(user.email)
        .bind(user.role)
        .bind(user.is_active)
        .bind(now)
        .fetch_optional(self.database.pool())
        .await?;
        record.map(User::from).ok_or(StorageError::NotFound)
    }

    pub async fn delete(&self, id: UserId) -> StorageResult<()> {
        record_query("delete", async {
            let now = OffsetDateTime::now_utc();
            let result = query(
                r#"
                UPDATE users
                SET is_deleted = TRUE, updated_at = $2
                WHERE id = $1 AND is_deleted = FALSE
                "#,
            )
            .bind(id.0)
            .bind(now)
            .execute(self.database.pool())
            .await?;
            ensure_rows_affected(result.rows_affected())
        })
        .await
    }

    pub async fn find_by_id(&self, id: UserId) -> StorageResult<Option<User>> {
        self.find_record_by_id(&id).await.map(|record| record.map(User::from))
    }

    pub async fn find_by_email(&self, email: &str) -> StorageResult<Option<User>> {
        self.find_record_by_email(email).await.map(|record| record.map(User::from))
    }

    pub async fn find_auth_by_username(&self, username: &str) -> StorageResult<Option<UserAuthRecord>> {
        self.find_record_by_username(username).await.map(|record| record.map(UserRecord::into_auth))
    }

    pub async fn find_auth_by_email(&self, email: &str) -> StorageResult<Option<UserAuthRecord>> {
        self.find_record_by_email(email).await.map(|record| record.map(UserRecord::into_auth))
    }

    pub async fn record_login(&self, id: UserId) -> StorageResult<()> {
        let now = OffsetDateTime::now_utc();
        let result = query(
            r#"
            UPDATE users
            SET last_login_at = $2, updated_at = $2
            WHERE id = $1 AND is_deleted = FALSE
            "#,
        )
        .bind(id.0)
        .bind(now)
        .execute(self.database.pool())
        .await?;
        ensure_rows_affected(result.rows_affected())
    }

    pub async fn list(&self, page: PageRequest) -> StorageResult<Page<User>> {
        self.list_slice(PageSliceRequest {
            offset: (page.page - PAGE_INDEX_OFFSET) * page.page_size,
            limit: page.page_size,
            page: page.page,
            page_size: page.page_size,
        })
        .await
    }

    pub async fn list_slice(&self, request: PageSliceRequest) -> StorageResult<Page<User>> {
        record_query("list_slice", async {
            let total = query_scalar::<_, i64>("SELECT COUNT(*) FROM users WHERE is_deleted = FALSE")
                .fetch_one(self.database.pool())
                .await?;
            let items = query_as::<_, UserRecord>(&format!(
                r#"
                SELECT {USER_COLUMNS}
                FROM users
                WHERE is_deleted = FALSE
                ORDER BY created_at ASC
                LIMIT $1 OFFSET $2
                "#
            ))
            .bind(to_i64(request.limit)?)
            .bind(to_i64(request.offset)?)
            .fetch_all(self.database.pool())
            .await?;
            Ok(Page {
                items: items.into_iter().map(User::from).collect(),
                total: to_u64(total)?,
                page: request.page,
                page_size: request.page_size,
            })
        })
        .await
    }

    async fn find_record_by_id(&self, id: &UserId) -> StorageResult<Option<UserRecord>> {
        self.find_active_record("id = $1", id.0.as_str()).await
    }

    async fn find_record_by_email(&self, email: &str) -> StorageResult<Option<UserRecord>> {
        self.find_active_record("email = $1", email).await
    }

    async fn find_record_by_username(&self, username: &str) -> StorageResult<Option<UserRecord>> {
        self.find_active_record("username = $1", username).await
    }

    async fn find_active_record(&self, predicate: &str, value: &str) -> StorageResult<Option<UserRecord>> {
        record_query("find_active_record", async {
            query_as::<_, UserRecord>(&format!(
                r#"
                SELECT {USER_COLUMNS}
                FROM users
                WHERE is_deleted = FALSE AND {predicate}
                "#
            ))
            .bind(value)
            .fetch_optional(self.database.pool())
            .await
            .map_err(StorageError::from)
        })
        .await
    }
}

async fn ensure_role_exists(pool: &PgPool, role: &str) -> StorageResult<()> {
    let exists = query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM roles WHERE code = $1)")
        .bind(role)
        .fetch_one(pool)
        .await?;
    if exists {
        return Ok(());
    }
    Err(StorageError::Conflict(format!("role does not exist: {role}")))
}

fn ensure_rows_affected(rows_affected: u64) -> StorageResult<()> {
    if rows_affected == 0 {
        return Err(StorageError::NotFound);
    }
    Ok(())
}

async fn record_query<T, F>(operation: &'static str, action: F) -> StorageResult<T>
where
    F: Future<Output = StorageResult<T>>,
{
    db_query_metric("user_store", operation, action).await
}
