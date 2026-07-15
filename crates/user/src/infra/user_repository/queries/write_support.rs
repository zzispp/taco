use sqlx::{Postgres, Transaction, query};
use storage::{StorageError, StorageResult};
use time::OffsetDateTime;
use types::user::UserId;

use crate::application::ReplaceUserRecord;
use constants::system::STATUS_NORMAL;

use super::UserQueries;
use crate::infra::user_repository::{sql, write};

impl UserQueries {
    pub(super) async fn insert_user(&self, user_id: &str, input: ReplaceUserRecord, password_hash: String) -> StorageResult<()> {
        let mut tx = self.database.pool().begin().await?;
        insert_user_in_transaction(&mut tx, InsertUserCommand { user_id, input, password_hash }).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub(super) async fn update_user(&self, id: &UserId, input: ReplaceUserRecord) -> StorageResult<()> {
        let mut tx = self.database.pool().begin().await?;
        update_user_in_transaction(&mut tx, id, input).await?;
        tx.commit().await.map_err(StorageError::from)
    }

    pub(super) async fn ensure_references(&self, input: &ReplaceUserRecord) -> StorageResult<()> {
        write::ensure_dept_exists(self.database.pool(), input.dept_id.as_deref()).await?;
        write::ensure_ids_exist(self.database.pool(), write::ReferenceTable::role(), &input.role_ids).await?;
        write::ensure_ids_exist(self.database.pool(), write::ReferenceTable::post(), &input.post_ids).await
    }
}

pub(super) struct InsertUserCommand<'a> {
    pub(super) user_id: &'a str,
    pub(super) input: ReplaceUserRecord,
    pub(super) password_hash: String,
}

pub(super) async fn insert_user_in_transaction(tx: &mut Transaction<'_, Postgres>, command: InsertUserCommand<'_>) -> StorageResult<()> {
    let InsertUserCommand { user_id, input, password_hash } = command;
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
        .execute(&mut **tx)
        .await?;
    write::replace_relations(
        tx,
        user_id,
        write::UserRelationIds {
            role_ids: input.role_ids,
            post_ids: input.post_ids,
        },
    )
    .await
}

pub(super) async fn update_user_in_transaction(tx: &mut Transaction<'_, Postgres>, id: &UserId, input: ReplaceUserRecord) -> StorageResult<()> {
    let revoke_sessions = should_revoke_sessions(input.password_hash.is_some(), &input.status);
    write::execute_user_update(tx, id, &input).await?;
    write::replace_relations(
        tx,
        &id.0,
        write::UserRelationIds {
            role_ids: input.role_ids,
            post_ids: input.post_ids,
        },
    )
    .await?;
    if revoke_sessions {
        revoke_user_sessions(tx, std::slice::from_ref(&id.0)).await?;
    }
    Ok(())
}

pub(super) fn should_revoke_sessions(password_changed: bool, status: &str) -> bool {
    password_changed || status != STATUS_NORMAL
}

pub(super) async fn revoke_user_sessions(tx: &mut Transaction<'_, Postgres>, user_ids: &[String]) -> StorageResult<()> {
    if user_ids.is_empty() {
        return Ok(());
    }
    query("DELETE FROM sys_user_session WHERE user_id=ANY($1)")
        .bind(user_ids)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub(super) fn required_password(password_hash: Option<String>) -> StorageResult<String> {
    password_hash.ok_or_else(|| StorageError::Database("password_hash is required".into()))
}

#[cfg(test)]
mod tests {
    use super::should_revoke_sessions;

    #[test]
    fn password_and_disabled_status_revoke_but_an_unchanged_active_user_does_not() {
        assert!(!should_revoke_sessions(false, "0"));
        assert!(should_revoke_sessions(true, "0"));
        assert!(should_revoke_sessions(false, "1"));
    }
}
