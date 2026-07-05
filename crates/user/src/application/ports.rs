use async_trait::async_trait;
use kernel::pagination::{Page, PageRequest, PageSliceRequest};

use super::AppResult;
use crate::domain::{Credentials, NewUser, ReplaceUser, User, UserFormOptions, UserId};
use types::rbac::DataScopeFilter;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserListFilter {
    pub page: PageRequest,
    pub username: Option<String>,
    pub phonenumber: Option<String>,
    pub status: Option<String>,
    pub dept_id: Option<String>,
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
    async fn record_login(&self, id: UserId) -> AppResult<()>;
    async fn list(&self, filter: UserListFilter) -> AppResult<Page<User>>;
    async fn list_scoped(&self, filter: UserListFilter, scope: DataScopeFilter) -> AppResult<Page<User>>;
    async fn list_slice(&self, filter: UserListFilter, request: PageSliceRequest) -> AppResult<Page<User>>;
    async fn update_password(&self, id: UserId, password_hash: String) -> AppResult<()>;
    async fn update_status(&self, id: UserId, status: String) -> AppResult<User>;
    async fn replace_roles(&self, id: UserId, role_ids: Vec<String>) -> AppResult<User>;
    async fn form_options(&self) -> AppResult<UserFormOptions>;
}

pub trait PasswordHasher: Send + Sync + 'static {
    fn hash(&self, password: &str) -> AppResult<String>;
    fn verify(&self, password: &str, password_hash: &str) -> AppResult<bool>;
}

#[async_trait]
pub trait UserUseCase: Send + Sync + 'static {
    async fn sign_up(&self, input: NewUser) -> AppResult<User>;
    async fn sign_in(&self, input: Credentials) -> AppResult<User>;
    async fn authenticated_user(&self, id: UserId) -> AppResult<User>;
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
    async fn form_options(&self) -> AppResult<UserFormOptions>;
}
