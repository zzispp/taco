use async_trait::async_trait;
use kernel::error::LocalizedError;

use crate::application::{
    AppError, AppResult, PasswordHasher, PasswordPolicy, PasswordPolicyProvider, ReplaceUserRecord, SystemUserProvider, SystemUserRecord, UserAuthRecord,
    UserImportInput, UserImportReport, UserImportRow, UserListFilter, UserRepository, UserUseCase,
};
use kernel::pagination::Page;

use crate::domain::{Credentials, NewUser, ProfileUpdate, ReplaceUser, User, UserFormOptions, UserId, UserProfile};
use types::rbac::DataScopeFilter;

const IMPORTED_USER_ROLE_ID: &str = "2";

use self::{
    system_user::{find_auth_by_identifier, list_with_system_user, reject_conflicting_system_user, reject_protected_user_id, system_user_by_id},
    validation::{
        sanitize_credentials, sanitize_new_user, sanitize_profile_update, sanitize_replace_user, validate_credentials, validate_new_user, validate_page,
        validate_profile_update, validate_replace_user,
    },
};

mod system_user;
mod use_case;
mod validation;

pub struct UserService<R, H, P = StaticPasswordPolicyProvider, S = NoSystemUserProvider> {
    repository: R,
    password_hasher: H,
    password_policy: P,
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

struct UniqueUserCheck<'a> {
    username: &'a str,
    email: &'a str,
    phone: Option<&'a str>,
    current_id: Option<UserId>,
}

#[derive(Clone, Copy)]
pub struct NoSystemUserProvider;

impl SystemUserProvider for NoSystemUserProvider {
    fn system_user(&self) -> Option<SystemUserRecord> {
        None
    }
}

#[derive(Clone, Copy)]
pub struct StaticPasswordPolicyProvider;

#[async_trait]
impl PasswordPolicyProvider for StaticPasswordPolicyProvider {
    async fn password_policy(&self) -> AppResult<PasswordPolicy> {
        Ok(PasswordPolicy::default())
    }
}

impl<R, H> UserService<R, H, StaticPasswordPolicyProvider, NoSystemUserProvider>
where
    R: UserRepository,
    H: PasswordHasher,
{
    pub const fn new(repository: R, password_hasher: H) -> Self {
        Self::with_password_policy(repository, password_hasher, StaticPasswordPolicyProvider)
    }
}

impl<R, H, P> UserService<R, H, P, NoSystemUserProvider>
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
            system_users: NoSystemUserProvider,
        }
    }
}

impl<R, H, P, S> UserService<R, H, P, S>
where
    R: UserRepository,
    H: PasswordHasher,
    P: PasswordPolicyProvider,
    S: SystemUserProvider,
{
    async fn create_unique_user(&self, input: NewUser) -> AppResult<User> {
        let input = sanitize_new_user(input);
        validate_new_user(&input, &self.password_policy.password_policy().await?)?;
        self.ensure_unique_user(UniqueUserCheck {
            username: &input.username,
            email: &input.email,
            phone: input.phonenumber.as_deref(),
            current_id: None,
        })
        .await?;
        self.ensure_unique_system_user(&input.username, &input.email)?;
        self.repository.create(self.new_user_record(input)?).await
    }

    async fn ensure_unique_user(&self, check: UniqueUserCheck<'_>) -> AppResult<()> {
        if let Some(found) = self.repository.find_auth_by_username(check.username).await? {
            reject_conflicting_user(found.user.id, check.current_id.as_ref(), "username")?;
        }

        self.ensure_unique_user_profile(check.email, check.phone, check.current_id).await
    }

    async fn ensure_unique_user_profile(&self, email: &str, phone: Option<&str>, current_id: Option<UserId>) -> AppResult<()> {
        if let Some(found) = self.repository.find_by_email(email).await? {
            reject_conflicting_user(found.id, current_id.as_ref(), "email")?;
        }

        if let Some(phone) = phone
            && let Some(found) = self.repository.find_by_phone(phone).await?
        {
            reject_conflicting_user(found.id, current_id.as_ref(), "phonenumber")?;
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

fn default_if_blank(value: String, default: &str) -> String {
    let value = value.trim();
    if value.is_empty() { default.into() } else { value.into() }
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

    Err(AppError::Conflict(localized_param("errors.user.duplicate_field", "field", field)))
}

fn verify_password<H: PasswordHasher>(hasher: &H, password: &str, found: &UserAuthRecord) -> AppResult<()> {
    if hasher.verify(password, &found.password_hash)? {
        return Ok(());
    }

    Err(AppError::Unauthorized)
}

fn reject_blank_avatar(avatar: &str) -> AppResult<()> {
    if avatar.trim().is_empty() {
        return Err(AppError::InvalidInput(localized("errors.user.avatar_blank")));
    }
    Ok(())
}

fn hash_optional_password<H: PasswordHasher>(hasher: &H, password: Option<String>) -> AppResult<Option<String>> {
    password.map(|value| hasher.hash(&value)).transpose()
}

fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
}

fn localized_param(key: &'static str, param: &'static str, value: impl Into<String>) -> LocalizedError {
    LocalizedError::new(key).with_param(param, value)
}

#[cfg(test)]
mod system_tests;
#[cfg(test)]
mod tests;
