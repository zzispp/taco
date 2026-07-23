use crate::{
    application::{ReplaceUserRecord, UserImportWrite},
    domain::{AvatarFileId, ProfileUpdate, User, UserId},
    infra::user_repository::write,
};
use audit_contract::{AuditOutboxEvent, AuditOutboxRecord};
use constants::system::STATUS_NORMAL;
use sqlx::{Postgres, Transaction, query};
use storage::{StorageError, StorageResult, outbox::append_audit_record};
use time::OffsetDateTime;

use super::{
    UserQueries,
    admin_continuity::{acquire_admin_continuity_lock, ensure_enabled_system_administrator_remains},
    cleanup::delete_user_relations,
    ensure_batch_rows,
    write_support::{InsertUserCommand, insert_user_in_transaction, required_password, revoke_user_sessions, update_user_in_transaction},
};

impl UserQueries {
    pub(in crate::infra::user_repository) async fn create_with_audit(&self, input: ReplaceUserRecord, audit: &AuditOutboxRecord) -> StorageResult<User> {
        self.ensure_references(&input).await?;
        let user_id = self.database.next_id();
        let password_hash = required_password(input.password_hash.clone())?;
        let mut transaction = self.database.pool().begin().await?;
        insert_user_in_transaction(
            &mut transaction,
            InsertUserCommand {
                user_id: &user_id,
                input,
                password_hash,
            },
        )
        .await?;
        commit_audited_write(transaction, &created_user_audit(audit, &user_id)).await?;
        self.find_by_id(UserId(user_id)).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra::user_repository) async fn import_with_audit(&self, writes: Vec<UserImportWrite>, audit: &AuditOutboxRecord) -> StorageResult<()> {
        for write in &writes {
            self.ensure_references(import_record(write)).await?;
        }

        let mut transaction = self.database.pool().begin().await?;
        acquire_admin_continuity_lock(&mut transaction).await?;
        for write in writes {
            match write {
                UserImportWrite::Create(input) => {
                    let user_id = self.database.next_id();
                    let password_hash = required_password(input.password_hash.clone())?;
                    insert_user_in_transaction(
                        &mut transaction,
                        InsertUserCommand {
                            user_id: &user_id,
                            input,
                            password_hash,
                        },
                    )
                    .await?;
                }
                UserImportWrite::Replace { id, user } => {
                    update_user_in_transaction(&mut transaction, &id, user).await?;
                }
            }
        }
        ensure_enabled_system_administrator_remains(&mut transaction).await?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra::user_repository) async fn replace_with_audit(
        &self,
        id: UserId,
        input: ReplaceUserRecord,
        audit: &AuditOutboxRecord,
    ) -> StorageResult<User> {
        self.ensure_references(&input).await?;
        let mut transaction = self.database.pool().begin().await?;
        acquire_admin_continuity_lock(&mut transaction).await?;
        update_user_in_transaction(&mut transaction, &id, input).await?;
        ensure_enabled_system_administrator_remains(&mut transaction).await?;
        commit_audited_write(transaction, audit).await?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra::user_repository) async fn delete_with_audit(&self, id: UserId, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let user_id = id.0;
        let mut transaction = self.database.pool().begin().await?;
        acquire_admin_continuity_lock(&mut transaction).await?;
        let result = query("UPDATE sys_user SET del_flag = '2', update_time = $2 WHERE user_id = $1 AND del_flag = '0'")
            .bind(&user_id)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut *transaction)
            .await?;
        write::ensure_rows_affected(result.rows_affected())?;
        delete_user_relations(&mut transaction, std::slice::from_ref(&user_id)).await?;
        ensure_enabled_system_administrator_remains(&mut transaction).await?;
        revoke_user_sessions(&mut transaction, &[user_id]).await?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra::user_repository) async fn delete_many_with_audit(&self, ids: Vec<UserId>, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let ids = ids.into_iter().map(|id| id.0).collect::<Vec<_>>();
        let mut transaction = self.database.pool().begin().await?;
        acquire_admin_continuity_lock(&mut transaction).await?;
        let result = query("UPDATE sys_user SET del_flag = '2', update_time = $2 WHERE user_id = ANY($1) AND del_flag = '0'")
            .bind(&ids)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut *transaction)
            .await?;
        ensure_batch_rows(result.rows_affected(), ids.len())?;
        delete_user_relations(&mut transaction, &ids).await?;
        ensure_enabled_system_administrator_remains(&mut transaction).await?;
        revoke_user_sessions(&mut transaction, &ids).await?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra::user_repository) async fn record_login_with_audit(&self, id: UserId, ipaddr: String, audit: &AuditOutboxRecord) -> StorageResult<()> {
        let now = OffsetDateTime::now_utc();
        let mut transaction = self.database.pool().begin().await?;
        let result = query("UPDATE sys_user SET login_ip = $2, login_date = $3, update_time = $3 WHERE user_id = $1 AND del_flag = '0'")
            .bind(id.0)
            .bind(ipaddr)
            .bind(now)
            .execute(&mut *transaction)
            .await?;
        write::ensure_rows_affected(result.rows_affected())?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra::user_repository) async fn update_password_with_audit(
        &self,
        id: UserId,
        password_hash: String,
        audit: &AuditOutboxRecord,
    ) -> StorageResult<()> {
        let user_id = id.0;
        let mut transaction = self.database.pool().begin().await?;
        let result = query("UPDATE sys_user SET password=$2,pwd_update_date=CURRENT_TIMESTAMP,update_time=CURRENT_TIMESTAMP WHERE user_id=$1 AND del_flag='0'")
            .bind(&user_id)
            .bind(password_hash)
            .execute(&mut *transaction)
            .await?;
        write::ensure_rows_affected(result.rows_affected())?;
        revoke_user_sessions(&mut transaction, &[user_id]).await?;
        commit_audited_write(transaction, audit).await
    }

    pub(in crate::infra::user_repository) async fn update_profile_with_audit(
        &self,
        id: UserId,
        profile: ProfileUpdate,
        audit: &AuditOutboxRecord,
    ) -> StorageResult<User> {
        let mut transaction = self.database.pool().begin().await?;
        let result = query("UPDATE sys_user SET nick_name=$2,email=$3,phonenumber=$4,sex=$5,update_time=CURRENT_TIMESTAMP WHERE user_id=$1 AND del_flag='0'")
            .bind(&id.0)
            .bind(profile.nick_name)
            .bind(profile.email)
            .bind(profile.phonenumber)
            .bind(profile.sex)
            .execute(&mut *transaction)
            .await?;
        write::ensure_rows_affected(result.rows_affected())?;
        commit_audited_write(transaction, audit).await?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra::user_repository) async fn update_avatar_with_audit(
        &self,
        id: UserId,
        avatar: AvatarFileId,
        audit: &AuditOutboxRecord,
    ) -> StorageResult<User> {
        let mut transaction = self.database.pool().begin().await?;
        let result =
            query("UPDATE sys_user SET avatar_file_id=$2,avatar_version=avatar_version+1,update_time=CURRENT_TIMESTAMP WHERE user_id=$1 AND del_flag='0'")
                .bind(&id.0)
                .bind(avatar.as_str())
                .execute(&mut *transaction)
                .await?;
        write::ensure_rows_affected(result.rows_affected())?;
        commit_audited_write(transaction, audit).await?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra::user_repository) async fn update_status_with_audit(
        &self,
        id: UserId,
        status: String,
        audit: &AuditOutboxRecord,
    ) -> StorageResult<User> {
        let revoke_sessions = status != STATUS_NORMAL;
        let mut transaction = self.database.pool().begin().await?;
        acquire_admin_continuity_lock(&mut transaction).await?;
        let result = query("UPDATE sys_user SET status=$2,update_time=CURRENT_TIMESTAMP WHERE user_id=$1 AND del_flag='0'")
            .bind(&id.0)
            .bind(status)
            .execute(&mut *transaction)
            .await?;
        write::ensure_rows_affected(result.rows_affected())?;
        ensure_enabled_system_administrator_remains(&mut transaction).await?;
        if revoke_sessions {
            revoke_user_sessions(&mut transaction, std::slice::from_ref(&id.0)).await?;
        }
        commit_audited_write(transaction, audit).await?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }

    pub(in crate::infra::user_repository) async fn replace_roles_with_audit(
        &self,
        id: UserId,
        role_ids: Vec<String>,
        audit: &AuditOutboxRecord,
    ) -> StorageResult<User> {
        write::ensure_ids_exist(self.database.pool(), write::ReferenceTable::role(), &role_ids).await?;
        let mut transaction = self.database.pool().begin().await?;
        acquire_admin_continuity_lock(&mut transaction).await?;
        write::replace_roles(&mut transaction, &id.0, role_ids).await?;
        ensure_enabled_system_administrator_remains(&mut transaction).await?;
        commit_audited_write(transaction, audit).await?;
        self.find_by_id(id).await?.ok_or(StorageError::NotFound)
    }
}

fn import_record(write: &UserImportWrite) -> &ReplaceUserRecord {
    match write {
        UserImportWrite::Create(user) => user,
        UserImportWrite::Replace { user, .. } => user,
    }
}

fn created_user_audit(audit: &AuditOutboxRecord, user_id: &str) -> AuditOutboxRecord {
    let mut audit = audit.clone();
    if let AuditOutboxEvent::Security(event) = &mut audit.event {
        event.user_id = Some(user_id.into());
    }
    audit
}

async fn commit_audited_write(mut transaction: Transaction<'_, Postgres>, audit: &AuditOutboxRecord) -> StorageResult<()> {
    match append_audit_record(&mut transaction, audit).await {
        Ok(()) => transaction.commit().await.map_err(StorageError::from),
        Err(error) => {
            transaction.rollback().await.map_err(StorageError::from)?;
            Err(error)
        }
    }
}
