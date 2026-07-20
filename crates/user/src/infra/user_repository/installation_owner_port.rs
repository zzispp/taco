use async_trait::async_trait;
use kernel::error::LocalizedError;

use crate::{
    application::{AppError, AppResult, InstallationOwnerRepository, ReplaceUserRecord},
    domain::User,
};

use super::{StorageUserRepository, mapping::storage_error};

#[async_trait]
impl InstallationOwnerRepository for StorageUserRepository {
    async fn has_installation_owner(&self) -> AppResult<bool> {
        self.queries.has_installation_owner().await.map_err(storage_error)
    }

    async fn create_installation_owner(&self, user: ReplaceUserRecord) -> AppResult<User> {
        self.queries
            .create_installation_owner(user)
            .await
            .map_err(storage_error)?
            .ok_or_else(|| AppError::Conflict(LocalizedError::new("errors.user.installation_owner_exists")))
    }
}
