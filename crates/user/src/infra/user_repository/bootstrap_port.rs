use async_trait::async_trait;
use kernel::error::LocalizedError;

use crate::{
    application::{AdminBootstrapRepository, AppError, AppResult, ReplaceUserRecord},
    domain::User,
};

use super::{StorageUserRepository, mapping::storage_error, queries::bootstrap::BootstrapAdminOutcome};

#[async_trait]
impl AdminBootstrapRepository for StorageUserRepository {
    async fn create_bootstrap_admin(&self, user: ReplaceUserRecord) -> AppResult<User> {
        match self.queries.bootstrap_admin(user).await.map_err(storage_error)? {
            BootstrapAdminOutcome::Created(user) => Ok(*user),
            BootstrapAdminOutcome::ExistingSuperAdmin => Err(AppError::Conflict(LocalizedError::new("errors.user.bootstrap_admin_exists"))),
            BootstrapAdminOutcome::MissingAdminRole => Err(AppError::InvalidInput(LocalizedError::new("errors.user.bootstrap_admin_role_missing"))),
        }
    }
}
