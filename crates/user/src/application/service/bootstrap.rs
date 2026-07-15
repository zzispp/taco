use async_trait::async_trait;
use constants::system::{STATUS_NORMAL, SUPER_ADMIN_ROLE_KEY};

use super::*;
use crate::application::{AdminBootstrapRepository, AdminBootstrapUseCase, BootstrapAdminInput};

#[async_trait]
impl<R, H, P, F, C> AdminBootstrapUseCase for UserService<R, H, P, F, C>
where
    R: UserRepository + AdminBootstrapRepository,
    H: PasswordHasher,
    P: PasswordPolicyProvider,
    F: LoginFailureStore,
    C: LoginLockConfigProvider,
{
    async fn bootstrap_admin(&self, input: BootstrapAdminInput) -> AppResult<User> {
        let role_id = admin_role_id(self.repository.form_options().await?)?;
        let record = self
            .prepare_new_user(NewUser {
                nick_name: input.username.clone(),
                username: input.username,
                password: input.password,
                dept_id: None,
                email: input.email,
                phonenumber: None,
                sex: "2".into(),
                status: STATUS_NORMAL.into(),
                remark: None,
                role_ids: vec![role_id],
                post_ids: vec![],
            })
            .await?;
        self.repository.create_bootstrap_admin(record).await
    }
}

fn admin_role_id(options: UserFormOptions) -> AppResult<String> {
    options
        .roles
        .into_iter()
        .find(|role| role.role_key == SUPER_ADMIN_ROLE_KEY && role.status == STATUS_NORMAL)
        .map(|role| role.role_id)
        .ok_or_else(|| AppError::InvalidInput(localized("errors.user.bootstrap_admin_role_missing")))
}
