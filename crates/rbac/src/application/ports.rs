use async_trait::async_trait;
use kernel::pagination::{Page, PageRequest};

use crate::api::CurrentUser;
use crate::domain::{
    DataScopeFilter, Menu, MenuInput, NavResponse, PermissionSnapshot, Role, RoleDataScopeInput, RoleDeptBindingInput, RoleInput, RoleMenuBindingInput,
    RoleOption, RoleUser, RoleUserBindingInput,
};
use types::system::SortBatchInput;

use super::RbacResult;

#[async_trait]
pub trait RbacRepository: Send + Sync + 'static {
    async fn create_role(&self, input: RoleInput) -> RbacResult<Role>;
    async fn replace_role(&self, role_id: &str, input: RoleInput) -> RbacResult<Role>;
    async fn update_role_status(&self, role_id: &str, status: String) -> RbacResult<Role>;
    async fn update_role_data_scope(&self, role_id: &str, input: RoleDataScopeInput) -> RbacResult<Role>;
    async fn delete_role(&self, role_id: &str) -> RbacResult<()>;
    async fn delete_roles(&self, role_ids: &[String]) -> RbacResult<()>;
    async fn find_role(&self, role_id: &str) -> RbacResult<Option<Role>>;
    async fn role_name_exists(&self, name: &str, current_id: Option<&str>) -> RbacResult<bool>;
    async fn role_key_exists(&self, key: &str, current_id: Option<&str>) -> RbacResult<bool>;
    async fn role_has_users(&self, role_id: &str) -> RbacResult<bool>;
    async fn page_roles(&self, filter: RoleListFilter) -> RbacResult<Page<Role>>;
    async fn page_roles_scoped(&self, filter: RoleListFilter, scope: DataScopeFilter) -> RbacResult<Page<Role>>;
    async fn role_options(&self) -> RbacResult<Vec<RoleOption>>;
    async fn page_role_users(&self, filter: RoleUserListFilter, scope: Option<DataScopeFilter>) -> RbacResult<Page<RoleUser>>;
    async fn scoped_user_ids(&self, user_ids: &[String], scope: DataScopeFilter) -> RbacResult<Vec<String>>;
    async fn replace_role_users(&self, role_id: &str, input: RoleUserBindingInput) -> RbacResult<()>;
    async fn delete_role_user(&self, role_id: &str, user_id: &str) -> RbacResult<()>;
    async fn delete_role_users(&self, role_id: &str, user_ids: &[String]) -> RbacResult<()>;
    async fn create_menu(&self, input: MenuInput) -> RbacResult<Menu>;
    async fn replace_menu(&self, menu_id: &str, input: MenuInput) -> RbacResult<Menu>;
    async fn update_menu_sort(&self, menu_id: &str, order_num: i64) -> RbacResult<Menu>;
    async fn update_menu_sorts(&self, input: SortBatchInput) -> RbacResult<Vec<Menu>>;
    async fn delete_menu(&self, menu_id: &str) -> RbacResult<()>;
    async fn find_menu(&self, menu_id: &str) -> RbacResult<Option<Menu>>;
    async fn menu_has_children(&self, menu_id: &str) -> RbacResult<bool>;
    async fn menu_has_role_bindings(&self, menu_id: &str) -> RbacResult<bool>;
    async fn list_menus(&self) -> RbacResult<Vec<Menu>>;
    async fn page_menus(&self, filter: MenuListFilter) -> RbacResult<Page<Menu>>;
    async fn replace_role_menus(&self, role_id: &str, input: RoleMenuBindingInput) -> RbacResult<()>;
    async fn replace_role_depts(&self, role_id: &str, input: RoleDeptBindingInput) -> RbacResult<()>;
    async fn role_menu_ids(&self, role_id: &str) -> RbacResult<Vec<String>>;
    async fn role_dept_ids(&self, role_id: &str) -> RbacResult<Vec<String>>;
    async fn permission_snapshot(&self) -> RbacResult<PermissionSnapshot>;
}

#[async_trait]
pub trait RbacCache: Send + Sync + 'static {
    async fn write_snapshot(&self, snapshot: &PermissionSnapshot) -> RbacResult<()>;
    async fn read_snapshot(&self) -> RbacResult<PermissionSnapshot>;
    async fn read_nav(&self, role_keys: &[String], admin: bool) -> RbacResult<NavResponse>;
}

