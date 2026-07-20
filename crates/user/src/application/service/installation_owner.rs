use async_trait::async_trait;
use constants::system::STATUS_NORMAL;

use super::validation::sanitize_and_validate_new_user;
use super::*;
use crate::{
    application::{INSTALLATION_OWNER_PASSWORD_MIN_LENGTH, InstallationOwnerInput, InstallationOwnerRepository, InstallationOwnerUseCase},
    domain::NewUser,
};

#[async_trait]
impl<R, H, P, F, C> InstallationOwnerUseCase for UserService<R, H, P, F, C>
where
    R: UserRepository + InstallationOwnerRepository,
    H: PasswordHasher,
    P: PasswordPolicyProvider,
    F: LoginFailureStore,
    C: LoginLockConfigProvider,
{
    async fn has_installation_owner(&self) -> AppResult<bool> {
        self.repository.has_installation_owner().await
    }

    async fn create_installation_owner(&self, input: InstallationOwnerInput) -> AppResult<User> {
        let user = self.prepare_installation_owner(input).await?;
        self.repository.create_installation_owner(user).await
    }
}

impl<R, H, P, F, C> UserService<R, H, P, F, C>
where
    R: UserRepository,
    H: PasswordHasher,
    P: PasswordPolicyProvider,
    F: LoginFailureStore,
    C: LoginLockConfigProvider,
{
    async fn prepare_installation_owner(&self, input: InstallationOwnerInput) -> AppResult<ReplaceUserRecord> {
        self.prepare_new_user_with_policy(initial_owner_new_user(&input), installation_owner_password_policy())
            .await
    }
}

pub(crate) fn validate_initial_installation_owner(input: &InstallationOwnerInput) -> AppResult<()> {
    let policy = installation_owner_password_policy();
    sanitize_and_validate_new_user(initial_owner_new_user(input), &policy).map(|_| ())
}

fn initial_owner_new_user(input: &InstallationOwnerInput) -> NewUser {
    NewUser {
        username: input.username.clone(),
        password: input.password.clone(),
        nick_name: input.username.clone(),
        dept_id: None,
        email: input.email.clone(),
        phonenumber: None,
        sex: "2".into(),
        status: STATUS_NORMAL.into(),
        remark: None,
        role_ids: Vec::new(),
        post_ids: Vec::new(),
    }
}

fn installation_owner_password_policy() -> PasswordPolicy {
    PasswordPolicy {
        min_length: INSTALLATION_OWNER_PASSWORD_MIN_LENGTH,
        forbid_username_contains: true,
        ..PasswordPolicy::default()
    }
}
