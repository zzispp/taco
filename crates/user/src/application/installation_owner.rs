use std::fmt;

use async_trait::async_trait;

use crate::domain::User;

use super::{AppResult, ReplaceUserRecord};

pub const INSTALLATION_OWNER_PASSWORD_MIN_LENGTH: usize = 8;

/// The single account that owns an installation and bypasses business RBAC.
///
/// Only the setup flow may invoke this use case. Ordinary user-management
/// commands are intentionally unable to create or replace this account.
#[derive(Clone, PartialEq, Eq)]
pub struct InstallationOwnerInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl fmt::Debug for InstallationOwnerInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InstallationOwnerInput")
            .field("username", &self.username)
            .field("email", &self.email)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

/// Validates identity fields and the fixed initial-owner password policy
/// without querying existing users.
pub fn validate_initial_installation_owner(input: &InstallationOwnerInput) -> AppResult<()> {
    super::service::validate_initial_installation_owner(input)
}

#[async_trait]
pub trait InstallationOwnerRepository: Send + Sync + 'static {
    async fn has_installation_owner(&self) -> AppResult<bool>;
    async fn create_installation_owner(&self, user: ReplaceUserRecord) -> AppResult<User>;
}

#[async_trait]
pub trait InstallationOwnerUseCase: Send + Sync + 'static {
    async fn has_installation_owner(&self) -> AppResult<bool>;
    async fn create_installation_owner(&self, input: InstallationOwnerInput) -> AppResult<User>;
}

#[cfg(test)]
mod tests {
    use super::InstallationOwnerInput;

    #[test]
    fn installation_owner_input_debug_redacts_the_password() {
        let input = InstallationOwnerInput {
            username: "owner".into(),
            email: "owner@example.test".into(),
            password: "owner-password".into(),
        };

        let rendered = format!("{input:?}");
        assert!(!rendered.contains("owner-password"));
        assert!(rendered.contains("[REDACTED]"));
    }
}
