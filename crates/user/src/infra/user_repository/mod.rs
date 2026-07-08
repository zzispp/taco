mod filter_sql;
mod mapping;
mod queries;
mod record;
mod sql;
mod write;

use async_trait::async_trait;
use kernel::pagination::{Page, PageSliceRequest};
use storage::Database;

use crate::{
    application::{AppResult, ReplaceUserRecord, UserAuthRecord, UserListFilter, UserRepository},
    domain::{ProfileUpdate, User, UserId, UserProfileGroups},
};
use types::rbac::DataScopeFilter;

use self::{
    mapping::{storage_error, user_auth_record},
    queries::UserQueries,
};

#[derive(Clone)]
pub struct StorageUserRepository {
    queries: UserQueries,
}

impl StorageUserRepository {
    pub fn new(database: Database) -> Self {
        Self {
            queries: UserQueries::new(database),
        }
    }
}

#[async_trait]
impl UserRepository for StorageUserRepository {
    async fn create(&self, user: ReplaceUserRecord) -> AppResult<User> {
        self.queries.create(user).await.map_err(storage_error)
    }

    async fn replace(&self, id: UserId, user: ReplaceUserRecord) -> AppResult<User> {
        self.queries.replace(id, user).await.map_err(storage_error)
    }

    async fn delete(&self, id: UserId) -> AppResult<()> {
        self.queries.delete(id).await.map_err(storage_error)
    }

    async fn delete_many(&self, ids: Vec<UserId>) -> AppResult<()> {
        self.queries.delete_many(ids).await.map_err(storage_error)
    }

    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>> {
        self.queries.find_by_id(id).await.map_err(storage_error)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        self.queries.find_by_email(email).await.map_err(storage_error)
    }

    async fn find_by_phone(&self, phone: &str) -> AppResult<Option<User>> {
        self.queries.find_by_phone(phone).await.map_err(storage_error)
    }

    async fn find_auth_by_username(&self, username: &str) -> AppResult<Option<UserAuthRecord>> {
        self.queries
            .find_auth_by_username(username)
            .await
            .map(|record| record.map(user_auth_record))
            .map_err(storage_error)
    }

    async fn find_auth_by_email(&self, email: &str) -> AppResult<Option<UserAuthRecord>> {
        self.queries
            .find_auth_by_email(email)
            .await
            .map(|record| record.map(user_auth_record))
            .map_err(storage_error)
    }

    async fn find_auth_by_id(&self, id: UserId) -> AppResult<Option<UserAuthRecord>> {
        self.queries
            .find_auth_by_id(id)
            .await
            .map(|record| record.map(user_auth_record))
            .map_err(storage_error)
    }

    async fn record_login(&self, id: UserId) -> AppResult<()> {
        self.queries.record_login(id).await.map_err(storage_error)
    }

    async fn list(&self, filter: UserListFilter) -> AppResult<Page<User>> {
        self.queries.list(filter).await.map_err(storage_error)
    }

    async fn list_scoped(&self, filter: UserListFilter, scope: DataScopeFilter) -> AppResult<Page<User>> {
        self.queries.list_scoped(filter, scope).await.map_err(storage_error)
    }

    async fn list_scoped_ids(&self, ids: Vec<UserId>, scope: DataScopeFilter) -> AppResult<Vec<UserId>> {
        self.queries
            .scoped_existing_user_ids(ids.into_iter().map(|id| id.0).collect(), &scope)
            .await
            .map(|ids| ids.into_iter().map(UserId).collect())
            .map_err(storage_error)
    }

    async fn list_slice(&self, filter: UserListFilter, request: PageSliceRequest) -> AppResult<Page<User>> {
        self.queries.list_slice(filter, request).await.map_err(storage_error)
    }

    async fn update_password(&self, id: UserId, password_hash: String) -> AppResult<()> {
        self.queries.update_password(id, password_hash).await.map_err(storage_error)
    }

    async fn update_profile(&self, id: UserId, profile: ProfileUpdate) -> AppResult<User> {
        self.queries.update_profile(id, profile).await.map_err(storage_error)
    }

    async fn update_avatar(&self, id: UserId, avatar: String) -> AppResult<User> {
        self.queries.update_avatar(id, avatar).await.map_err(storage_error)
    }

    async fn update_status(&self, id: UserId, status: String) -> AppResult<User> {
        self.queries.update_status(id, status).await.map_err(storage_error)
    }

    async fn replace_roles(&self, id: UserId, role_ids: Vec<String>) -> AppResult<User> {
        self.queries.replace_roles(id, role_ids).await.map_err(storage_error)
    }

    async fn profile_groups(&self, id: UserId) -> AppResult<UserProfileGroups> {
        self.queries.profile_groups(id).await.map_err(storage_error)
    }

    async fn form_options(&self) -> AppResult<crate::domain::UserFormOptions> {
        self.queries.form_options().await.map_err(storage_error)
    }
}
