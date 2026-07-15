use async_trait::async_trait;

use crate::{
    application::{AppError, AppResult, LoginFailureStore, LoginLockConfig, LoginLockConfigProvider, PasswordHasher, PasswordPolicyProvider, UserRepository},
    domain::UserId,
};

use super::{StaticPasswordPolicyProvider, UserService};

#[derive(Clone, Copy)]
pub struct UnconfiguredLoginSecurity;

#[async_trait]
impl LoginLockConfigProvider for UnconfiguredLoginSecurity {
    async fn login_lock_config(&self) -> AppResult<LoginLockConfig> {
        Err(AppError::Infrastructure("infra.user.login_lock_config_unconfigured".into()))
    }
}

#[async_trait]
impl LoginFailureStore for UnconfiguredLoginSecurity {
    async fn failure_count(&self, _user_id: &UserId) -> AppResult<u32> {
        Err(AppError::Infrastructure("infra.user.login_failure_store_unconfigured".into()))
    }

    async fn record_failure(&self, _user_id: &UserId, _ttl_seconds: u64) -> AppResult<u32> {
        Err(AppError::Infrastructure("infra.user.login_failure_store_unconfigured".into()))
    }

    async fn clear_failures(&self, _user_id: &UserId) -> AppResult<()> {
        Err(AppError::Infrastructure("infra.user.login_failure_store_unconfigured".into()))
    }
}

impl<R, H> UserService<R, H, StaticPasswordPolicyProvider, UnconfiguredLoginSecurity, UnconfiguredLoginSecurity>
where
    R: UserRepository,
    H: PasswordHasher,
{
    pub const fn new(repository: R, password_hasher: H) -> Self {
        Self::with_password_policy(repository, password_hasher, StaticPasswordPolicyProvider)
    }
}

impl<R, H, P> UserService<R, H, P, UnconfiguredLoginSecurity, UnconfiguredLoginSecurity>
where
    R: UserRepository,
    H: PasswordHasher,
    P: PasswordPolicyProvider,
{
    pub const fn with_password_policy(repository: R, password_hasher: H, password_policy: P) -> Self {
        Self {
            repository,
            password_hasher,
            password_policy,
            login_failures: UnconfiguredLoginSecurity,
            login_lock_config: UnconfiguredLoginSecurity,
        }
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
    pub fn with_login_security<NF, NC>(self, login_failures: NF, login_lock_config: NC) -> UserService<R, H, P, NF, NC>
    where
        NF: LoginFailureStore,
        NC: LoginLockConfigProvider,
    {
        UserService {
            repository: self.repository,
            password_hasher: self.password_hasher,
            password_policy: self.password_policy,
            login_failures,
            login_lock_config,
        }
    }
}
