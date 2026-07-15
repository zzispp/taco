use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use types::system::SortBatchInput;

use crate::application::{AuditedRbacRepository, RbacAuditedAdminUseCase};

use super::*;

#[async_trait]
impl<R, C> RbacAuditedAdminUseCase for RbacService<R, C>
where
    R: AuditedRbacRepository,
    C: RbacCache,
{
    async fn create_role_with_audit(&self, input: RoleInput, audit: AuditOutboxRecord) -> RbacResult<Role> {
        self.create_role_with_audit_command(input, audit).await
    }

    async fn replace_role_with_audit(&self, role_id: &str, input: RoleInput, audit: AuditOutboxRecord) -> RbacResult<Role> {
        self.replace_role_with_audit_command(role_id, input, audit).await
    }

    async fn update_role_status_with_audit(&self, role_id: &str, status: String, audit: AuditOutboxRecord) -> RbacResult<Role> {
        self.update_role_status_with_audit_command(role_id, status, audit).await
    }

    async fn update_role_data_scope_with_audit(&self, role_id: &str, input: RoleDataScopeInput, audit: AuditOutboxRecord) -> RbacResult<Role> {
        self.update_role_data_scope_with_audit_command(role_id, input, audit).await
    }

    async fn delete_role_with_audit(&self, role_id: &str, audit: AuditOutboxRecord) -> RbacResult<()> {
        self.delete_role_with_audit_command(role_id, audit).await
    }

    async fn delete_roles_with_audit(&self, role_ids: Vec<String>, audit: AuditOutboxRecord) -> RbacResult<()> {
        self.delete_roles_with_audit_command(role_ids, audit).await
    }

    async fn replace_role_users_with_audit(&self, role_id: &str, input: RoleUserBindingInput, audit: AuditOutboxRecord) -> RbacResult<()> {
        self.replace_role_users_with_audit_command(role_id, input, audit).await
    }

    async fn delete_role_user_with_audit(&self, role_id: &str, user_id: &str, audit: AuditOutboxRecord) -> RbacResult<()> {
        self.delete_role_user_with_audit_command(role_id, user_id, audit).await
    }

    async fn delete_role_users_with_audit(&self, role_id: &str, user_ids: Vec<String>, audit: AuditOutboxRecord) -> RbacResult<()> {
        self.delete_role_users_with_audit_command(role_id, user_ids, audit).await
    }

    async fn create_menu_with_audit(&self, input: MenuInput, audit: AuditOutboxRecord) -> RbacResult<Menu> {
        self.create_menu_with_audit_command(input, audit).await
    }

    async fn replace_menu_with_audit(&self, menu_id: &str, input: MenuInput, audit: AuditOutboxRecord) -> RbacResult<Menu> {
        self.replace_menu_with_audit_command(menu_id, input, audit).await
    }

    async fn update_menu_sort_with_audit(&self, menu_id: &str, order_num: i64, audit: AuditOutboxRecord) -> RbacResult<Menu> {
        self.update_menu_sort_with_audit_command(menu_id, order_num, audit).await
    }

    async fn update_menu_sorts_with_audit(&self, input: SortBatchInput, audit: AuditOutboxRecord) -> RbacResult<Vec<Menu>> {
        self.update_menu_sorts_with_audit_command(input, audit).await
    }

    async fn delete_menu_with_audit(&self, menu_id: &str, audit: AuditOutboxRecord) -> RbacResult<()> {
        self.delete_menu_with_audit_command(menu_id, audit).await
    }

    async fn replace_role_menus_with_audit(&self, role_id: &str, input: RoleMenuBindingInput, audit: AuditOutboxRecord) -> RbacResult<()> {
        self.replace_role_menus_with_audit_command(role_id, input, audit).await
    }

    async fn replace_role_depts_with_audit(&self, role_id: &str, input: RoleDeptBindingInput, audit: AuditOutboxRecord) -> RbacResult<()> {
        self.replace_role_depts_with_audit_command(role_id, input, audit).await
    }
}
