use async_trait::async_trait;
use kernel::pagination::{Page, PageRequest, PageSliceRequest};
use serde::{Deserialize, Serialize};
use types::http::Locale;

use super::{AppResult, AvatarConfig, IpLocationConfig, PasswordPolicy};
use crate::domain::{Credentials, NewUser, ProfileUpdate, ReplaceUser, User, UserFormOptions, UserId, UserProfile, UserProfileGroups};
use types::rbac::DataScopeFilter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserListFilter {
    pub page: PageRequest,
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
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
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
pub struct SystemUserRecord {
    pub user: User,
    pub password_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserImportRow {
    pub dept_id: Option<String>,
    pub username: String,
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
    pub default_password: String,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnlineSession {
    pub token_id: String,
    pub user_id: UserId,
    pub dept_name: Option<String>,
    pub user_name: String,
    pub ipaddr: String,
    pub login_location: String,
    pub browser: String,
    pub os: String,
    pub login_time: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OnlineSessionFilter {
    pub ipaddr: Option<String>,
    pub user_name: Option<String>,
    pub login_location: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AvatarFile {
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub bytes: Vec<u8>,
}

pub trait SystemUserProvider: Send + Sync + 'static {
    fn system_user(&self) -> Option<SystemUserRecord>;
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
    async fn record_login(&self, id: UserId) -> AppResult<()>;
    async fn list(&self, filter: UserListFilter) -> AppResult<Page<User>>;
    async fn list_scoped(&self, filter: UserListFilter, scope: DataScopeFilter) -> AppResult<Page<User>>;
    async fn list_scoped_ids(&self, ids: Vec<UserId>, scope: DataScopeFilter) -> AppResult<Vec<UserId>>;
    async fn list_slice(&self, filter: UserListFilter, request: PageSliceRequest) -> AppResult<Page<User>>;
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
pub trait PublicIpResolver: Send + Sync + 'static {
    async fn resolve_public_ip(&self) -> AppResult<String>;
}

#[async_trait]
pub trait IpLocationSettingsReader: Send + Sync + 'static {
    async fn ip_location_config(&self) -> AppResult<IpLocationConfig>;
}

#[async_trait]
pub trait IpLocationResolver: Send + Sync + 'static {
    async fn resolve_login_location(&self, ipaddr: &str, locale: Locale) -> AppResult<String>;
}

#[async_trait]
pub trait SystemConfigProvider: Send + Sync + 'static {
    async fn config_by_key(&self, key: &str) -> AppResult<String>;
}

#[async_trait]
pub trait OnlineSessionStore: Send + Sync + 'static {
    async fn save(&self, session: &OnlineSession, ttl_seconds: u64) -> AppResult<()>;
    async fn find(&self, token_id: &str) -> AppResult<Option<OnlineSession>>;
    async fn delete(&self, token_id: &str) -> AppResult<()>;
    async fn list(&self) -> AppResult<Vec<OnlineSession>>;
}

#[async_trait]
pub trait PasswordPolicyProvider: Send + Sync + 'static {
    async fn password_policy(&self) -> AppResult<PasswordPolicy>;
}

#[async_trait]
pub trait AvatarConfigProvider: Send + Sync + 'static {
    async fn avatar_config(&self) -> AppResult<AvatarConfig>;
}

#[async_trait]
pub trait UserUseCase: Send + Sync + 'static {
    async fn sign_up(&self, input: NewUser) -> AppResult<User>;
    async fn sign_in(&self, input: Credentials) -> AppResult<User>;
    async fn authenticated_user(&self, id: UserId) -> AppResult<User>;
    async fn profile(&self, id: UserId) -> AppResult<UserProfile>;
    async fn update_profile(&self, id: UserId, profile: ProfileUpdate) -> AppResult<User>;
    async fn change_password(&self, id: UserId, old_password: String, new_password: String) -> AppResult<()>;
    async fn update_avatar(&self, id: UserId, avatar: String) -> AppResult<User>;
    async fn create_user(&self, input: NewUser) -> AppResult<User>;
    async fn replace_user(&self, id: UserId, input: ReplaceUser) -> AppResult<User>;
    async fn delete_user(&self, id: UserId) -> AppResult<()>;
    async fn delete_users(&self, ids: Vec<UserId>) -> AppResult<()>;
    async fn get_user(&self, id: UserId) -> AppResult<User>;
    async fn reset_password(&self, id: UserId, password: String) -> AppResult<()>;
    async fn update_status(&self, id: UserId, status: String) -> AppResult<User>;
    async fn replace_roles(&self, id: UserId, role_ids: Vec<String>) -> AppResult<User>;
    async fn list_users(&self, filter: UserListFilter) -> AppResult<Page<User>>;
    async fn list_users_scoped(&self, filter: UserListFilter, scope: DataScopeFilter) -> AppResult<Page<User>>;
    async fn ensure_user_ids_scoped(&self, ids: Vec<UserId>, scope: DataScopeFilter) -> AppResult<()>;
    async fn filter_online_sessions_scoped(&self, sessions: Vec<OnlineSession>, scope: DataScopeFilter) -> AppResult<Vec<OnlineSession>>;
    async fn import_users(&self, input: UserImportInput) -> AppResult<UserImportReport>;
    async fn form_options(&self) -> AppResult<UserFormOptions>;
}
