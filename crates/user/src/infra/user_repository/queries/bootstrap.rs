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

pub(in crate::infra::user_repository) enum BootstrapAdminOutcome {
    Created(Box<User>),
    ExistingSuperAdmin,
    MissingAdminRole,
}

impl UserQueries {
    pub(in crate::infra::user_repository) async fn bootstrap_admin(&self, mut input: ReplaceUserRecord) -> StorageResult<BootstrapAdminOutcome> {
        let mut transaction = self.database.pool().begin().await?;
        acquire_bootstrap_lock(&mut transaction).await?;
        if super_admin_exists(&mut transaction).await? {
            return Ok(BootstrapAdminOutcome::ExistingSuperAdmin);
        }
        let Some(role_id) = active_admin_role_id(&mut transaction).await? else {
            return Ok(BootstrapAdminOutcome::MissingAdminRole);
        };
        input.role_ids = vec![role_id];
        let password_hash = required_password(input.password_hash.take())?;
        let user_id = self.database.next_id();
        insert_user_in_transaction(
            &mut transaction,
            InsertUserCommand {
                user_id: &user_id,
                input,
                password_hash,
            },
        )
        .await?;
        transaction.commit().await?;
        let user = self.find_by_id(UserId(user_id)).await?.ok_or(StorageError::NotFound)?;
        Ok(BootstrapAdminOutcome::Created(Box::new(user)))
    }
}

async fn acquire_bootstrap_lock(transaction: &mut Transaction<'_, Postgres>) -> StorageResult<()> {
    query("SELECT pg_advisory_xact_lock(hashtext('taco.bootstrap_admin'))")
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

async fn super_admin_exists(transaction: &mut Transaction<'_, Postgres>) -> StorageResult<bool> {
    query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM sys_user u
            WHERE u.del_flag = '0'
              AND EXISTS (
                SELECT 1
                FROM sys_user_role ur
                INNER JOIN sys_role r ON r.role_id = ur.role_id
                WHERE ur.user_id = u.user_id
                  AND r.role_key = $1
              )
        )
        "#,
    )
    .bind(constants::system::SUPER_ADMIN_ROLE_KEY)
    .fetch_one(&mut **transaction)
    .await
    .map_err(Into::into)
}

async fn active_admin_role_id(transaction: &mut Transaction<'_, Postgres>) -> StorageResult<Option<String>> {
    query_scalar("SELECT role_id FROM sys_role WHERE role_key = $1 AND status = $2 AND del_flag = '0'")
        .bind(constants::system::SUPER_ADMIN_ROLE_KEY)
        .bind(constants::system::STATUS_NORMAL)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(Into::into)
}
