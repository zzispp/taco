use async_trait::async_trait;
use kernel::pagination::Page;
use storage::Database;

use crate::{
    application::{MenuListFilter, RbacRepository, RbacResult, RoleListFilter, RoleUserListFilter},
    domain::{
        DataScopeFilter, Menu, MenuInput, PermissionSnapshot, Role, RoleDataScopeInput, RoleDeptBindingInput, RoleInput, RoleMenuBindingInput, RoleOption,
        RoleUser, RoleUserBindingInput,
    },
};
use types::system::SortBatchInput;

use super::{
    mapping::{permission_snapshot, storage_error},
    menu_queries::MenuQueries,
    role_queries::RoleQueries,
};

#[derive(Clone)]
pub struct StorageRbacRepository {
    roles: RoleQueries,
    menus: MenuQueries,
}

impl StorageRbacRepository {
    pub fn new(database: Database) -> Self {
        Self {
            roles: RoleQueries::new(database.clone()),
            menus: MenuQueries::new(database),
        }
    }
}

#[async_trait]
impl RbacRepository for StorageRbacRepository {
    async fn create_role(&self, input: RoleInput) -> RbacResult<Role> {
        self.roles.create(input).await.map_err(storage_error)
    }

    async fn replace_role(&self, role_id: &str, input: RoleInput) -> RbacResult<Role> {
        self.roles.replace(role_id, input).await.map_err(storage_error)
    }

    async fn update_role_status(&self, role_id: &str, status: String) -> RbacResult<Role> {
        self.roles.update_status(role_id, status).await.map_err(storage_error)
    }

    async fn update_role_data_scope(&self, role_id: &str, input: RoleDataScopeInput) -> RbacResult<Role> {
        self.roles.update_data_scope(role_id, input).await.map_err(storage_error)
    }

    async fn delete_role(&self, role_id: &str) -> RbacResult<()> {
        self.roles.delete(role_id).await.map_err(storage_error)
    }

    async fn delete_roles(&self, role_ids: &[String]) -> RbacResult<()> {
        self.roles.delete_many(role_ids).await.map_err(storage_error)
    }

    async fn find_role(&self, role_id: &str) -> RbacResult<Option<Role>> {
        self.roles.find(role_id).await.map_err(storage_error)
    }

    async fn role_name_exists(&self, name: &str, current_id: Option<&str>) -> RbacResult<bool> {
        self.roles.role_name_exists(name, current_id).await.map_err(storage_error)
    }

    async fn role_key_exists(&self, key: &str, current_id: Option<&str>) -> RbacResult<bool> {
        self.roles.role_key_exists(key, current_id).await.map_err(storage_error)
    }

    async fn role_has_users(&self, role_id: &str) -> RbacResult<bool> {
        self.roles.has_users(role_id).await.map_err(storage_error)
    }

    async fn page_roles(&self, filter: RoleListFilter) -> RbacResult<Page<Role>> {
        self.roles.page(filter).await.map_err(storage_error)
    }

    async fn page_roles_scoped(&self, filter: RoleListFilter, scope: DataScopeFilter) -> RbacResult<Page<Role>> {
        self.roles.page_scoped(filter, scope).await.map_err(storage_error)
    }

    async fn role_options(&self) -> RbacResult<Vec<RoleOption>> {
        self.roles.options().await.map_err(storage_error)
    }

    async fn page_role_users(&self, filter: RoleUserListFilter, scope: Option<DataScopeFilter>) -> RbacResult<Page<RoleUser>> {
        self.roles.page_users(filter, scope).await.map_err(storage_error)
    }

    async fn scoped_user_ids(&self, user_ids: &[String], scope: DataScopeFilter) -> RbacResult<Vec<String>> {
        self.roles.scoped_user_ids(user_ids, scope).await.map_err(storage_error)
    }

    async fn replace_role_users(&self, role_id: &str, input: RoleUserBindingInput) -> RbacResult<()> {
        self.roles.replace_users(role_id, input).await.map_err(storage_error)
    }

    async fn delete_role_user(&self, role_id: &str, user_id: &str) -> RbacResult<()> {
        self.roles.delete_user(role_id, user_id).await.map_err(storage_error)
    }

    async fn delete_role_users(&self, role_id: &str, user_ids: &[String]) -> RbacResult<()> {
        self.roles.delete_users(role_id, user_ids).await.map_err(storage_error)
    }

    async fn create_menu(&self, input: MenuInput) -> RbacResult<Menu> {
        self.menus.create(input).await.map_err(storage_error)
    }

    async fn replace_menu(&self, menu_id: &str, input: MenuInput) -> RbacResult<Menu> {
        self.menus.replace(menu_id, input).await.map_err(storage_error)
    }

    async fn update_menu_sort(&self, menu_id: &str, order_num: i64) -> RbacResult<Menu> {
        self.menus.update_sort(menu_id, order_num).await.map_err(storage_error)
    }

    async fn update_menu_sorts(&self, input: SortBatchInput) -> RbacResult<Vec<Menu>> {
        self.menus.update_sorts(input).await.map_err(storage_error)
    }

    async fn delete_menu(&self, menu_id: &str) -> RbacResult<()> {
        self.menus.delete(menu_id).await.map_err(storage_error)
    }

    async fn find_menu(&self, menu_id: &str) -> RbacResult<Option<Menu>> {
        self.menus.find(menu_id).await.map_err(storage_error)
    }

    async fn menu_has_children(&self, menu_id: &str) -> RbacResult<bool> {
        self.menus.has_children(menu_id).await.map_err(storage_error)
    }

    async fn menu_has_role_bindings(&self, menu_id: &str) -> RbacResult<bool> {
        self.menus.has_role_bindings(menu_id).await.map_err(storage_error)
    }

    async fn list_menus(&self) -> RbacResult<Vec<Menu>> {
        self.menus.list().await.map_err(storage_error)
    }

    async fn page_menus(&self, filter: MenuListFilter) -> RbacResult<Page<Menu>> {
        self.menus.page(filter).await.map_err(storage_error)
    }

    async fn replace_role_menus(&self, role_id: &str, input: RoleMenuBindingInput) -> RbacResult<()> {
        self.roles.replace_menus(role_id, input).await.map_err(storage_error)
    }

    async fn replace_role_depts(&self, role_id: &str, input: RoleDeptBindingInput) -> RbacResult<()> {
        self.roles.replace_depts(role_id, input).await.map_err(storage_error)
    }

    async fn role_menu_ids(&self, role_id: &str) -> RbacResult<Vec<String>> {
        self.roles.menu_ids(role_id).await.map_err(storage_error)
    }

    async fn role_dept_ids(&self, role_id: &str) -> RbacResult<Vec<String>> {
        self.roles.dept_ids(role_id).await.map_err(storage_error)
    }

    async fn permission_snapshot(&self) -> RbacResult<PermissionSnapshot> {
        let permissions = self.roles.permission_rows().await.map_err(storage_error)?;
        let depts = self.roles.dept_rows().await.map_err(storage_error)?;
        let menus = self.menus.role_menu_rows().await.map_err(storage_error)?;
        Ok(permission_snapshot(permissions, depts, menus))
    }
}
