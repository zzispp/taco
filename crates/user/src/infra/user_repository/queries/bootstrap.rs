use constants::system::{ADMIN_ROLE_KEY, DEL_FLAG_ACTIVE, STATUS_NORMAL};
use sqlx::{Postgres, Transaction, query, query_scalar};
use storage::{StorageError, StorageResult};

use crate::application::{BootstrapAdministratorOutcome, BootstrapAdministratorRecord};

use super::{
    UserQueries,
    admin_continuity::{acquire_admin_continuity_lock, enabled_system_administrator_exists},
};

const SYSTEM_ADMINISTRATOR_ROLE_QUERY: &str = r#"
    SELECT role_id
    FROM sys_role
    WHERE role_key = $1 AND system = TRUE AND status = $2 AND del_flag = $3
"#;

const INSERT_BOOTSTRAP_ADMINISTRATOR: &str = r#"
    INSERT INTO sys_user (user_id,user_name,nick_name,email,password,status,del_flag,create_by,create_time)
    VALUES ($1,$2,$3,$4,$5,$6,$7,$2,CURRENT_TIMESTAMP)
"#;

const SYSTEM_ADMINISTRATOR_ROLE_MISSING: &str = "enabled system administrator role is missing";

impl UserQueries {
    pub async fn has_enabled_system_administrator(&self) -> StorageResult<bool> {
        enabled_system_administrator_exists(self.database.pool()).await
    }

    pub async fn create_system_administrator_if_absent(&self, record: BootstrapAdministratorRecord) -> StorageResult<BootstrapAdministratorOutcome> {
        let user_id = self.database.next_id();
        let mut transaction = self.database.pool().begin().await?;
        acquire_admin_continuity_lock(&mut transaction).await?;
        if enabled_system_administrator_exists(&mut *transaction).await? {
            transaction.commit().await?;
            return Ok(BootstrapAdministratorOutcome::AlreadyPresent);
        }

        let role_id = enabled_system_administrator_role_id(&mut transaction).await?;
        insert_system_administrator(&mut transaction, &user_id, record).await?;
        bind_system_administrator_role(&mut transaction, &user_id, &role_id).await?;
        transaction.commit().await?;
        Ok(BootstrapAdministratorOutcome::Created)
    }
}

async fn enabled_system_administrator_role_id(transaction: &mut Transaction<'_, Postgres>) -> StorageResult<String> {
    query_scalar(SYSTEM_ADMINISTRATOR_ROLE_QUERY)
        .bind(ADMIN_ROLE_KEY)
        .bind(STATUS_NORMAL)
        .bind(DEL_FLAG_ACTIVE)
        .fetch_optional(&mut **transaction)
        .await?
        .ok_or_else(|| StorageError::Database(SYSTEM_ADMINISTRATOR_ROLE_MISSING.into()))
}

async fn insert_system_administrator(transaction: &mut Transaction<'_, Postgres>, user_id: &str, record: BootstrapAdministratorRecord) -> StorageResult<()> {
    query(INSERT_BOOTSTRAP_ADMINISTRATOR)
        .bind(user_id)
        .bind(&record.username)
        .bind(record.nick_name)
        .bind(record.email)
        .bind(record.password_hash)
        .bind(STATUS_NORMAL)
        .bind(DEL_FLAG_ACTIVE)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

async fn bind_system_administrator_role(transaction: &mut Transaction<'_, Postgres>, user_id: &str, role_id: &str) -> StorageResult<()> {
    query("INSERT INTO sys_user_role (user_id,role_id) VALUES ($1,$2)")
        .bind(user_id)
        .bind(role_id)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}
