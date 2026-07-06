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
    pub const fn with_system_user(repository: R, password_hasher: H, password_policy: P, system_users: S) -> Self {
        Self {
            repository,
            password_hasher,
            password_policy,
            system_users,
        }
    }

    async fn create_unique_user(&self, input: NewUser) -> AppResult<User> {
        let input = sanitize_new_user(input);
        validate_new_user(&input, &self.password_policy.password_policy().await?)?;
        self.ensure_unique_user(&input.username, &input.email, input.phonenumber.as_deref(), None)
            .await?;
        self.ensure_unique_system_user(&input.username, &input.email)?;
        self.repository.create(self.new_user_record(input)?).await
    }

    async fn ensure_unique_user(&self, username: &str, email: &str, phone: Option<&str>, current_id: Option<UserId>) -> AppResult<()> {
        if let Some(found) = self.repository.find_auth_by_username(username).await? {
            reject_conflicting_user(found.user.id, current_id.as_ref(), "username")?;
        }

        self.ensure_unique_user_profile(email, phone, current_id).await
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

#[async_trait]
impl<R, H, P, S> UserUseCase for UserService<R, H, P, S>
where
    R: UserRepository,
    H: PasswordHasher,
    P: PasswordPolicyProvider,
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

    async fn profile(&self, id: UserId) -> AppResult<UserProfile> {
        let user = self.authenticated_user(id.clone()).await?;
        let groups = self.repository.profile_groups(id).await?;
        Ok(UserProfile {
            user,
            role_group: groups.role_group,
            post_group: groups.post_group,
            dept_name: groups.dept_name,
        })
    }

    async fn update_profile(&self, id: UserId, profile: ProfileUpdate) -> AppResult<User> {
        let profile = sanitize_profile_update(profile);
        validate_profile_update(&profile)?;
        ensure_user_exists(self.repository.find_by_id(id.clone()).await?)?;
        self.ensure_unique_user_profile(&profile.email, profile.phonenumber.as_deref(), Some(id.clone()))
            .await?;
        self.repository.update_profile(id, profile).await
    }

    async fn change_password(&self, id: UserId, old_password: String, new_password: String) -> AppResult<()> {
        let old_password = old_password.trim().to_owned();
        let new_password = new_password.trim().to_owned();
        if old_password == new_password {
            return Err(AppError::Conflict(localized("errors.user.new_password_same_as_old")));
        }
        let found = self.repository.find_auth_by_id(id.clone()).await?.ok_or(AppError::Unauthorized)?;
        validation::validate_password(&new_password, &self.password_policy.password_policy().await?, Some(&found.user.username))?;
        verify_password(&self.password_hasher, &old_password, &found)?;
        let hash = self.password_hasher.hash(&new_password)?;
        self.repository.update_password(id, hash).await
    }

    async fn update_avatar(&self, id: UserId, avatar: String) -> AppResult<User> {
        reject_blank_avatar(&avatar)?;
        self.repository.update_avatar(id, avatar.trim().into()).await
    }

    async fn create_user(&self, input: NewUser) -> AppResult<User> {
        self.create_unique_user(input).await
    }

    async fn replace_user(&self, id: UserId, input: ReplaceUser) -> AppResult<User> {
        reject_protected_user_id(&self.system_users, &id)?;
        let input = sanitize_replace_user(input);
        validate_replace_user(&input, &self.password_policy.password_policy().await?)?;
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
            return Err(AppError::InvalidInput(localized("errors.validation.ids_required")));
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
        let user = self.repository.find_by_id(id.clone()).await?.ok_or(AppError::NotFound)?;
        validation::validate_password(&password, &self.password_policy.password_policy().await?, Some(&user.username))?;
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

    async fn import_users(&self, input: UserImportInput) -> AppResult<UserImportReport> {
        if input.rows.is_empty() {
            return Err(AppError::InvalidInput(localized("errors.user.import_empty")));
        }
        let mut successes = Vec::new();
        let mut failures = Vec::new();
        for row in input.rows {
            match self.import_user_row(row, input.update_support, &input.default_password).await {
                Ok(message) => successes.push(message),
                Err(error) => failures.push(error.to_string()),
            }
        }
        if !failures.is_empty() {
            return Err(AppError::InvalidInput(localized_param(
                "errors.user.import_failed",
                "errors",
                failures.join("; "),
            )));
        }
        Ok(UserImportReport {
            success_count: successes.len(),
            message: format!("用户导入成功，共 {} 条。{}", successes.len(), successes.join("；")),
        })
    }

    async fn form_options(&self) -> AppResult<UserFormOptions> {
        self.repository.form_options().await
    }
}

impl<R, H, P, S> UserService<R, H, P, S>
where
    R: UserRepository,
    H: PasswordHasher,
    P: PasswordPolicyProvider,
    S: SystemUserProvider,
{
    async fn import_user_row(&self, row: UserImportRow, update_support: bool, default_password: &str) -> AppResult<String> {
        let username = row.username.trim().to_owned();
        let found = self.repository.find_auth_by_username(&username).await?;
        match found {
            None => self.create_imported_user(row, default_password).await,
            Some(existing) if update_support => self.update_imported_user(row, existing.user).await,
            Some(_) => Err(AppError::Conflict(localized_param("errors.user.import_account_exists", "username", username))),
        }
    }

    async fn create_imported_user(&self, row: UserImportRow, default_password: &str) -> AppResult<String> {
        let username = row.username.trim().to_owned();
        self.create_unique_user(NewUser {
            username: username.clone(),
            password: default_password.to_owned(),
            nick_name: row.nick_name,
            dept_id: row.dept_id,
            email: row.email,
            phonenumber: row.phonenumber,
            sex: default_if_blank(row.sex, "2"),
            status: default_if_blank(row.status, "0"),
            remark: None,
            role_ids: vec![IMPORTED_USER_ROLE_ID.into()],
            post_ids: vec![],
        })
        .await?;
        Ok(format!("账号 {username} 导入成功"))
    }

    async fn update_imported_user(&self, row: UserImportRow, existing: User) -> AppResult<String> {
        let username = row.username.trim().to_owned();
        self.replace_user(
            existing.id.clone(),
            ReplaceUser {
                username: username.clone(),
                password: None,
                nick_name: row.nick_name,
                dept_id: existing.dept_id,
                email: row.email,
                phonenumber: row.phonenumber,
                sex: default_if_blank(row.sex, "2"),
                status: default_if_blank(row.status, "0"),
                remark: existing.remark,
                role_ids: existing.role_ids,
                post_ids: existing.post_ids,
            },
        )
        .await?;
        Ok(format!("账号 {username} 更新成功"))
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