#[async_trait]
pub trait RbacUseCase: Send + Sync + 'static {
    async fn navbar(&self, current_user: &CurrentUser) -> RbacResult<NavResponse>;
    async fn authorize_api(&self, config: &AuthorizationConfig, request: ApiCheckRequest) -> RbacResult<()>;
    async fn data_scope_filter(&self, current_user: &CurrentUser) -> RbacResult<DataScopeFilter>;
    fn validate_protected_handlers(&self, config: &AuthorizationConfig) -> RbacResult<()>;
    fn validate_data_scope_handlers(&self, handlers: &[&str]) -> RbacResult<()>;
    fn is_whitelisted(&self, config: &AuthorizationConfig, method: &str, path: &str) -> RbacResult<bool>;
}

#[async_trait]
pub trait RbacAdminUseCase: Send + Sync + 'static {
    async fn create_role(&self, input: RoleInput) -> RbacResult<Role>;
    async fn replace_role(&self, role_id: &str, input: RoleInput) -> RbacResult<Role>;
    async fn update_role_status(&self, role_id: &str, status: String) -> RbacResult<Role>;
    async fn update_role_data_scope(&self, role_id: &str, input: RoleDataScopeInput) -> RbacResult<Role>;
    async fn delete_role(&self, role_id: &str) -> RbacResult<()>;
    async fn delete_roles(&self, role_ids: Vec<String>) -> RbacResult<()>;
    async fn get_role(&self, role_id: &str) -> RbacResult<Role>;
    async fn page_roles(&self, filter: RoleListFilter) -> RbacResult<Page<Role>>;
    async fn page_roles_scoped(&self, filter: RoleListFilter, scope: DataScopeFilter) -> RbacResult<Page<Role>>;
    async fn role_options(&self) -> RbacResult<Vec<RoleOption>>;
    async fn page_role_users(&self, filter: RoleUserListFilter, scope: Option<DataScopeFilter>) -> RbacResult<Page<RoleUser>>;
    async fn ensure_user_ids_scoped(&self, user_ids: Vec<String>, scope: DataScopeFilter) -> RbacResult<()>;
    async fn replace_role_users(&self, role_id: &str, input: RoleUserBindingInput) -> RbacResult<()>;
    async fn delete_role_user(&self, role_id: &str, user_id: &str) -> RbacResult<()>;
    async fn delete_role_users(&self, role_id: &str, user_ids: Vec<String>) -> RbacResult<()>;
    async fn create_menu(&self, input: MenuInput) -> RbacResult<Menu>;
    async fn replace_menu(&self, menu_id: &str, input: MenuInput) -> RbacResult<Menu>;
    async fn update_menu_sort(&self, menu_id: &str, order_num: i64) -> RbacResult<Menu>;
    async fn update_menu_sorts(&self, input: SortBatchInput) -> RbacResult<Vec<Menu>>;
    async fn delete_menu(&self, menu_id: &str) -> RbacResult<()>;
    async fn get_menu(&self, menu_id: &str) -> RbacResult<Menu>;
    async fn page_menus(&self, filter: MenuListFilter) -> RbacResult<Page<Menu>>;
    async fn list_menus(&self) -> RbacResult<Vec<Menu>>;
    async fn replace_role_menus(&self, role_id: &str, input: RoleMenuBindingInput) -> RbacResult<()>;
    async fn replace_role_depts(&self, role_id: &str, input: RoleDeptBindingInput) -> RbacResult<()>;
    async fn role_menu_ids(&self, role_id: &str) -> RbacResult<Vec<String>>;
    async fn role_dept_ids(&self, role_id: &str) -> RbacResult<Vec<String>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleListFilter {
    pub page: PageRequest,
    pub role_name: Option<String>,
    pub role_key: Option<String>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuListFilter {
    pub page: PageRequest,
    pub menu_name: Option<String>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleUserListFilter {
    pub page: PageRequest,
    pub role_id: String,
    pub username: Option<String>,
    pub phonenumber: Option<String>,
    pub allocated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiCheckRequest {
    pub method: String,
    pub path: String,
    pub role_keys: Vec<String>,
    pub permissions: Vec<String>,
    pub admin: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthWhitelistRule {
    pub methods: Vec<String>,
    pub path_pattern: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizationConfig {
    pub whitelist: Vec<AuthWhitelistRule>,
    pub route_permissions: Vec<crate::domain::RoutePermissionRule>,
}
