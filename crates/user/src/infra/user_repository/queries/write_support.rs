use sqlx::query;
use storage::{StorageError, StorageResult};
use time::OffsetDateTime;
use types::user::UserId;

use crate::application::ReplaceUserRecord;

use super::UserQueries;
use crate::infra::user_repository::{sql, write};

impl UserQueries {
    pub(super) async fn insert_user(&self, user_id: &str, input: ReplaceUserRecord, password_hash: String) -> StorageResult<()> {
        let mut tx = self.database.pool().begin().await?;
        let now = OffsetDateTime::now_utc();
        query(sql::insert_user())
            .bind(user_id)
            .bind(input.dept_id)
            .bind(input.username)
            .bind(input.nick_name)
            .bind(input.email)
            .bind(input.phonenumber)
            .bind(input.sex)
            .bind(password_hash)
            .bind(input.status)
            .bind(input.remark)
            .bind(now)
            .execute(&mut *tx)
            .await?;
        write::replace_relations(
            &mut tx,
            user_id,
            write::UserRelationIds {
                role_ids: input.role_ids,
                post_ids: input.post_ids,
            },
        )
        .await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub(super) async fn update_user(&self, id: &UserId, input: ReplaceUserRecord) -> StorageResult<()> {
        let mut tx = self.database.pool().begin().await?;
        write::execute_user_update(&mut tx, id, &input).await?;
        write::replace_relations(
            &mut tx,
            &id.0,
            write::UserRelationIds {
                role_ids: input.role_ids,
                post_ids: input.post_ids,
            },
        )
        .await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub(super) async fn ensure_references(&self, input: &ReplaceUserRecord) -> StorageResult<()> {
        write::ensure_dept_exists(self.database.pool(), input.dept_id.as_deref()).await?;
        write::ensure_ids_exist(self.database.pool(), write::ReferenceTable::role(), &input.role_ids).await?;
        write::ensure_ids_exist(self.database.pool(), write::ReferenceTable::post(), &input.post_ids).await
    }
}

pub(super) fn required_password(password_hash: Option<String>) -> StorageResult<String> {
    password_hash.ok_or_else(|| StorageError::Database("password_hash is required".into()))
}
