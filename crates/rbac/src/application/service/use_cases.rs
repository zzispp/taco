use async_trait::async_trait;
use kernel::pagination::{Page, PageRequest};

use crate::application::{ApiCheckRequest, AuthorizationConfig, RbacAdminUseCase, RbacCache, RbacRepository, RbacResult, RbacService, RbacUseCase};
use crate::domain::{
    ApiPermission, ApiPermissionInput, MenuItem, MenuItemInput, MenuSection, MenuSectionInput, NavResponse, Role, RoleApiBindingInput, RoleInput,
    RoleMenuBindingInput,
};

#[async_trait]
impl<R, C> RbacUseCase for RbacService<R, C>
where
    R: RbacRepository,
    C: RbacCache,
{
    async fn navbar(&self, role_code: &str) -> RbacResult<NavResponse> {
        self.navbar(role_code).await
    }

    async fn authorize_api(&self, config: &AuthorizationConfig, request: ApiCheckRequest) -> RbacResult<()> {
        self.authorize_api(config, request).await
    }

    fn is_whitelisted(&self, config: &AuthorizationConfig, method: &str, path: &str) -> RbacResult<bool> {
        self.is_whitelisted(config, method, path)
    }
}

#[async_trait]
impl<R, C> RbacAdminUseCase for RbacService<R, C>
where
    R: RbacRepository,
    C: RbacCache,
{
    async fn create_role(&self, input: RoleInput) -> RbacResult<Role> {
        self.create_role(input).await
    }

    async fn replace_role(&self, code: &str, input: RoleInput) -> RbacResult<Role> {
        self.replace_role(code, input).await
    }

    async fn delete_role(&self, code: &str) -> RbacResult<()> {
        self.delete_role(code).await
    }

    async fn page_roles(&self, page: PageRequest) -> RbacResult<Page<Role>> {
        self.page_roles(page).await
    }

    async fn create_api(&self, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        self.create_api(input).await
    }

    async fn replace_api(&self, id: &str, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        self.replace_api(id, input).await
    }

    async fn delete_api(&self, id: &str) -> RbacResult<()> {
        self.delete_api(id).await
    }

    async fn page_apis(&self, page: PageRequest) -> RbacResult<Page<ApiPermission>> {
        self.page_apis(page).await
    }

    async fn create_menu_section(&self, input: MenuSectionInput) -> RbacResult<MenuSection> {
        self.create_menu_section(input).await
    }

    async fn replace_menu_section(&self, id: &str, input: MenuSectionInput) -> RbacResult<MenuSection> {
        self.replace_menu_section(id, input).await
    }

    async fn delete_menu_section(&self, id: &str) -> RbacResult<()> {
        self.delete_menu_section(id).await
    }

    async fn page_menu_sections(&self, page: PageRequest) -> RbacResult<Page<MenuSection>> {
        self.page_menu_sections(page).await
    }

    async fn create_menu_item(&self, input: MenuItemInput) -> RbacResult<MenuItem> {
        self.create_menu_item(input).await
    }

    async fn replace_menu_item(&self, id: &str, input: MenuItemInput) -> RbacResult<MenuItem> {
        self.replace_menu_item(id, input).await
    }

    async fn delete_menu_item(&self, id: &str) -> RbacResult<()> {
        self.delete_menu_item(id).await
    }

    async fn page_menu_items(&self, page: PageRequest) -> RbacResult<Page<MenuItem>> {
        self.page_menu_items(page).await
    }

    async fn replace_role_apis(&self, role_code: &str, input: RoleApiBindingInput) -> RbacResult<()> {
        self.replace_role_apis(role_code, input.api_permission_ids).await
    }

    async fn replace_role_menus(&self, role_code: &str, input: RoleMenuBindingInput) -> RbacResult<()> {
        self.replace_role_menus(role_code, input).await
    }

    async fn role_api_ids(&self, role_code: &str) -> RbacResult<Vec<String>> {
        Ok(self.role_api_bindings(role_code).await?.api_permission_ids)
    }

    async fn role_menu_item_ids(&self, role_code: &str) -> RbacResult<Vec<String>> {
        Ok(self.role_menu_bindings(role_code).await?.menu_item_ids)
    }
}
