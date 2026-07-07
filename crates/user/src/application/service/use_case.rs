use async_trait::async_trait;

use super::*;

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
        self.ensure_unique_user(UniqueUserCheck {
            username: &input.username,
            email: &input.email,
            phone: input.phonenumber.as_deref(),
            current_id: Some(id.clone()),
        })
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
