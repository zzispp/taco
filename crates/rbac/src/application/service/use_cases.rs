use async_trait::async_trait;
use kernel::pagination::CursorPage;

use crate::api::CurrentUser;
use crate::application::{
    ApiCheckRequest, AuthorizationConfig, MenuListFilter, RbacAdminUseCase, RbacCache, RbacRepository, RbacResult, RbacService, RbacUseCase, RoleExportRequest,
    RoleExportSink, RoleListFilter, RoleUserListFilter,
};
use crate::domain::{
    DataScopeFilter, Menu, MenuInput, NavResponse, Role, RoleDataScopeInput, RoleDeptBindingInput, RoleInput, RoleMenuBindingInput, RoleOption, RoleUser,
    RoleUserBindingInput,
};
use types::system::SortBatchInput;

#[async_trait]
impl<R, C> RbacUseCase for RbacService<R, C>
where
    R: RbacRepository,
    C: RbacCache,
{
    async fn navbar(&self, current_user: &CurrentUser) -> RbacResult<NavResponse> {
        self.navbar(current_user).await
    }

    async fn authorize_api(&self, config: &AuthorizationConfig, request: ApiCheckRequest) -> RbacResult<()> {
        self.authorize_api(config, request).await
    }

    async fn data_scope_filter(&self, current_user: &CurrentUser) -> RbacResult<DataScopeFilter> {
        self.data_scope_filter(current_user).await
    }

    fn validate_protected_handlers(&self, config: &AuthorizationConfig) -> RbacResult<()> {
        self.validate_protected_handlers(config)
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

    async fn replace_role(&self, role_id: &str, input: RoleInput) -> RbacResult<Role> {
        self.replace_role(role_id, input).await
    }

    async fn update_role_status(&self, role_id: &str, status: String) -> RbacResult<Role> {
        self.update_role_status(role_id, status).await
    }

    async fn update_role_data_scope(&self, role_id: &str, input: RoleDataScopeInput) -> RbacResult<Role> {
        self.update_role_data_scope(role_id, input).await
    }

    async fn delete_role(&self, role_id: &str) -> RbacResult<()> {
        self.delete_role(role_id).await
    }

    async fn delete_roles(&self, role_ids: Vec<String>) -> RbacResult<()> {
        self.delete_roles(role_ids).await
    }

    async fn get_role(&self, role_id: &str) -> RbacResult<Role> {
        self.get_role(role_id).await
    }

    async fn page_roles(&self, filter: RoleListFilter) -> RbacResult<CursorPage<Role>> {
        self.page_roles(filter).await
    }

    async fn page_roles_scoped(&self, filter: RoleListFilter, scope: DataScopeFilter) -> RbacResult<CursorPage<Role>> {
        self.page_roles_scoped(filter, scope).await
    }

    async fn export_roles(&self, request: RoleExportRequest, sink: &mut dyn RoleExportSink) -> RbacResult<()> {
        self.export_roles(request, sink).await
    }

    async fn role_options(&self) -> RbacResult<Vec<RoleOption>> {
        self.role_options().await
    }

    async fn page_role_users(&self, filter: RoleUserListFilter, scope: Option<DataScopeFilter>) -> RbacResult<CursorPage<RoleUser>> {
        self.page_role_users(filter, scope).await
    }

    async fn ensure_user_ids_scoped(&self, user_ids: Vec<String>, scope: DataScopeFilter) -> RbacResult<()> {
        self.ensure_user_ids_scoped(user_ids, scope).await
    }

    async fn replace_role_users(&self, role_id: &str, input: RoleUserBindingInput) -> RbacResult<()> {
        self.replace_role_users(role_id, input).await
    }

    async fn delete_role_user(&self, role_id: &str, user_id: &str) -> RbacResult<()> {
        self.delete_role_user(role_id, user_id).await
    }

    async fn delete_role_users(&self, role_id: &str, user_ids: Vec<String>) -> RbacResult<()> {
        self.delete_role_users(role_id, user_ids).await
    }

    async fn create_menu(&self, input: MenuInput) -> RbacResult<Menu> {
        self.create_menu(input).await
    }

    async fn replace_menu(&self, menu_id: &str, input: MenuInput) -> RbacResult<Menu> {
        self.replace_menu(menu_id, input).await
    }

    async fn update_menu_sort(&self, menu_id: &str, order_num: i64) -> RbacResult<Menu> {
        self.update_menu_sort(menu_id, order_num).await
    }

    async fn update_menu_sorts(&self, input: SortBatchInput) -> RbacResult<Vec<Menu>> {
        self.update_menu_sorts(input).await
    }

    async fn delete_menu(&self, menu_id: &str) -> RbacResult<()> {
        self.delete_menu(menu_id).await
    }

    async fn get_menu(&self, menu_id: &str) -> RbacResult<Menu> {
        self.get_menu(menu_id).await
    }

    async fn page_menus(&self, filter: MenuListFilter) -> RbacResult<CursorPage<Menu>> {
        self.page_menus(filter).await
    }

    async fn list_menus(&self) -> RbacResult<Vec<Menu>> {
        self.list_menus().await
    }

    async fn replace_role_menus(&self, role_id: &str, input: RoleMenuBindingInput) -> RbacResult<()> {
        self.replace_role_menus(role_id, input).await
    }

    async fn replace_role_depts(&self, role_id: &str, input: RoleDeptBindingInput) -> RbacResult<()> {
        self.replace_role_depts(role_id, input).await
    }

    async fn role_menu_ids(&self, role_id: &str) -> RbacResult<Vec<String>> {
        self.role_menu_ids(role_id).await
    }

    async fn role_dept_ids(&self, role_id: &str) -> RbacResult<Vec<String>> {
        self.role_dept_ids(role_id).await
    }
}
