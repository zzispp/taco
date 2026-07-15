use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;

use crate::{
    application::{AppResult, AuditedUserRepository, ReplaceUserRecord, UserImportWrite},
    domain::{ProfileUpdate, User, UserId},
};

use super::{StorageUserRepository, mapping::storage_error};

#[async_trait]
impl AuditedUserRepository for StorageUserRepository {
    async fn create_with_audit(&self, user: ReplaceUserRecord, audit: &AuditOutboxRecord) -> AppResult<User> {
        self.queries.create_with_audit(user, audit).await.map_err(storage_error)
    }

    async fn import_with_audit(&self, writes: Vec<UserImportWrite>, audit: &AuditOutboxRecord) -> AppResult<()> {
        self.queries.import_with_audit(writes, audit).await.map_err(storage_error)
    }

    async fn replace_with_audit(&self, id: UserId, user: ReplaceUserRecord, audit: &AuditOutboxRecord) -> AppResult<User> {
        self.queries.replace_with_audit(id, user, audit).await.map_err(storage_error)
    }

    async fn delete_with_audit(&self, id: UserId, audit: &AuditOutboxRecord) -> AppResult<()> {
        self.queries.delete_with_audit(id, audit).await.map_err(storage_error)
    }

    async fn delete_many_with_audit(&self, ids: Vec<UserId>, audit: &AuditOutboxRecord) -> AppResult<()> {
        self.queries.delete_many_with_audit(ids, audit).await.map_err(storage_error)
    }

    async fn record_login_with_audit(&self, id: UserId, ipaddr: String, audit: &AuditOutboxRecord) -> AppResult<()> {
        self.queries.record_login_with_audit(id, ipaddr, audit).await.map_err(storage_error)
    }

    async fn update_password_with_audit(&self, id: UserId, password_hash: String, audit: &AuditOutboxRecord) -> AppResult<()> {
        self.queries.update_password_with_audit(id, password_hash, audit).await.map_err(storage_error)
    }

    async fn update_profile_with_audit(&self, id: UserId, profile: ProfileUpdate, audit: &AuditOutboxRecord) -> AppResult<User> {
        self.queries.update_profile_with_audit(id, profile, audit).await.map_err(storage_error)
    }

    async fn update_avatar_with_audit(&self, id: UserId, avatar: String, audit: &AuditOutboxRecord) -> AppResult<User> {
        self.queries.update_avatar_with_audit(id, avatar, audit).await.map_err(storage_error)
    }

    async fn update_status_with_audit(&self, id: UserId, status: String, audit: &AuditOutboxRecord) -> AppResult<User> {
        self.queries.update_status_with_audit(id, status, audit).await.map_err(storage_error)
    }

    async fn replace_roles_with_audit(&self, id: UserId, role_ids: Vec<String>, audit: &AuditOutboxRecord) -> AppResult<User> {
        self.queries.replace_roles_with_audit(id, role_ids, audit).await.map_err(storage_error)
    }
}
