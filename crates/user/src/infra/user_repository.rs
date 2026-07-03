use async_trait::async_trait;
use kernel::pagination::{Page, PageRequest, PageSliceRequest};
use storage::{Database, StorageError};

use crate::{
    application::{AppError, AppResult, ReplaceUserRecord, UserAuthRecord, UserRepository},
    domain::{User, UserId},
};

use super::persistence::{UserAuthRecord as StorageUserAuthRecord, UserRecordInput as StorageUserRecordInput, UserStore};

#[derive(Clone)]
pub struct StorageUserRepository {
    store: UserStore,
}

impl StorageUserRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: UserStore::new(database),
        }
    }
}

#[async_trait]
impl UserRepository for StorageUserRepository {
    async fn create(&self, user: ReplaceUserRecord) -> AppResult<User> {
        self.store
            .create(storage_record_input(user))
            .await
            .map(user_from_storage)
            .map_err(storage_error)
    }

    async fn replace(&self, id: UserId, user: ReplaceUserRecord) -> AppResult<User> {
        self.store
            .replace(id.into(), storage_record_input(user))
            .await
            .map(user_from_storage)
            .map_err(storage_error)
    }

    async fn delete(&self, id: UserId) -> AppResult<()> {
        self.store.delete(id.into()).await.map_err(storage_error)
    }

    async fn find_by_id(&self, id: UserId) -> AppResult<Option<User>> {
        self.store
            .find_by_id(id.into())
            .await
            .map(|record| record.map(user_from_storage))
            .map_err(storage_error)
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        self.store
            .find_by_email(email)
            .await
            .map(|record| record.map(user_from_storage))
            .map_err(storage_error)
    }

    async fn find_auth_by_username(&self, username: &str) -> AppResult<Option<UserAuthRecord>> {
        self.store
            .find_auth_by_username(username)
            .await
            .map(|record| record.map(user_auth_record))
            .map_err(storage_error)
    }

    async fn find_auth_by_email(&self, email: &str) -> AppResult<Option<UserAuthRecord>> {
        self.store
            .find_auth_by_email(email)
            .await
            .map(|record| record.map(user_auth_record))
            .map_err(storage_error)
    }

    async fn record_login(&self, id: UserId) -> AppResult<()> {
        self.store.record_login(id.into()).await.map_err(storage_error)
    }

    async fn list(&self, page: PageRequest) -> AppResult<Page<User>> {
        self.store.list(page).await.map(page_from_storage).map_err(storage_error)
    }

    async fn list_slice(&self, request: PageSliceRequest) -> AppResult<Page<User>> {
        self.store.list_slice(request).await.map(page_from_storage).map_err(storage_error)
    }
}

fn storage_record_input(record: ReplaceUserRecord) -> StorageUserRecordInput {
    StorageUserRecordInput {
        username: record.username,
        password_hash: record.password_hash,
        email: record.email,
        role: record.role,
        is_active: record.is_active,
    }
}

fn user_auth_record(record: StorageUserAuthRecord) -> UserAuthRecord {
    UserAuthRecord {
        user: user_from_storage(record.user),
        password_hash: record.password_hash,
    }
}

fn user_from_storage(user: types::user::User) -> User {
    user.into()
}

fn page_from_storage(page: Page<types::user::User>) -> Page<User> {
    Page {
        items: page.items.into_iter().map(user_from_storage).collect(),
        total: page.total,
        page: page.page,
        page_size: page.page_size,
    }
}

fn storage_error(error: StorageError) -> AppError {
    match error {
        StorageError::NotFound => AppError::NotFound,
        StorageError::Conflict(message) => AppError::Conflict(message),
        StorageError::Database(message) => AppError::Infrastructure(message),
    }
}
