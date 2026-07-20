use async_trait::async_trait;
use kernel::error::LocalizedError;

use crate::{
    application::{AppError, AppResult, InstallationOwnerRepository, ReplaceUserRecord},
    domain::User,
};

use super::{MemoryUserRepository, set_installation_owner, store_created_user};

#[async_trait]
impl InstallationOwnerRepository for MemoryUserRepository {
    async fn has_installation_owner(&self) -> AppResult<bool> {
        Ok(self.state.lock().unwrap().installation_owner.is_some())
    }

    async fn create_installation_owner(&self, mut record: ReplaceUserRecord) -> AppResult<User> {
        let mut state = self.state.lock().unwrap();
        if state.installation_owner.is_some() {
            return Err(AppError::Conflict(LocalizedError::new("errors.user.installation_owner_exists")));
        }
        record.role_ids.clear();
        record.post_ids.clear();
        let mut user = store_created_user(&mut state, record);
        set_installation_owner(&mut state, user.id.clone());
        user.is_installation_owner = true;
        Ok(user)
    }
}
