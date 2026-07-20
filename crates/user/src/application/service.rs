use std::collections::HashSet;

use async_trait::async_trait;
use kernel::error::LocalizedError;

use crate::application::{
    AppError, AppResult, AuditedPasswordChange, AuditedUserRepository, LoginFailureStore, LoginLockConfigProvider, OnlineSession, PasswordHasher,
    PasswordPolicy, PasswordPolicyProvider, ReplaceUserRecord, UserAuthRecord, UserExportRequest, UserExportSink, UserImportInput, UserImportMessage,
    UserImportReport, UserImportRow, UserImportWrite, UserListFilter, UserRepository, UserUseCase, VerifiedLogin,
};
use kernel::pagination::CursorPage;

use crate::domain::{Credentials, NewUser, ProfileUpdate, ReplaceUser, User, UserFormOptions, UserId, UserProfile};
use rbac::domain::DataScopeFilter;

const IMPORT_ACCOUNT_CREATED_KEY: &str = "messages.user.import_account_created";
const IMPORT_ACCOUNT_UPDATED_KEY: &str = "messages.user.import_account_updated";
const DATA_SCOPE_FORBIDDEN_KEY: &str = "errors.user.data_scope_forbidden";
const INSTALLATION_OWNER_PROTECTED_KEY: &str = "errors.user.installation_owner_protected";

use self::validation::{
    sanitize_and_validate_new_user, sanitize_credentials, sanitize_filter, sanitize_profile_update, sanitize_replace_user, validate_credentials, validate_page,
    validate_profile_update, validate_replace_user,
};

mod audited;
mod authentication;
mod commands;
mod imports;
mod installation_owner;
mod security;
mod use_case;
mod validation;

pub(crate) use installation_owner::validate_initial_installation_owner;
pub use security::UnconfiguredLoginSecurity;

pub struct UserService<R, H, P = StaticPasswordPolicyProvider, F = UnconfiguredLoginSecurity, C = UnconfiguredLoginSecurity> {
    repository: R,
    password_hasher: H,
    password_policy: P,
    login_failures: F,
    login_lock_config: C,
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
pub struct StaticPasswordPolicyProvider;

#[async_trait]
impl PasswordPolicyProvider for StaticPasswordPolicyProvider {
    async fn password_policy(&self) -> AppResult<PasswordPolicy> {
        Ok(PasswordPolicy::default())
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
    async fn create_unique_user(&self, input: NewUser) -> AppResult<User> {
        self.repository.create(self.prepare_new_user(input).await?).await
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

fn reject_unscoped_user_ids(requested: &[UserId], scoped: &[UserId]) -> AppResult<()> {
    if requested.iter().all(|id| scoped.contains(id)) {
        return Ok(());
    }
    Err(AppError::Forbidden(localized(DATA_SCOPE_FORBIDDEN_KEY)))
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
mod tests;
