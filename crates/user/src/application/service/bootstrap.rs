use crate::application::{
    AppResult, BootstrapAdministratorInput, BootstrapAdministratorOutcome, BootstrapAdministratorRecord, BootstrapAdministratorRepository, LoginFailureStore,
    LoginLockConfigProvider, PasswordHasher, PasswordPolicyProvider, UserRepository,
};

use super::{UniqueUserCheck, UserService, validation::sanitize_and_validate_new_user};

impl<R, H, P, F, C> UserService<R, H, P, F, C>
where
    R: UserRepository + BootstrapAdministratorRepository,
    H: PasswordHasher,
    P: PasswordPolicyProvider,
    F: LoginFailureStore,
    C: LoginLockConfigProvider,
{
    pub async fn has_enabled_system_administrator(&self) -> AppResult<bool> {
        self.repository.has_enabled_system_administrator().await
    }

    pub async fn bootstrap_administrator(&self, input: BootstrapAdministratorInput) -> AppResult<BootstrapAdministratorOutcome> {
        if self.has_enabled_system_administrator().await? {
            return Ok(BootstrapAdministratorOutcome::AlreadyPresent);
        }

        let record = self.prepare_bootstrap_administrator(input).await?;
        self.repository.create_system_administrator_if_absent(record).await
    }

    async fn prepare_bootstrap_administrator(&self, input: BootstrapAdministratorInput) -> AppResult<BootstrapAdministratorRecord> {
        let input = input.into_new_user();
        let policy = self.password_policy.password_policy().await?;
        let input = sanitize_and_validate_new_user(input, &policy)?;
        self.ensure_unique_user(UniqueUserCheck {
            username: &input.username,
            email: &input.email,
            phone: None,
            current_id: None,
        })
        .await?;
        let password_hash = self.password_hasher.hash(&input.password)?;
        Ok(BootstrapAdministratorRecord::from_new_user(input, password_hash))
    }
}
