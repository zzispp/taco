use sqlx::{Postgres, Transaction, query, query_scalar};
use storage::{StorageError, StorageResult};

use crate::{
    application::ReplaceUserRecord,
    domain::{User, UserId},
};

use super::{
    UserQueries,
    write_support::{InsertUserCommand, insert_user_in_transaction, required_password},
};

impl UserQueries {
    pub(in crate::infra::user_repository) async fn has_installation_owner(&self) -> StorageResult<bool> {
        query_scalar("SELECT EXISTS(SELECT 1 FROM sys_installation_owner WHERE singleton_id = 1)")
            .fetch_one(self.database.pool())
            .await
            .map_err(Into::into)
    }

    pub(in crate::infra::user_repository) async fn is_installation_owner(&self, id: &UserId) -> StorageResult<bool> {
        query_scalar("SELECT EXISTS(SELECT 1 FROM sys_installation_owner WHERE singleton_id = 1 AND owner_user_id = $1)")
            .bind(&id.0)
            .fetch_one(self.database.pool())
            .await
            .map_err(Into::into)
    }

    pub(in crate::infra::user_repository) async fn create_installation_owner(&self, input: ReplaceUserRecord) -> StorageResult<Option<User>> {
        self.ensure_references(&input).await?;
        let mut transaction = self.database.pool().begin().await?;
        acquire_installation_owner_lock(&mut transaction).await?;
        if owner_id(&mut transaction).await?.is_some() {
            return Ok(None);
        }

        let user_id = self.database.next_id();
        insert_installation_owner(&mut transaction, &user_id, input).await?;
        query("INSERT INTO sys_installation_owner (singleton_id, owner_user_id) VALUES (1, $1)")
            .bind(&user_id)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await.map_err(StorageError::from)?;
        let user = self.find_by_id(UserId(user_id)).await?.ok_or(StorageError::NotFound)?;
        Ok(Some(user))
    }
}

async fn acquire_installation_owner_lock(transaction: &mut Transaction<'_, Postgres>) -> StorageResult<()> {
    query("SELECT pg_advisory_xact_lock(hashtext('taco.installation_owner'))")
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

async fn owner_id(transaction: &mut Transaction<'_, Postgres>) -> StorageResult<Option<String>> {
    query_scalar("SELECT owner_user_id FROM sys_installation_owner WHERE singleton_id = 1 FOR UPDATE")
        .fetch_optional(&mut **transaction)
        .await
        .map_err(Into::into)
}

async fn insert_installation_owner(transaction: &mut Transaction<'_, Postgres>, user_id: &str, mut input: ReplaceUserRecord) -> StorageResult<()> {
    input.role_ids.clear();
    input.post_ids.clear();
    let password_hash = required_password(input.password_hash.take())?;
    insert_user_in_transaction(transaction, InsertUserCommand { user_id, input, password_hash }).await
}
