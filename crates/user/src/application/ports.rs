use super::{AppResult, AvatarConfig, LoginLockConfig, PasswordPolicy};
use crate::domain::{Credentials, NewUser, ProfileUpdate, ReplaceUser, User, UserFormOptions, UserId, UserProfile, UserProfileGroups};
use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use kernel::pagination::{CursorPage, CursorPageRequest};
use rbac::domain::DataScopeFilter;
use time::OffsetDateTime;

mod online_session;

pub use online_session::{OnlineSession, OnlineSessionCleanup, OnlineSessionFilter, OnlineSessionPageRequest, OnlineSessionSearch, OnlineSessionStore};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserListFilter {
    pub page: CursorPageRequest,
    pub username: Option<String>,
    pub nick_name: Option<String>,
    pub phonenumber: Option<String>,
    pub email: Option<String>,
    pub sex: Option<String>,
    pub status: Option<String>,
    pub dept_id: Option<String>,
    pub dept_name: Option<String>,
    pub post_ids: Vec<String>,
    pub role_ids: Vec<String>,
    pub begin_time: Option<OffsetDateTime>,
    pub end_time: Option<OffsetDateTime>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserExportRequest {
    pub filter: UserListFilter,
    pub scope: Option<DataScopeFilter>,
    pub batch_size: u64,
}

pub trait UserExportSink: Send {
    fn append(&mut self, users: &[User]) -> AppResult<()>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceUserRecord {
    pub username: String,
    pub password_hash: Option<String>,
    pub nick_name: String,
    pub dept_id: Option<String>,
    pub email: String,
    pub phonenumber: Option<String>,
    pub sex: String,
    pub status: String,
    pub remark: Option<String>,
    pub role_ids: Vec<String>,
    pub post_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserAuthRecord {
    pub user: User,
    pub password_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizationUser {
    pub id: UserId,
    pub username: String,
    pub dept_id: Option<String>,
    pub status: String,
    pub is_installation_owner: bool,
    pub role_keys: Vec<String>,
    pub permissions: Vec<String>,
}

impl AuthorizationUser {
    pub fn from_user(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            dept_id: user.dept_id,
            status: user.status,
            is_installation_owner: user.is_installation_owner,
            role_keys: user.roles.into_iter().map(|role| role.role_key).collect(),
            permissions: user.permissions,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedLogin {
    user: User,
}

impl VerifiedLogin {
    pub(crate) fn new(user: User) -> Self {
        Self { user }
    }

    pub fn user(&self) -> &User {
        &self.user
    }

    pub(crate) fn into_user(self) -> User {
        self.user
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserImportRow {
    pub dept_id: Option<String>,
    pub username: String,
    pub password: String,
    pub nick_name: String,
    pub email: String,
    pub phonenumber: Option<String>,
    pub sex: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserImportInput {
    pub rows: Vec<UserImportRow>,
    pub update_support: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserImportReport {
    pub success_count: usize,
    pub messages: Vec<UserImportMessage>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserImportMessage {
    pub key: &'static str,
    pub username: String,
}

impl UserImportMessage {
    pub fn new(key: &'static str, username: impl Into<String>) -> Self {
        Self {
            key,
            username: username.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarFile {
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

#[async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn create(&self, user: ReplaceUserRecord) -> AppResult<User>;
    async fn replace(&self, id: UserId, user: ReplaceUserRecord) -> AppResult<User>;
    async fn delete(&self, id: UserId) -> AppResult<()>;
    async fn delete_many(&self, ids: Vec<UserId>) -> AppResult<()>;
    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<User>>;
    async fn find_auth_by_username(&self, username: &str) -> AppResult<Option<UserAuthRecord>>;
    async fn find_auth_by_email(&self, email: &str) -> AppResult<Option<UserAuthRecord>>;
    async fn find_auth_by_id(&self, id: UserId) -> AppResult<Option<UserAuthRecord>>;
    async fn find_authorization_by_id(&self, id: UserId) -> AppResult<Option<AuthorizationUser>>;
    async fn is_installation_owner(&self, id: &UserId) -> AppResult<bool>;
    async fn record_login(&self, id: UserId, ipaddr: String) -> AppResult<()>;
    async fn list(&self, filter: UserListFilter) -> AppResult<CursorPage<User>>;
    async fn list_scoped(&self, filter: UserListFilter, scope: DataScopeFilter) -> AppResult<CursorPage<User>>;
    async fn list_scoped_ids(&self, ids: Vec<UserId>, scope: DataScopeFilter) -> AppResult<Vec<UserId>>;
    async fn export_users(&self, request: UserExportRequest, sink: &mut dyn UserExportSink) -> AppResult<()>;
    async fn update_password(&self, id: UserId, password_hash: String) -> AppResult<()>;
    async fn update_profile(&self, id: UserId, profile: ProfileUpdate) -> AppResult<User>;
    async fn update_avatar(&self, id: UserId, avatar: String) -> AppResult<User>;
    async fn update_status(&self, id: UserId, status: String) -> AppResult<User>;
    async fn replace_roles(&self, id: UserId, role_ids: Vec<String>) -> AppResult<User>;
    async fn profile_groups(&self, id: UserId) -> AppResult<UserProfileGroups>;
    async fn form_options(&self) -> AppResult<UserFormOptions>;
}

pub trait PasswordHasher: Send + Sync + 'static {
    fn hash(&self, password: &str) -> AppResult<String>;
    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool>;
}

#[async_trait]
pub trait AvatarStorage: Send + Sync + 'static {
    async fn store_avatar(&self, file: AvatarFile, max_bytes: usize) -> AppResult<String>;
}

#[async_trait]
pub trait AccountVerifier: Send + Sync + 'static {
    async fn verify_account(&self, token: Option<&str>) -> AppResult<()>;
}

#[async_trait]
pub trait SystemConfigProvider: Send + Sync + 'static {
    async fn config_by_key(&self, key: &str) -> AppResult<String>;
}

#[async_trait]
pub trait PasswordPolicyProvider: Send + Sync + 'static {
    async fn password_policy(&self) -> AppResult<PasswordPolicy>;
}

#[async_trait]
pub trait AvatarConfigProvider: Send + Sync + 'static {
    async fn avatar_config(&self) -> AppResult<AvatarConfig>;
}

/// Supplies the required login-lock policy to authentication application services.
///
/// Implementations must load and validate the single owning runtime parameter instead of
/// providing defaults. Missing, malformed, or unavailable configuration is returned as an
/// `AppError` and is not handled as a best-effort condition by callers.
#[async_trait]
pub trait LoginLockConfigProvider: Send + Sync + 'static {
    async fn login_lock_config(&self) -> AppResult<LoginLockConfig>;
}

/// Persists the failed-login counter used to enforce the login-lock policy.
///
/// Implementations must scope counters by [`UserId`] and make `record_failure` atomically
/// increment the counter while applying the supplied TTL. Storage failures propagate as an
/// `AppError`; authentication correctness depends on this port, so its operations are not
/// best-effort.
#[async_trait]
pub trait LoginFailureStore: Send + Sync + 'static {
    async fn failure_count(&self, user_id: &UserId) -> AppResult<u32>;
    async fn record_failure(&self, user_id: &UserId, ttl_seconds: u64) -> AppResult<u32>;
    async fn clear_failures(&self, user_id: &UserId) -> AppResult<()>;
}

#[async_trait]
pub trait UserUseCase: Send + Sync + 'static {
    async fn sign_up(&self, input: NewUser) -> AppResult<User>;
    async fn sign_up_with_audit(&self, input: NewUser, audit: AuditOutboxRecord) -> AppResult<User>;
    async fn sign_in(&self, input: Credentials) -> AppResult<VerifiedLogin>;
    async fn complete_sign_in(&self, login: VerifiedLogin, ipaddr: String) -> AppResult<User>;
    async fn complete_sign_in_with_audit(&self, login: VerifiedLogin, ipaddr: String, audit: AuditOutboxRecord) -> AppResult<User>;
    async fn unlock_login(&self, username: &str) -> AppResult<()>;
    async fn authenticated_user(&self, id: UserId) -> AppResult<User>;
    async fn authorization_user(&self, id: UserId) -> AppResult<AuthorizationUser>;
    async fn profile(&self, id: UserId) -> AppResult<UserProfile>;
    async fn update_profile(&self, id: UserId, profile: ProfileUpdate) -> AppResult<User>;
    async fn update_profile_with_audit(&self, id: UserId, profile: ProfileUpdate, audit: AuditOutboxRecord) -> AppResult<User>;
    async fn change_password(&self, id: UserId, old_password: String, new_password: String) -> AppResult<()>;
    async fn change_password_with_audit(&self, input: super::AuditedPasswordChange) -> AppResult<()>;
    async fn update_avatar(&self, id: UserId, avatar: String) -> AppResult<User>;
    async fn update_avatar_with_audit(&self, id: UserId, avatar: String, audit: AuditOutboxRecord) -> AppResult<User>;
    async fn create_user(&self, input: NewUser) -> AppResult<User>;
    async fn create_user_with_audit(&self, input: NewUser, audit: AuditOutboxRecord) -> AppResult<User>;
    async fn replace_user(&self, id: UserId, input: ReplaceUser) -> AppResult<User>;
    async fn replace_user_with_audit(&self, id: UserId, input: ReplaceUser, audit: AuditOutboxRecord) -> AppResult<User>;
    async fn delete_user(&self, id: UserId) -> AppResult<()>;
    async fn delete_user_with_audit(&self, id: UserId, audit: AuditOutboxRecord) -> AppResult<()>;
    async fn delete_users(&self, ids: Vec<UserId>) -> AppResult<()>;
    async fn delete_users_with_audit(&self, ids: Vec<UserId>, audit: AuditOutboxRecord) -> AppResult<()>;
    async fn get_user(&self, id: UserId) -> AppResult<User>;
    async fn reset_password(&self, id: UserId, password: String) -> AppResult<()>;
    async fn reset_password_with_audit(&self, id: UserId, password: String, audit: AuditOutboxRecord) -> AppResult<()>;
    async fn update_status(&self, id: UserId, status: String) -> AppResult<User>;
    async fn update_status_with_audit(&self, id: UserId, status: String, audit: AuditOutboxRecord) -> AppResult<User>;
    async fn replace_roles(&self, id: UserId, role_ids: Vec<String>) -> AppResult<User>;
    async fn replace_roles_with_audit(&self, id: UserId, role_ids: Vec<String>, audit: AuditOutboxRecord) -> AppResult<User>;
    async fn list_users(&self, filter: UserListFilter) -> AppResult<CursorPage<User>>;
    async fn list_users_scoped(&self, filter: UserListFilter, scope: DataScopeFilter) -> AppResult<CursorPage<User>>;
    async fn export_users(&self, request: UserExportRequest, sink: &mut dyn UserExportSink) -> AppResult<()>;
    async fn ensure_user_ids_scoped(&self, ids: Vec<UserId>, scope: DataScopeFilter) -> AppResult<()>;
    async fn filter_online_sessions_scoped(&self, sessions: Vec<OnlineSession>, scope: DataScopeFilter) -> AppResult<Vec<OnlineSession>>;
    async fn import_users_with_audit(&self, input: UserImportInput, audit: AuditOutboxRecord) -> AppResult<UserImportReport>;
    async fn form_options(&self) -> AppResult<UserFormOptions>;
}
