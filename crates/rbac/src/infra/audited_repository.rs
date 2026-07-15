use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use types::system::SortBatchInput;

use crate::{
    application::{AuditedRbacRepository, RbacResult},
    domain::{Menu, MenuInput, Role, RoleDataScopeInput, RoleDeptBindingInput, RoleInput, RoleMenuBindingInput, RoleUserBindingInput},
};

use super::{StorageRbacRepository, mapping::storage_error};

#[async_trait]
impl AuditedRbacRepository for StorageRbacRepository {
    async fn create_role_with_audit(&self, input: RoleInput, audit: &AuditOutboxRecord) -> RbacResult<Role> {
        self.roles.create_with_audit(input, audit).await.map_err(storage_error)
    }

    async fn replace_role_with_audit(&self, role_id: &str, input: RoleInput, audit: &AuditOutboxRecord) -> RbacResult<Role> {
        self.roles.replace_with_audit(role_id, input, audit).await.map_err(storage_error)
    }

    async fn update_role_status_with_audit(&self, role_id: &str, status: String, audit: &AuditOutboxRecord) -> RbacResult<Role> {
        self.roles.update_status_with_audit(role_id, status, audit).await.map_err(storage_error)
    }

    async fn update_role_data_scope_with_audit(&self, role_id: &str, input: RoleDataScopeInput, audit: &AuditOutboxRecord) -> RbacResult<Role> {
        self.roles.update_data_scope_with_audit(role_id, input, audit).await.map_err(storage_error)
    }

    async fn delete_role_with_audit(&self, role_id: &str, audit: &AuditOutboxRecord) -> RbacResult<()> {
        self.roles.delete_with_audit(role_id, audit).await.map_err(storage_error)
    }

    async fn delete_roles_with_audit(&self, role_ids: &[String], audit: &AuditOutboxRecord) -> RbacResult<()> {
        self.roles.delete_many_with_audit(role_ids, audit).await.map_err(storage_error)
    }

    async fn replace_role_users_with_audit(&self, role_id: &str, input: RoleUserBindingInput, audit: &AuditOutboxRecord) -> RbacResult<()> {
        self.roles.replace_users_with_audit(role_id, input, audit).await.map_err(storage_error)
    }

    async fn delete_role_user_with_audit(&self, role_id: &str, user_id: &str, audit: &AuditOutboxRecord) -> RbacResult<()> {
        self.roles.delete_user_with_audit(role_id, user_id, audit).await.map_err(storage_error)
    }

    async fn delete_role_users_with_audit(&self, role_id: &str, user_ids: &[String], audit: &AuditOutboxRecord) -> RbacResult<()> {
        self.roles.delete_users_with_audit(role_id, user_ids, audit).await.map_err(storage_error)
    }

    async fn create_menu_with_audit(&self, input: MenuInput, audit: &AuditOutboxRecord) -> RbacResult<Menu> {
        self.menus.create_with_audit(input, audit).await.map_err(storage_error)
    }

    async fn replace_menu_with_audit(&self, menu_id: &str, input: MenuInput, audit: &AuditOutboxRecord) -> RbacResult<Menu> {
        self.menus.replace_with_audit(menu_id, input, audit).await.map_err(storage_error)
    }

    async fn update_menu_sort_with_audit(&self, menu_id: &str, order_num: i64, audit: &AuditOutboxRecord) -> RbacResult<Menu> {
        self.menus.update_sort_with_audit(menu_id, order_num, audit).await.map_err(storage_error)
    }

    async fn update_menu_sorts_with_audit(&self, input: SortBatchInput, audit: &AuditOutboxRecord) -> RbacResult<Vec<Menu>> {
        self.menus.update_sorts_with_audit(input, audit).await.map_err(storage_error)
    }

    async fn delete_menu_with_audit(&self, menu_id: &str, audit: &AuditOutboxRecord) -> RbacResult<()> {
        self.menus.delete_with_audit(menu_id, audit).await.map_err(storage_error)
    }

    async fn replace_role_menus_with_audit(&self, role_id: &str, input: RoleMenuBindingInput, audit: &AuditOutboxRecord) -> RbacResult<()> {
        self.roles.replace_menus_with_audit(role_id, input, audit).await.map_err(storage_error)
    }

    async fn replace_role_depts_with_audit(&self, role_id: &str, input: RoleDeptBindingInput, audit: &AuditOutboxRecord) -> RbacResult<()> {
        self.roles.replace_depts_with_audit(role_id, input, audit).await.map_err(storage_error)
    }
}
