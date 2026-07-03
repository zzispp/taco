use async_trait::async_trait;
use kernel::pagination::{Page, PageRequest};

use super::RbacResult;
use crate::domain::{
    ApiPermission, ApiPermissionInput, MenuItem, MenuItemInput, MenuSection, MenuSectionInput, NavResponse, PermissionSnapshot, Role, RoleApiBindingInput,
    RoleInput, RoleMenuBindingInput,
};

/// Persists RBAC roles, API permissions, menus, and bindings.
#[async_trait]
pub trait RbacRepository: Send + Sync + 'static {
    async fn create_role(&self, input: RoleInput) -> RbacResult<Role>;
    async fn create_system_role(&self, input: RoleInput) -> RbacResult<Role>;
    async fn replace_role(&self, code: &str, input: RoleInput) -> RbacResult<Role>;
    async fn replace_system_role(&self, code: &str, input: RoleInput) -> RbacResult<Role>;
    async fn delete_role(&self, code: &str) -> RbacResult<()>;
    async fn find_role(&self, code: &str) -> RbacResult<Option<Role>>;
    async fn role_has_api_bindings(&self, code: &str) -> RbacResult<bool>;
    async fn role_has_menu_bindings(&self, code: &str) -> RbacResult<bool>;
    async fn role_has_users(&self, code: &str) -> RbacResult<bool>;
    async fn list_roles(&self) -> RbacResult<Vec<Role>>;
    async fn page_roles(&self, page: PageRequest) -> RbacResult<Page<Role>>;
    async fn create_api(&self, input: ApiPermissionInput) -> RbacResult<ApiPermission>;
    async fn replace_api(&self, id: &str, input: ApiPermissionInput) -> RbacResult<ApiPermission>;
    async fn delete_api(&self, id: &str) -> RbacResult<()>;
    async fn find_api(&self, id: &str) -> RbacResult<Option<ApiPermission>>;
    async fn api_has_role_bindings(&self, id: &str) -> RbacResult<bool>;
    async fn list_apis(&self) -> RbacResult<Vec<ApiPermission>>;
    async fn page_apis(&self, page: PageRequest) -> RbacResult<Page<ApiPermission>>;
    async fn create_menu_section(&self, input: MenuSectionInput) -> RbacResult<MenuSection>;
    async fn replace_menu_section(&self, id: &str, input: MenuSectionInput) -> RbacResult<MenuSection>;
    async fn delete_menu_section(&self, id: &str) -> RbacResult<()>;
    async fn find_menu_section(&self, id: &str) -> RbacResult<Option<MenuSection>>;
    async fn menu_section_has_items(&self, id: &str) -> RbacResult<bool>;
    async fn page_menu_sections(&self, page: PageRequest) -> RbacResult<Page<MenuSection>>;
    async fn create_menu_item(&self, input: MenuItemInput) -> RbacResult<MenuItem>;
    async fn replace_menu_item(&self, id: &str, input: MenuItemInput) -> RbacResult<MenuItem>;
    async fn delete_menu_item(&self, id: &str) -> RbacResult<()>;
    async fn find_menu_item(&self, id: &str) -> RbacResult<Option<MenuItem>>;
    async fn menu_item_has_children(&self, id: &str) -> RbacResult<bool>;
    async fn menu_item_has_role_bindings(&self, id: &str) -> RbacResult<bool>;
    async fn list_menu_items(&self) -> RbacResult<Vec<MenuItem>>;
    async fn page_menu_items(&self, page: PageRequest) -> RbacResult<Page<MenuItem>>;
    async fn replace_role_apis(&self, role_code: &str, api_permission_ids: Vec<String>) -> RbacResult<()>;
    async fn replace_role_menus(&self, role_code: &str, input: RoleMenuBindingInput) -> RbacResult<()>;
    async fn role_api_ids(&self, role_code: &str) -> RbacResult<Vec<String>>;
    async fn role_menu_item_ids(&self, role_code: &str) -> RbacResult<Vec<String>>;
    async fn permission_snapshot(&self) -> RbacResult<PermissionSnapshot>;
}

/// Stores and reads RBAC cache snapshots. Missing cache data is an explicit infrastructure error.
#[async_trait]
pub trait RbacCache: Send + Sync + 'static {
    async fn write_snapshot(&self, snapshot: &PermissionSnapshot) -> RbacResult<()>;
    async fn read_snapshot(&self) -> RbacResult<PermissionSnapshot>;
    async fn read_nav(&self, role_code: &str) -> RbacResult<NavResponse>;
}

#[async_trait]
pub trait RbacUseCase: Send + Sync + 'static {
    async fn navbar(&self, role_code: &str) -> RbacResult<NavResponse>;
    async fn authorize_api(&self, config: &AuthorizationConfig, request: ApiCheckRequest) -> RbacResult<()>;
    fn is_whitelisted(&self, config: &AuthorizationConfig, method: &str, path: &str) -> RbacResult<bool>;
}

#[async_trait]
pub trait RbacAdminUseCase: Send + Sync + 'static {
    async fn create_role(&self, input: RoleInput) -> RbacResult<Role>;
    async fn replace_role(&self, code: &str, input: RoleInput) -> RbacResult<Role>;
    async fn delete_role(&self, code: &str) -> RbacResult<()>;
    async fn page_roles(&self, page: PageRequest) -> RbacResult<Page<Role>>;
    async fn create_api(&self, input: ApiPermissionInput) -> RbacResult<ApiPermission>;
    async fn replace_api(&self, id: &str, input: ApiPermissionInput) -> RbacResult<ApiPermission>;
    async fn delete_api(&self, id: &str) -> RbacResult<()>;
    async fn page_apis(&self, page: PageRequest) -> RbacResult<Page<ApiPermission>>;
    async fn create_menu_section(&self, input: MenuSectionInput) -> RbacResult<MenuSection>;
    async fn replace_menu_section(&self, id: &str, input: MenuSectionInput) -> RbacResult<MenuSection>;
    async fn delete_menu_section(&self, id: &str) -> RbacResult<()>;
    async fn page_menu_sections(&self, page: PageRequest) -> RbacResult<Page<MenuSection>>;
    async fn create_menu_item(&self, input: MenuItemInput) -> RbacResult<MenuItem>;
    async fn replace_menu_item(&self, id: &str, input: MenuItemInput) -> RbacResult<MenuItem>;
    async fn delete_menu_item(&self, id: &str) -> RbacResult<()>;
    async fn page_menu_items(&self, page: PageRequest) -> RbacResult<Page<MenuItem>>;
    async fn replace_role_apis(&self, role_code: &str, input: RoleApiBindingInput) -> RbacResult<()>;
    async fn replace_role_menus(&self, role_code: &str, input: RoleMenuBindingInput) -> RbacResult<()>;
    async fn role_api_ids(&self, role_code: &str) -> RbacResult<Vec<String>>;
    async fn role_menu_item_ids(&self, role_code: &str) -> RbacResult<Vec<String>>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiCheckRequest {
    pub method: String,
    pub path: String,
    pub role_code: String,
    pub system: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthWhitelistRule {
    pub methods: Vec<String>,
    pub path_pattern: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizationConfig {
    pub whitelist: Vec<AuthWhitelistRule>,
}
