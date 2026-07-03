use async_trait::async_trait;

use crate::application::{
    AppError, AppResult, PasswordHasher, ReplaceUserRecord, SystemUserProvider, SystemUserRecord, UserAuthRecord, UserRepository, UserUseCase,
};
use kernel::pagination::{Page, PageRequest};

use crate::domain::{Credentials, NewUser, ReplaceUser, User, UserId};

use self::{
    system_user::{find_auth_by_identifier, list_with_system_user, reject_conflicting_system_user, reject_system_user_id, system_user_by_id},
    validation::{
        sanitize_credentials, sanitize_new_user, sanitize_replace_user, validate_credentials, validate_new_user, validate_page, validate_replace_user,
    },
};

mod system_user;
mod validation;

pub struct UserService<R, H, S = NoSystemUserProvider> {
    repository: R,
    password_hasher: H,
    system_users: S,
}

struct UserRecordInput {
    username: String,
    password: String,
    email: String,
    role: String,
    is_active: bool,
}

#[derive(Clone, Copy)]
pub struct NoSystemUserProvider;

impl SystemUserProvider for NoSystemUserProvider {
    fn system_user(&self) -> Option<SystemUserRecord> {
        None
    }
}

impl<R, H> UserService<R, H, NoSystemUserProvider>
where
    R: UserRepository,
    H: PasswordHasher,
{
    pub const fn new(repository: R, password_hasher: H) -> Self {
        Self {
            repository,
            password_hasher,
            system_users: NoSystemUserProvider,
        }
    }
}

impl<R, H, S> UserService<R, H, S>
where
    R: UserRepository,
    H: PasswordHasher,
    S: SystemUserProvider,
{
    pub const fn with_system_user(repository: R, password_hasher: H, system_users: S) -> Self {
        Self {
            repository,
            password_hasher,
            system_users,
        }
    }

    async fn create_unique_user(&self, input: NewUser) -> AppResult<User> {
        let input = sanitize_new_user(input);
        validate_new_user(&input)?;
        self.ensure_unique_user(&input.username, &input.email, None).await?;
        self.ensure_unique_system_user(&input.username, &input.email)?;
        self.repository.create(self.new_user_record(input)?).await
    }

    async fn ensure_unique_user(&self, username: &str, email: &str, current_id: Option<UserId>) -> AppResult<()> {
        if let Some(found) = self.repository.find_auth_by_username(username).await? {
            reject_conflicting_user(found.user.id, current_id.as_ref(), "username")?;
        }

        if let Some(found) = self.repository.find_by_email(email).await? {
            reject_conflicting_user(found.id, current_id.as_ref(), "email")?;
        }

        Ok(())
    }

    fn ensure_unique_system_user(&self, username: &str, email: &str) -> AppResult<()> {
        reject_conflicting_system_user(&self.system_users, username, email)
    }

    fn new_user_record(&self, input: NewUser) -> AppResult<ReplaceUserRecord> {
        self.to_record(UserRecordInput::from(input))
    }

    fn replace_user_record(&self, input: ReplaceUser) -> AppResult<ReplaceUserRecord> {
        self.to_record(UserRecordInput::from(input))
    }

    fn to_record(&self, input: UserRecordInput) -> AppResult<ReplaceUserRecord> {
        Ok(ReplaceUserRecord {
            username: input.username,
            password_hash: self.password_hasher.hash(&input.password)?,
            email: input.email,
            role: input.role,
            is_active: input.is_active,
        })
    }
}

#[async_trait]
impl<R, H, S> UserUseCase for UserService<R, H, S>
where
    R: UserRepository,
    H: PasswordHasher,
    S: SystemUserProvider,
{
    async fn sign_up(&self, input: NewUser) -> AppResult<User> {
        self.create_unique_user(input).await
    }

    async fn sign_in(&self, input: Credentials) -> AppResult<User> {
        let input = sanitize_credentials(input);
        validate_credentials(&input)?;
        let found = find_auth_by_identifier(&self.repository, &self.system_users, &input.identifier)
            .await?
            .ok_or(AppError::Unauthorized)?;
        verify_password(&self.password_hasher, &input.password, &found)?;
        if !found.user.system {
            self.repository.record_login(found.user.id.clone()).await?;
        }
        Ok(found.user)
    }

    async fn authenticated_user(&self, id: UserId) -> AppResult<User> {
        if let Some(system_user) = system_user_by_id(&self.system_users, &id) {
            return Ok(system_user.user);
        }
        self.repository.find_by_id(id).await?.ok_or(AppError::Unauthorized)
    }

    async fn create_user(&self, input: NewUser) -> AppResult<User> {
        self.create_unique_user(input).await
    }

    async fn replace_user(&self, id: UserId, input: ReplaceUser) -> AppResult<User> {
        reject_system_user_id(&self.system_users, &id)?;
        let input = sanitize_replace_user(input);
        validate_replace_user(&input)?;
        ensure_user_exists(self.repository.find_by_id(id.clone()).await?)?;
        self.ensure_unique_user(&input.username, &input.email, Some(id.clone())).await?;
        self.ensure_unique_system_user(&input.username, &input.email)?;
        self.repository.replace(id, self.replace_user_record(input)?).await
    }

    async fn delete_user(&self, id: UserId) -> AppResult<()> {
        reject_system_user_id(&self.system_users, &id)?;
        self.repository.delete(id).await
    }

    async fn list_users(&self, page: PageRequest) -> AppResult<Page<User>> {
        validate_page(page)?;
        match self.system_users.system_user() {
            Some(system_user) => list_with_system_user(&self.repository, page, system_user.user).await,
            None => self.repository.list(page).await,
        }
    }
}

impl From<NewUser> for UserRecordInput {
    fn from(value: NewUser) -> Self {
        Self {
            username: value.username,
            password: value.password,
            email: value.email,
            role: value.role,
            is_active: value.is_active,
        }
    }
}

impl From<ReplaceUser> for UserRecordInput {
    fn from(value: ReplaceUser) -> Self {
        Self {
            username: value.username,
            password: value.password,
            email: value.email,
            role: value.role,
            is_active: value.is_active,
        }
    }
}

fn ensure_user_exists(user: Option<User>) -> AppResult<()> {
    match user {
        Some(_) => Ok(()),
        None => Err(AppError::NotFound),
    }
}

fn reject_conflicting_user(id: UserId, current_id: Option<&UserId>, field: &str) -> AppResult<()> {
    if current_id == Some(&id) {
        return Ok(());
    }

    Err(AppError::Conflict(format!("{field} already exists")))
}

fn verify_password<H: PasswordHasher>(hasher: &H, password: &str, found: &UserAuthRecord) -> AppResult<()> {
    if hasher.verify(password, &found.password_hash)? {
        return Ok(());
    }

    Err(AppError::Unauthorized)
}

#[cfg(test)]
mod system_tests;
#[cfg(test)]
mod tests;
