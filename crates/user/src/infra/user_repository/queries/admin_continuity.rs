use constants::system::{ADMIN_ROLE_CONTINUITY_LOCK_KEY, ADMIN_ROLE_KEY, DEL_FLAG_ACTIVE, LAST_ENABLED_ADMIN_REQUIRED_CONFLICT, STATUS_NORMAL};
use sqlx::{Executor, Postgres, Transaction, query, query_scalar};
use storage::{StorageError, StorageResult};

const ENABLED_SYSTEM_ADMINISTRATOR_EXISTS_QUERY: &str = r#"
    SELECT EXISTS(
        SELECT 1
        FROM sys_user user_account
        INNER JOIN sys_user_role user_role ON user_role.user_id = user_account.user_id
        INNER JOIN sys_role role ON role.role_id = user_role.role_id
        WHERE role.role_key = $1
          AND role.system = TRUE
          AND role.status = $2
          AND role.del_flag = $3
          AND user_account.status = $2
          AND user_account.del_flag = $3
    )
"#;

pub(super) async fn acquire_admin_continuity_lock(transaction: &mut Transaction<'_, Postgres>) -> StorageResult<()> {
    query("SELECT pg_advisory_xact_lock(hashtext($1))")
        .bind(ADMIN_ROLE_CONTINUITY_LOCK_KEY)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

pub(super) async fn ensure_enabled_system_administrator_remains(transaction: &mut Transaction<'_, Postgres>) -> StorageResult<()> {
    if enabled_system_administrator_exists(&mut **transaction).await? {
        return Ok(());
    }
    Err(StorageError::Conflict(LAST_ENABLED_ADMIN_REQUIRED_CONFLICT.into()))
}

pub(super) async fn enabled_system_administrator_exists<'e>(executor: impl Executor<'e, Database = Postgres>) -> StorageResult<bool> {
    query_scalar(ENABLED_SYSTEM_ADMINISTRATOR_EXISTS_QUERY)
        .bind(ADMIN_ROLE_KEY)
        .bind(STATUS_NORMAL)
        .bind(DEL_FLAG_ACTIVE)
        .fetch_one(executor)
        .await
        .map_err(StorageError::from)
}
