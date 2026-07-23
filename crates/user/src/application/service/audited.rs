use audit_contract::AuditOutboxRecord;

use super::*;

impl<R, H, P, F, C> UserService<R, H, P, F, C>
where
    R: AuditedUserRepository,
    H: PasswordHasher,
    P: PasswordPolicyProvider,
    F: LoginFailureStore,
    C: LoginLockConfigProvider,
{
    pub(super) async fn sign_up_with_audit_command(&self, input: NewUser, audit: AuditOutboxRecord) -> AppResult<User> {
        let user = self.prepare_new_user(input).await?;
        self.repository.create_with_audit(user, &audit).await
    }

    pub(super) async fn complete_sign_in_with_audit_command(&self, login: VerifiedLogin, ipaddr: String, audit: AuditOutboxRecord) -> AppResult<User> {
        let user = login.into_user();
        self.login_failures.clear_failures(&user.id).await?;
        self.repository.record_login_with_audit(user.id.clone(), ipaddr, &audit).await?;
        Ok(user)
    }

    pub(super) async fn update_profile_with_audit_command(&self, id: UserId, profile: ProfileUpdate, audit: AuditOutboxRecord) -> AppResult<User> {
        let profile = self.prepare_profile_update(&id, profile).await?;
        self.repository.update_profile_with_audit(id, profile, &audit).await
    }

    pub(super) async fn change_password_with_audit_command(&self, input: AuditedPasswordChange) -> AppResult<()> {
        let hash = self.prepare_password_change(&input.user_id, input.old_password, input.new_password).await?;
        self.repository.update_password_with_audit(input.user_id, hash, &input.audit).await
    }

    pub(super) async fn update_avatar_with_audit_command(&self, id: UserId, avatar: crate::domain::AvatarFileId, audit: AuditOutboxRecord) -> AppResult<User> {
        let avatar = self.prepare_avatar_update(avatar)?;
        self.repository.update_avatar_with_audit(id, avatar, &audit).await
    }

    pub(super) async fn create_user_with_audit_command(&self, input: NewUser, audit: AuditOutboxRecord) -> AppResult<User> {
        let user = self.prepare_new_user(input).await?;
        self.repository.create_with_audit(user, &audit).await
    }

    pub(super) async fn replace_user_with_audit_command(&self, id: UserId, input: ReplaceUser, audit: AuditOutboxRecord) -> AppResult<User> {
        let user = self.prepare_replacement(&id, input).await?;
        self.repository.replace_with_audit(id, user, &audit).await
    }

    pub(super) async fn delete_user_with_audit_command(&self, id: UserId, audit: AuditOutboxRecord) -> AppResult<()> {
        self.repository.delete_with_audit(id, &audit).await
    }

    pub(super) async fn delete_users_with_audit_command(&self, ids: Vec<UserId>, audit: AuditOutboxRecord) -> AppResult<()> {
        self.validate_user_deletions(&ids)?;
        self.repository.delete_many_with_audit(ids, &audit).await
    }

    pub(super) async fn reset_password_with_audit_command(&self, id: UserId, password: String, audit: AuditOutboxRecord) -> AppResult<()> {
        let hash = self.prepare_password_reset(&id, password).await?;
        self.repository.update_password_with_audit(id, hash, &audit).await
    }

    pub(super) async fn update_status_with_audit_command(&self, id: UserId, status: String, audit: AuditOutboxRecord) -> AppResult<User> {
        let status = self.prepare_status_update(status)?;
        self.repository.update_status_with_audit(id, status, &audit).await
    }

    pub(super) async fn replace_roles_with_audit_command(&self, id: UserId, role_ids: Vec<String>, audit: AuditOutboxRecord) -> AppResult<User> {
        let role_ids = self.prepare_role_replacement(role_ids)?;
        self.repository.replace_roles_with_audit(id, role_ids, &audit).await
    }

    pub(super) async fn import_users_with_audit_command(&self, input: UserImportInput, audit: AuditOutboxRecord) -> AppResult<UserImportReport> {
        let prepared = self.prepare_user_import(input).await?;
        self.repository.import_with_audit(prepared.writes, &audit).await?;
        Ok(prepared.report)
    }
}
