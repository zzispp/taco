use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use types::system::SortBatchInput;

use crate::{
    application::{RbacRepository, RbacResult},
    domain::{Menu, MenuInput, Role, RoleDataScopeInput, RoleDeptBindingInput, RoleInput, RoleMenuBindingInput, RoleUserBindingInput},
};

/// Persists RBAC state changes and their immutable operation-audit record in one
/// PostgreSQL transaction. There is intentionally no unaudited default for these
/// commands: management handlers must choose this port for successful writes.
#[async_trait]
pub trait AuditedRbacRepository: RbacRepository {
    async fn create_role_with_audit(&self, input: RoleInput, audit: &AuditOutboxRecord) -> RbacResult<Role>;
    async fn replace_role_with_audit(&self, role_id: &str, input: RoleInput, audit: &AuditOutboxRecord) -> RbacResult<Role>;
    async fn update_role_status_with_audit(&self, role_id: &str, status: String, audit: &AuditOutboxRecord) -> RbacResult<Role>;
    async fn update_role_data_scope_with_audit(&self, role_id: &str, input: RoleDataScopeInput, audit: &AuditOutboxRecord) -> RbacResult<Role>;
    async fn delete_role_with_audit(&self, role_id: &str, audit: &AuditOutboxRecord) -> RbacResult<()>;
    async fn delete_roles_with_audit(&self, role_ids: &[String], audit: &AuditOutboxRecord) -> RbacResult<()>;
    async fn replace_role_users_with_audit(&self, role_id: &str, input: RoleUserBindingInput, audit: &AuditOutboxRecord) -> RbacResult<()>;
    async fn delete_role_user_with_audit(&self, role_id: &str, user_id: &str, audit: &AuditOutboxRecord) -> RbacResult<()>;
    async fn delete_role_users_with_audit(&self, role_id: &str, user_ids: &[String], audit: &AuditOutboxRecord) -> RbacResult<()>;
    async fn create_menu_with_audit(&self, input: MenuInput, audit: &AuditOutboxRecord) -> RbacResult<Menu>;
    async fn replace_menu_with_audit(&self, menu_id: &str, input: MenuInput, audit: &AuditOutboxRecord) -> RbacResult<Menu>;
    async fn update_menu_sort_with_audit(&self, menu_id: &str, order_num: i64, audit: &AuditOutboxRecord) -> RbacResult<Menu>;
    async fn update_menu_sorts_with_audit(&self, input: SortBatchInput, audit: &AuditOutboxRecord) -> RbacResult<Vec<Menu>>;
    async fn delete_menu_with_audit(&self, menu_id: &str, audit: &AuditOutboxRecord) -> RbacResult<()>;
    async fn replace_role_menus_with_audit(&self, role_id: &str, input: RoleMenuBindingInput, audit: &AuditOutboxRecord) -> RbacResult<()>;
    async fn replace_role_depts_with_audit(&self, role_id: &str, input: RoleDeptBindingInput, audit: &AuditOutboxRecord) -> RbacResult<()>;
}

/// Management write use cases with a required immutable operation-audit record.
///
/// Kept separate from [`super::RbacAdminUseCase`] so read-only test doubles and
/// integrations cannot accidentally advertise transactional-audit support.
#[async_trait]
pub trait RbacAuditedAdminUseCase: Send + Sync + 'static {
    async fn create_role_with_audit(&self, input: RoleInput, audit: AuditOutboxRecord) -> RbacResult<Role>;
    async fn replace_role_with_audit(&self, role_id: &str, input: RoleInput, audit: AuditOutboxRecord) -> RbacResult<Role>;
    async fn update_role_status_with_audit(&self, role_id: &str, status: String, audit: AuditOutboxRecord) -> RbacResult<Role>;
    async fn update_role_data_scope_with_audit(&self, role_id: &str, input: RoleDataScopeInput, audit: AuditOutboxRecord) -> RbacResult<Role>;
    async fn delete_role_with_audit(&self, role_id: &str, audit: AuditOutboxRecord) -> RbacResult<()>;
    async fn delete_roles_with_audit(&self, role_ids: Vec<String>, audit: AuditOutboxRecord) -> RbacResult<()>;
    async fn replace_role_users_with_audit(&self, role_id: &str, input: RoleUserBindingInput, audit: AuditOutboxRecord) -> RbacResult<()>;
    async fn delete_role_user_with_audit(&self, role_id: &str, user_id: &str, audit: AuditOutboxRecord) -> RbacResult<()>;
    async fn delete_role_users_with_audit(&self, role_id: &str, user_ids: Vec<String>, audit: AuditOutboxRecord) -> RbacResult<()>;
    async fn create_menu_with_audit(&self, input: MenuInput, audit: AuditOutboxRecord) -> RbacResult<Menu>;
    async fn replace_menu_with_audit(&self, menu_id: &str, input: MenuInput, audit: AuditOutboxRecord) -> RbacResult<Menu>;
    async fn update_menu_sort_with_audit(&self, menu_id: &str, order_num: i64, audit: AuditOutboxRecord) -> RbacResult<Menu>;
    async fn update_menu_sorts_with_audit(&self, input: SortBatchInput, audit: AuditOutboxRecord) -> RbacResult<Vec<Menu>>;
    async fn delete_menu_with_audit(&self, menu_id: &str, audit: AuditOutboxRecord) -> RbacResult<()>;
    async fn replace_role_menus_with_audit(&self, role_id: &str, input: RoleMenuBindingInput, audit: AuditOutboxRecord) -> RbacResult<()>;
    async fn replace_role_depts_with_audit(&self, role_id: &str, input: RoleDeptBindingInput, audit: AuditOutboxRecord) -> RbacResult<()>;
}

/// Refreshes RBAC's Redis projection after an audited PostgreSQL transaction
/// has committed. Keeping this separate prevents a post-commit Redis error
/// from being misclassified as a failed transactional write.
#[async_trait]
pub trait RbacCacheRefreshUseCase: Send + Sync + 'static {
    async fn refresh_after_audited_write(&self) -> RbacResult<()>;
}
