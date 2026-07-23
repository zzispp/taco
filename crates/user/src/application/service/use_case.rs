use async_trait::async_trait;

use crate::application::AuthorizationUser;
use crate::domain::AvatarFileId;

use super::*;

#[async_trait]
impl<R, H, P, F, C> UserUseCase for UserService<R, H, P, F, C>
where
    R: AuditedUserRepository,
    H: PasswordHasher,
    P: PasswordPolicyProvider,
    F: LoginFailureStore,
    C: LoginLockConfigProvider,
{
    async fn sign_up(&self, input: NewUser) -> AppResult<User> {
        self.create_unique_user(input).await
    }

    async fn sign_up_with_audit(&self, input: NewUser, audit: audit_contract::AuditOutboxRecord) -> AppResult<User> {
        self.sign_up_with_audit_command(input, audit).await
    }

    async fn sign_in(&self, input: Credentials) -> AppResult<VerifiedLogin> {
        self.authenticate(input).await
    }

    async fn complete_sign_in(&self, login: VerifiedLogin, ipaddr: String) -> AppResult<User> {
        self.complete_authentication(login, ipaddr).await
    }

    async fn complete_sign_in_with_audit(&self, login: VerifiedLogin, ipaddr: String, audit: audit_contract::AuditOutboxRecord) -> AppResult<User> {
        self.complete_sign_in_with_audit_command(login, ipaddr, audit).await
    }

    async fn unlock_login(&self, username: &str) -> AppResult<()> {
        self.unlock_login_account(username).await
    }

    async fn authenticated_user(&self, id: UserId) -> AppResult<User> {
        let user = self.repository.find_by_id(id).await?.ok_or(AppError::Unauthorized)?;
        active_authenticated_user(user)
    }

    async fn authorization_user(&self, id: UserId) -> AppResult<AuthorizationUser> {
        let user = self.repository.find_authorization_by_id(id).await?.ok_or(AppError::Unauthorized)?;
        active_authorization_user(user)
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
        let profile = self.prepare_profile_update(&id, profile).await?;
        self.repository.update_profile(id, profile).await
    }

    async fn update_profile_with_audit(&self, id: UserId, profile: ProfileUpdate, audit: audit_contract::AuditOutboxRecord) -> AppResult<User> {
        self.update_profile_with_audit_command(id, profile, audit).await
    }

    async fn change_password(&self, id: UserId, old_password: String, new_password: String) -> AppResult<()> {
        let hash = self.prepare_password_change(&id, old_password, new_password).await?;
        self.repository.update_password(id, hash).await
    }

    async fn change_password_with_audit(&self, input: AuditedPasswordChange) -> AppResult<()> {
        self.change_password_with_audit_command(input).await
    }

    async fn update_avatar(&self, id: UserId, avatar: AvatarFileId) -> AppResult<User> {
        let avatar = self.prepare_avatar_update(avatar)?;
        self.repository.update_avatar(id, avatar).await
    }

    async fn update_avatar_with_audit(&self, id: UserId, avatar: AvatarFileId, audit: audit_contract::AuditOutboxRecord) -> AppResult<User> {
        self.update_avatar_with_audit_command(id, avatar, audit).await
    }

    async fn create_user(&self, input: NewUser) -> AppResult<User> {
        self.create_unique_user(input).await
    }

    async fn create_user_with_audit(&self, input: NewUser, audit: audit_contract::AuditOutboxRecord) -> AppResult<User> {
        self.create_user_with_audit_command(input, audit).await
    }

    async fn replace_user(&self, id: UserId, input: ReplaceUser) -> AppResult<User> {
        let user = self.prepare_replacement(&id, input).await?;
        self.repository.replace(id, user).await
    }

    async fn replace_user_with_audit(&self, id: UserId, input: ReplaceUser, audit: audit_contract::AuditOutboxRecord) -> AppResult<User> {
        self.replace_user_with_audit_command(id, input, audit).await
    }

    async fn delete_user(&self, id: UserId) -> AppResult<()> {
        self.repository.delete(id).await
    }

    async fn delete_user_with_audit(&self, id: UserId, audit: audit_contract::AuditOutboxRecord) -> AppResult<()> {
        self.delete_user_with_audit_command(id, audit).await
    }

    async fn delete_users(&self, ids: Vec<UserId>) -> AppResult<()> {
        self.validate_user_deletions(&ids)?;
        self.repository.delete_many(ids).await
    }

    async fn delete_users_with_audit(&self, ids: Vec<UserId>, audit: audit_contract::AuditOutboxRecord) -> AppResult<()> {
        self.delete_users_with_audit_command(ids, audit).await
    }

    async fn get_user(&self, id: UserId) -> AppResult<User> {
        self.repository.find_by_id(id).await?.ok_or(AppError::NotFound)
    }

    async fn reset_password(&self, id: UserId, password: String) -> AppResult<()> {
        let hash = self.prepare_password_reset(&id, password).await?;
        self.repository.update_password(id, hash).await
    }

    async fn reset_password_with_audit(&self, id: UserId, password: String, audit: audit_contract::AuditOutboxRecord) -> AppResult<()> {
        self.reset_password_with_audit_command(id, password, audit).await
    }

    async fn update_status(&self, id: UserId, status: String) -> AppResult<User> {
        let status = self.prepare_status_update(status)?;
        self.repository.update_status(id, status).await
    }

    async fn update_status_with_audit(&self, id: UserId, status: String, audit: audit_contract::AuditOutboxRecord) -> AppResult<User> {
        self.update_status_with_audit_command(id, status, audit).await
    }

    async fn replace_roles(&self, id: UserId, role_ids: Vec<String>) -> AppResult<User> {
        let role_ids = self.prepare_role_replacement(role_ids)?;
        self.repository.replace_roles(id, role_ids).await
    }

    async fn replace_roles_with_audit(&self, id: UserId, role_ids: Vec<String>, audit: audit_contract::AuditOutboxRecord) -> AppResult<User> {
        self.replace_roles_with_audit_command(id, role_ids, audit).await
    }

    async fn list_users(&self, filter: UserListFilter) -> AppResult<CursorPage<User>> {
        let filter = sanitize_filter(filter);
        validate_page(&filter.page)?;
        crate::application::cursor::UserCursorCodec::new(&filter, None)?.decode(&filter.page)?;
        self.repository.list(filter).await
    }

    async fn list_users_scoped(&self, filter: UserListFilter, scope: DataScopeFilter) -> AppResult<CursorPage<User>> {
        let filter = sanitize_filter(filter);
        validate_page(&filter.page)?;
        crate::application::cursor::UserCursorCodec::new(&filter, Some(&scope))?.decode(&filter.page)?;
        self.repository.list_scoped(filter, scope).await
    }

    async fn export_users(&self, request: UserExportRequest, sink: &mut dyn UserExportSink) -> AppResult<()> {
        if request.batch_size == 0 {
            return Err(AppError::InvalidInput(kernel::error::LocalizedError::new("errors.common.invalid_input")));
        }
        let request = UserExportRequest {
            filter: sanitize_filter(request.filter),
            ..request
        };
        self.repository.export_users(request, sink).await
    }

    async fn ensure_user_ids_scoped(&self, ids: Vec<UserId>, scope: DataScopeFilter) -> AppResult<()> {
        let scoped = self.repository.list_scoped_ids(ids.clone(), scope).await?;
        reject_unscoped_user_ids(&ids, &scoped)
    }

    async fn filter_online_sessions_scoped(&self, sessions: Vec<OnlineSession>, scope: DataScopeFilter) -> AppResult<Vec<OnlineSession>> {
        let allowed_ids = self.repository.list_scoped_ids(online_session_user_ids(&sessions), scope).await?;
        let allowed_ids = allowed_ids.into_iter().map(|id| id.0).collect::<HashSet<_>>();
        Ok(sessions.into_iter().filter(|session| allowed_ids.contains(&session.user_id.0)).collect())
    }

    async fn import_users_with_audit(&self, input: UserImportInput, audit: audit_contract::AuditOutboxRecord) -> AppResult<UserImportReport> {
        self.import_users_with_audit_command(input, audit).await
    }

    async fn form_options(&self) -> AppResult<UserFormOptions> {
        self.repository.form_options().await
    }
}

fn active_authenticated_user(user: User) -> AppResult<User> {
    if user.status != constants::system::STATUS_NORMAL {
        return Err(AppError::Unauthorized);
    }
    Ok(user)
}

fn active_authorization_user(user: AuthorizationUser) -> AppResult<AuthorizationUser> {
    if user.status != constants::system::STATUS_NORMAL {
        return Err(AppError::Unauthorized);
    }
    Ok(user)
}

fn online_session_user_ids(sessions: &[OnlineSession]) -> Vec<UserId> {
    let mut seen = HashSet::new();
    sessions
        .iter()
        .filter_map(|session| {
            let id = session.user_id.0.clone();
            seen.insert(id).then(|| session.user_id.clone())
        })
        .collect()
}
