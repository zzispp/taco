use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;

use crate::{
    application::{AppResult, ReplaceUserRecord, UserRepository},
    domain::{AvatarFileId, ProfileUpdate, User, UserId},
};

/// An account password change coupled with the immutable operation record that
/// must be written in the same transaction as the new password hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditedPasswordChange {
    pub user_id: UserId,
    pub old_password: String,
    pub new_password: String,
    pub audit: AuditOutboxRecord,
}

/// A validated import mutation. The application layer decides whether a row
/// creates or replaces a user; the repository applies the whole batch in one
/// transaction together with its operation record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UserImportWrite {
    Create(ReplaceUserRecord),
    Replace { id: UserId, user: ReplaceUserRecord },
}

/// Persists a successful user write and its immutable audit record in one PostgreSQL transaction.
///
/// This explicit port has no default implementation: callers must provide the complete audit
/// record, so an audited command can never silently degrade to an untracked write.
#[async_trait]
pub trait AuditedUserRepository: UserRepository {
    async fn create_with_audit(&self, user: ReplaceUserRecord, audit: &AuditOutboxRecord) -> AppResult<User>;
    async fn import_with_audit(&self, writes: Vec<UserImportWrite>, audit: &AuditOutboxRecord) -> AppResult<()>;
    async fn replace_with_audit(&self, id: UserId, user: ReplaceUserRecord, audit: &AuditOutboxRecord) -> AppResult<User>;
    async fn delete_with_audit(&self, id: UserId, audit: &AuditOutboxRecord) -> AppResult<()>;
    async fn delete_many_with_audit(&self, ids: Vec<UserId>, audit: &AuditOutboxRecord) -> AppResult<()>;
    async fn record_login_with_audit(&self, id: UserId, ipaddr: String, audit: &AuditOutboxRecord) -> AppResult<()>;
    async fn update_password_with_audit(&self, id: UserId, password_hash: String, audit: &AuditOutboxRecord) -> AppResult<()>;
    async fn update_profile_with_audit(&self, id: UserId, profile: ProfileUpdate, audit: &AuditOutboxRecord) -> AppResult<User>;
    async fn update_avatar_with_audit(&self, id: UserId, avatar: AvatarFileId, audit: &AuditOutboxRecord) -> AppResult<User>;
    async fn update_status_with_audit(&self, id: UserId, status: String, audit: &AuditOutboxRecord) -> AppResult<User>;
    async fn replace_roles_with_audit(&self, id: UserId, role_ids: Vec<String>, audit: &AuditOutboxRecord) -> AppResult<User>;
}
