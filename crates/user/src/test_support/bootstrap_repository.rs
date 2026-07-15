use async_trait::async_trait;
use kernel::error::LocalizedError;

use super::*;
use crate::application::AdminBootstrapRepository;

#[async_trait]
impl AdminBootstrapRepository for MemoryUserRepository {
    async fn create_bootstrap_admin(&self, mut record: ReplaceUserRecord) -> AppResult<User> {
        let mut state = self.state.lock().unwrap();
        if state.users.iter().any(is_existing_super_admin) {
            return Err(AppError::Conflict(LocalizedError::new("errors.user.bootstrap_admin_exists")));
        }
        record.role_ids = vec!["1".into()];
        Ok(store_created_user(&mut state, record))
    }
}

fn is_existing_super_admin(stored: &StoredUser) -> bool {
    stored.user.roles.iter().any(|role| role.role_key == constants::system::SUPER_ADMIN_ROLE_KEY)
}
