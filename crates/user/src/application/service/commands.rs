use super::*;

impl<R, H, P, F, C> UserService<R, H, P, F, C>
where
    R: UserRepository,
    H: PasswordHasher,
    P: PasswordPolicyProvider,
    F: LoginFailureStore,
    C: LoginLockConfigProvider,
{
    pub(super) async fn prepare_new_user(&self, input: NewUser) -> AppResult<ReplaceUserRecord> {
        let policy = self.password_policy.password_policy().await?;
        self.prepare_new_user_with_policy(input, policy).await
    }

    pub(super) async fn prepare_new_user_with_policy(&self, input: NewUser, policy: PasswordPolicy) -> AppResult<ReplaceUserRecord> {
        let input = sanitize_and_validate_new_user(input, &policy)?;
        self.ensure_unique_user(UniqueUserCheck {
            username: &input.username,
            email: &input.email,
            phone: input.phonenumber.as_deref(),
            current_id: None,
        })
        .await?;
        self.new_user_record(input)
    }

    pub(super) async fn prepare_replacement(&self, id: &UserId, input: ReplaceUser) -> AppResult<ReplaceUserRecord> {
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
        self.replace_user_record(input)
    }

    pub(super) async fn reject_installation_owner_mutation(&self, id: &UserId) -> AppResult<()> {
        if self.repository.is_installation_owner(id).await? {
            return Err(AppError::Forbidden(kernel::error::LocalizedError::new(INSTALLATION_OWNER_PROTECTED_KEY)));
        }
        Ok(())
    }

    pub(super) async fn reject_installation_owner_mutations(&self, ids: &[UserId]) -> AppResult<()> {
        for id in ids {
            self.reject_installation_owner_mutation(id).await?;
        }
        Ok(())
    }

    pub(super) async fn prepare_profile_update(&self, id: &UserId, profile: ProfileUpdate) -> AppResult<ProfileUpdate> {
        let profile = sanitize_profile_update(profile);
        validate_profile_update(&profile)?;
        ensure_user_exists(self.repository.find_by_id(id.clone()).await?)?;
        self.ensure_unique_user_profile(&profile.email, profile.phonenumber.as_deref(), Some(id.clone()))
            .await?;
        Ok(profile)
    }

    pub(super) async fn prepare_password_change(&self, id: &UserId, old_password: String, new_password: String) -> AppResult<String> {
        let old_password = old_password.trim().to_owned();
        let new_password = new_password.trim().to_owned();
        if old_password == new_password {
            return Err(AppError::Conflict(localized("errors.user.new_password_same_as_old")));
        }
        let found = self.repository.find_auth_by_id(id.clone()).await?.ok_or(AppError::Unauthorized)?;
        validation::validate_password(&new_password, &self.password_policy.password_policy().await?, Some(&found.user.username))?;
        verify_password(&self.password_hasher, &old_password, &found)?;
        self.password_hasher.hash(&new_password)
    }

    pub(super) async fn prepare_password_reset(&self, id: &UserId, password: String) -> AppResult<String> {
        let password = password.trim().to_owned();
        let user = self.repository.find_by_id(id.clone()).await?.ok_or(AppError::NotFound)?;
        validation::validate_password(&password, &self.password_policy.password_policy().await?, Some(&user.username))?;
        self.password_hasher.hash(&password)
    }

    pub(super) fn prepare_avatar_update(&self, avatar: String) -> AppResult<String> {
        reject_blank_avatar(&avatar)?;
        Ok(avatar.trim().into())
    }

    pub(super) fn validate_user_deletions(&self, ids: &[UserId]) -> AppResult<()> {
        if ids.is_empty() {
            return Err(AppError::InvalidInput(localized("errors.validation.ids_required")));
        }
        Ok(())
    }

    pub(super) fn prepare_status_update(&self, status: String) -> AppResult<String> {
        Ok(status.trim().into())
    }

    pub(super) fn prepare_role_replacement(&self, role_ids: Vec<String>) -> AppResult<Vec<String>> {
        Ok(role_ids
            .into_iter()
            .map(|value| value.trim().into())
            .filter(|value: &String| !value.is_empty())
            .collect())
    }
}
