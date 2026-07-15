use std::sync::Arc;

use async_trait::async_trait;
use user::application::{AppError, UserUseCase};

use crate::application::{AuditError, AuditResult, LoginUnlocker};

#[derive(Clone)]
pub struct UserLoginUnlocker {
    users: Arc<dyn UserUseCase>,
}

impl UserLoginUnlocker {
    pub fn new(users: Arc<dyn UserUseCase>) -> Self {
        Self { users }
    }
}

#[async_trait]
impl LoginUnlocker for UserLoginUnlocker {
    async fn unlock(&self, username: &str) -> AuditResult<()> {
        self.users.unlock_login(username).await.map_err(map_user_error)
    }
}

fn map_user_error(error: AppError) -> AuditError {
    match error {
        AppError::NotFound => AuditError::NotFound,
        AppError::Infrastructure(message) => AuditError::Infrastructure(message),
        other => AuditError::Infrastructure(other.to_string()),
    }
}
