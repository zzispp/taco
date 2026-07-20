use audit_contract::AuditOutboxRecord;
use types::system::SortBatchInput;

use crate::application::AuditedRbacRepository;

use super::*;

impl<R, C> RbacService<R, C>
where
    R: AuditedRbacRepository,
    C: RbacCache,
{
    pub(super) async fn create_role_with_audit_command(&self, input: RoleInput, audit: AuditOutboxRecord) -> RbacResult<Role> {
        let input = sanitize_role(input)?;
        reject_duplicate_role(&self.repository, &input, None).await?;
        self.repository.create_role_with_audit(input, &audit).await
    }

    pub(super) async fn replace_role_with_audit_command(&self, id: &str, input: RoleInput, audit: AuditOutboxRecord) -> RbacResult<Role> {
        reject_system_role_update(&self.repository, id).await?;
        let input = sanitize_role(input)?;
        reject_duplicate_role(&self.repository, &input, Some(id)).await?;
        self.repository.replace_role_with_audit(id, input, &audit).await
    }

    pub(super) async fn update_role_status_with_audit_command(&self, id: &str, status: String, audit: AuditOutboxRecord) -> RbacResult<Role> {
        reject_system_role_update(&self.repository, id).await?;
        self.repository.update_role_status_with_audit(id, required("status", status)?, &audit).await
    }

    pub(super) async fn update_role_data_scope_with_audit_command(&self, id: &str, input: RoleDataScopeInput, audit: AuditOutboxRecord) -> RbacResult<Role> {
        reject_system_role_update(&self.repository, id).await?;
        let input = sanitize_role_data_scope(input)?;
        self.repository.update_role_data_scope_with_audit(id, input, &audit).await
    }

    pub(super) async fn delete_role_with_audit_command(&self, id: &str, audit: AuditOutboxRecord) -> RbacResult<()> {
        reject_system_role_update(&self.repository, id).await?;
        reject_role_in_use(&self.repository, id).await?;
        self.repository.delete_role_with_audit(id, &audit).await
    }

    pub(super) async fn delete_roles_with_audit_command(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> RbacResult<()> {
        if ids.is_empty() {
            return Err(RbacError::InvalidInput(localized("errors.rbac.ids_required")));
        }
        for id in &ids {
            reject_system_role_update(&self.repository, id).await?;
            reject_role_in_use(&self.repository, id).await?;
        }
        self.repository.delete_roles_with_audit(&ids, &audit).await
    }

    pub(super) async fn replace_role_users_with_audit_command(&self, role_id: &str, input: RoleUserBindingInput, audit: AuditOutboxRecord) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_id).await?;
        let input = sanitize_role_users(input);
        reject_installation_owner_role_mutation(&self.repository, role_id, &input.user_ids, true).await?;
        self.repository.replace_role_users_with_audit(role_id, input, &audit).await
    }

    pub(super) async fn delete_role_user_with_audit_command(&self, role_id: &str, user_id: &str, audit: AuditOutboxRecord) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_id).await?;
        reject_installation_owner_role_mutation(&self.repository, role_id, &[user_id.into()], false).await?;
        self.repository.delete_role_user_with_audit(role_id, user_id, &audit).await
    }

    pub(super) async fn delete_role_users_with_audit_command(&self, role_id: &str, user_ids: Vec<String>, audit: AuditOutboxRecord) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_id).await?;
        let user_ids = clean_ids(user_ids);
        reject_empty_ids(&user_ids)?;
        reject_installation_owner_role_mutation(&self.repository, role_id, &user_ids, false).await?;
        self.repository.delete_role_users_with_audit(role_id, &user_ids, &audit).await
    }

    pub(super) async fn create_menu_with_audit_command(&self, input: MenuInput, audit: AuditOutboxRecord) -> RbacResult<Menu> {
        let input = sanitize_menu(input)?;
        reject_invalid_menu(&self.repository, &input, None).await?;
        self.repository.create_menu_with_audit(input, &audit).await
    }

    pub(super) async fn replace_menu_with_audit_command(&self, id: &str, input: MenuInput, audit: AuditOutboxRecord) -> RbacResult<Menu> {
        ensure_menu_exists(&self.repository, id).await?;
        let input = sanitize_menu(input)?;
        reject_invalid_menu(&self.repository, &input, Some(id)).await?;
        self.repository.replace_menu_with_audit(id, input, &audit).await
    }

    pub(super) async fn update_menu_sort_with_audit_command(&self, id: &str, order_num: i64, audit: AuditOutboxRecord) -> RbacResult<Menu> {
        ensure_menu_exists(&self.repository, id).await?;
        self.repository.update_menu_sort_with_audit(id, order_num, &audit).await
    }

    pub(super) async fn update_menu_sorts_with_audit_command(&self, input: SortBatchInput, audit: AuditOutboxRecord) -> RbacResult<Vec<Menu>> {
        self.repository.update_menu_sorts_with_audit(input, &audit).await
    }

    pub(super) async fn delete_menu_with_audit_command(&self, id: &str, audit: AuditOutboxRecord) -> RbacResult<()> {
        reject_menu_delete(&self.repository, id).await?;
        self.repository.delete_menu_with_audit(id, &audit).await
    }

    pub(super) async fn replace_role_menus_with_audit_command(&self, role_id: &str, input: RoleMenuBindingInput, audit: AuditOutboxRecord) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_id).await?;
        self.repository.replace_role_menus_with_audit(role_id, input, &audit).await?;
        Ok(())
    }

    pub(super) async fn replace_role_depts_with_audit_command(&self, role_id: &str, input: RoleDeptBindingInput, audit: AuditOutboxRecord) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_id).await?;
        self.repository.replace_role_depts_with_audit(role_id, input, &audit).await?;
        Ok(())
    }
}
