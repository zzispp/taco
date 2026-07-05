use async_trait::async_trait;

use crate::application::{
    AppError, AppResult, PasswordHasher, ReplaceUserRecord, SystemUserProvider, SystemUserRecord, UserAuthRecord, UserListFilter, UserRepository, UserUseCase,
};
use kernel::pagination::Page;

use crate::domain::{Credentials, NewUser, ReplaceUser, User, UserFormOptions, UserId};
use types::rbac::DataScopeFilter;

use self::{
    system_user::{find_auth_by_identifier, list_with_system_user, reject_conflicting_system_user, reject_protected_user_id, system_user_by_id},
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
    password: Option<String>,
    nick_name: String,
    dept_id: Option<String>,
    email: String,
    phonenumber: Option<String>,
    sex: String,
    status: String,
    remark: Option<String>,
    role_ids: Vec<String>,
    post_ids: Vec<String>,
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
        self.ensure_unique_user(&input.username, &input.email, input.phonenumber.as_deref(), None)
            .await?;
        self.ensure_unique_system_user(&input.username, &input.email)?;
        self.repository.create(self.new_user_record(input)?).await
    }

    async fn ensure_unique_user(&self, username: &str, email: &str, phone: Option<&str>, current_id: Option<UserId>) -> AppResult<()> {
        if let Some(found) = self.repository.find_auth_by_username(username).await? {
            reject_conflicting_user(found.user.id, current_id.as_ref(), "username")?;
        }

        if let Some(found) = self.repository.find_by_email(email).await? {
            reject_conflicting_user(found.id, current_id.as_ref(), "email")?;
        }

        if let Some(phone) = phone {
            if let Some(found) = self.repository.find_by_phone(phone).await? {
                reject_conflicting_user(found.id, current_id.as_ref(), "phonenumber")?;
            }
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
            password_hash: hash_optional_password(&self.password_hasher, input.password)?,
            nick_name: input.nick_name,
            dept_id: input.dept_id,
            email: input.email,
            phonenumber: input.phonenumber,
            sex: input.sex,
            status: input.status,
            remark: input.remark,
            role_ids: input.role_ids,
            post_ids: input.post_ids,
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
        reject_protected_user_id(&self.system_users, &id)?;
        let input = sanitize_replace_user(input);
        validate_replace_user(&input)?;
        ensure_user_exists(self.repository.find_by_id(id.clone()).await?)?;
        self.ensure_unique_user(&input.username, &input.email, input.phonenumber.as_deref(), Some(id.clone()))
            .await?;
        self.ensure_unique_system_user(&input.username, &input.email)?;
        self.repository.replace(id, self.replace_user_record(input)?).await
    }

    async fn delete_user(&self, id: UserId) -> AppResult<()> {
        reject_protected_user_id(&self.system_users, &id)?;
        self.repository.delete(id).await
    }

    async fn delete_users(&self, ids: Vec<UserId>) -> AppResult<()> {
        if ids.is_empty() {
            return Err(AppError::InvalidInput("ids are required".into()));
        }
        for id in &ids {
            reject_protected_user_id(&self.system_users, id)?;
        }
        self.repository.delete_many(ids).await
    }

    async fn get_user(&self, id: UserId) -> AppResult<User> {
        self.repository.find_by_id(id).await?.ok_or(AppError::NotFound)
    }

    async fn reset_password(&self, id: UserId, password: String) -> AppResult<()> {
        reject_protected_user_id(&self.system_users, &id)?;
        let password = password.trim().to_string();
        validation::validate_password(&password)?;
        let hash = self.password_hasher.hash(&password)?;
        self.repository.update_password(id, hash).await
    }

    async fn update_status(&self, id: UserId, status: String) -> AppResult<User> {
        reject_protected_user_id(&self.system_users, &id)?;
        self.repository.update_status(id, status.trim().into()).await
    }

    async fn replace_roles(&self, id: UserId, role_ids: Vec<String>) -> AppResult<User> {
        reject_protected_user_id(&self.system_users, &id)?;
        let role_ids = role_ids.into_iter().map(|id| id.trim().into()).filter(|id: &String| !id.is_empty()).collect();
        self.repository.replace_roles(id, role_ids).await
    }

    async fn list_users(&self, filter: UserListFilter) -> AppResult<Page<User>> {
        let filter = sanitize_filter(filter);
        validate_page(filter.page)?;
        match self.system_users.system_user() {
            Some(system_user) => list_with_system_user(&self.repository, filter, system_user.user).await,
            None => self.repository.list(filter).await,
        }
    }

    async fn list_users_scoped(&self, filter: UserListFilter, scope: DataScopeFilter) -> AppResult<Page<User>> {
        let filter = sanitize_filter(filter);
        validate_page(filter.page)?;
        self.repository.list_scoped(filter, scope).await
    }

    async fn form_options(&self) -> AppResult<UserFormOptions> {
        self.repository.form_options().await
    }
}

fn sanitize_filter(input: UserListFilter) -> UserListFilter {
    UserListFilter {
        page: input.page,
        username: trim_filter(input.username),
        phonenumber: trim_filter(input.phonenumber),
        status: trim_filter(input.status),
        dept_id: trim_filter(input.dept_id),
        begin_time: trim_filter(input.begin_time),
        end_time: trim_filter(input.end_time),
    }
}

fn trim_filter(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().into()).filter(|item: &String| !item.is_empty())
}

impl From<NewUser> for UserRecordInput {
    fn from(value: NewUser) -> Self {
        Self {
            username: value.username,
            password: Some(value.password),
            nick_name: value.nick_name,
            dept_id: value.dept_id,
            email: value.email,
            phonenumber: value.phonenumber,
            sex: value.sex,
            status: value.status,
            remark: value.remark,
            role_ids: value.role_ids,
            post_ids: value.post_ids,
        }
    }
}

impl From<ReplaceUser> for UserRecordInput {
    fn from(value: ReplaceUser) -> Self {
        Self {
            username: value.username,
            password: value.password,
            nick_name: value.nick_name,
            dept_id: value.dept_id,
            email: value.email,
            phonenumber: value.phonenumber,
            sex: value.sex,
            status: value.status,
            remark: value.remark,
            role_ids: value.role_ids,
            post_ids: value.post_ids,
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

fn hash_optional_password<H: PasswordHasher>(hasher: &H, password: Option<String>) -> AppResult<Option<String>> {
    password.map(|value| hasher.hash(&value)).transpose()
}

#[cfg(test)]
mod system_tests;
#[cfg(test)]
mod tests;
