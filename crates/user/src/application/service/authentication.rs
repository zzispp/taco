use crate::{
    application::{
        AppError, AppResult, LoginFailureStore, LoginLockConfig, LoginLockConfigProvider, PasswordHasher, PasswordPolicyProvider, UserRepository, VerifiedLogin,
    },
    domain::{Credentials, User, UserId},
};

use super::{UserService, sanitize_credentials, validate_credentials};

impl<R, H, P, F, C> UserService<R, H, P, F, C>
where
    R: UserRepository,
    H: PasswordHasher,
    P: PasswordPolicyProvider,
    F: LoginFailureStore,
    C: LoginLockConfigProvider,
{
    pub(super) async fn authenticate(&self, input: Credentials) -> AppResult<VerifiedLogin> {
        let input = sanitize_credentials(input);
        validate_credentials(&input)?;
        let Some(found) = find_auth_by_identifier(&self.repository, &input.identifier).await? else {
            self.password_hasher.hash(&input.password)?;
            return Err(AppError::Unauthorized);
        };
        let password_matches = self.password_hasher.verify(&input.password, &found.password_hash)?;
        if found.user.status != constants::system::STATUS_NORMAL {
            return Err(AppError::AccountDisabled);
        }
        let config = self.login_lock_config.login_lock_config().await?;
        config.validate()?;
        self.reject_locked(&found.user.id, &config).await?;
        self.verify_login_password(LoginPassword {
            user_id: &found.user.id,
            password_matches,
            config: &config,
        })
        .await?;
        Ok(VerifiedLogin::new(found.user))
    }

    pub(super) async fn complete_authentication(&self, login: VerifiedLogin, ipaddr: String) -> AppResult<User> {
        let user = login.into_user();
        self.login_failures.clear_failures(&user.id).await?;
        self.repository.record_login(user.id.clone(), ipaddr).await?;
        Ok(user)
    }

    pub(super) async fn unlock_login_account(&self, username: &str) -> AppResult<()> {
        let identifier = username.trim();
        let found = find_auth_by_identifier(&self.repository, identifier).await?.ok_or(AppError::NotFound)?;
        self.login_failures.clear_failures(&found.user.id).await
    }

    async fn reject_locked(&self, user_id: &UserId, config: &LoginLockConfig) -> AppResult<()> {
        if self.login_failures.failure_count(user_id).await? >= config.max_retry_count {
            return Err(AppError::AccountLocked {
                lock_minutes: config.lock_minutes,
            });
        }
        Ok(())
    }

    async fn verify_login_password(&self, input: LoginPassword<'_>) -> AppResult<()> {
        if input.password_matches {
            return Ok(());
        }
        let count = self.login_failures.record_failure(input.user_id, input.config.lock_seconds()?).await?;
        if count >= input.config.max_retry_count {
            return Err(AppError::AccountLocked {
                lock_minutes: input.config.lock_minutes,
            });
        }
        Err(AppError::Unauthorized)
    }
}

struct LoginPassword<'a> {
    user_id: &'a UserId,
    password_matches: bool,
    config: &'a LoginLockConfig,
}

async fn find_auth_by_identifier<R: UserRepository>(repository: &R, identifier: &str) -> AppResult<Option<crate::application::UserAuthRecord>> {
    if let Some(record) = repository.find_auth_by_username(identifier).await? {
        return Ok(Some(record));
    }
    repository.find_auth_by_email(identifier).await
}
