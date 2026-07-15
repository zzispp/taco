use async_trait::async_trait;

use crate::domain::User;

use super::{AppResult, ReplaceUserRecord};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootstrapAdminInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[async_trait]
pub trait AdminBootstrapRepository: Send + Sync + 'static {
    async fn create_bootstrap_admin(&self, user: ReplaceUserRecord) -> AppResult<User>;
}

#[async_trait]
pub trait AdminBootstrapUseCase: Send + Sync + 'static {
    async fn bootstrap_admin(&self, input: BootstrapAdminInput) -> AppResult<User>;
}
