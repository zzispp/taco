use kernel::pagination::CursorPage;

use crate::api::CurrentUser;
use crate::application::{
    ApiCheckRequest, AuthorizationConfig, MenuListFilter, RbacCache, RbacError, RbacRepository, RbacResult, RoleExportRequest, RoleExportSink, RoleListFilter,
    RoleUserListFilter,
};
use crate::domain::{
    DataScopeFilter, Menu, MenuInput, NavResponse, Role, RoleDataScopeInput, RoleDeptBindingInput, RoleInput, RoleMenuBindingInput, RoleOption, RoleUser,
    RoleUserBindingInput,
};
use types::system::SortBatchInput;

mod audited;
mod audited_use_case;
mod authorization;
mod cache_refresh_use_case;
mod localization;
mod validation;

use self::{
    authorization::{data_scope_filter, is_whitelisted, required_permission, validate_protected_handlers},
    localization::localized,
    validation::{
        clean_ids, ensure_menu_exists, ensure_role_exists, reject_duplicate_role, reject_empty_ids, reject_invalid_menu, reject_menu_delete,
        reject_role_in_use, reject_system_role_update, reject_unscoped_user_ids, required, sanitize_menu, sanitize_menu_filter, sanitize_role,
        sanitize_role_data_scope, sanitize_role_filter, sanitize_role_user_filter, sanitize_role_users, validate_page,
    },
};

pub struct RbacService<R, C> {
    repository: R,
    cache: C,
}

impl<R, C> RbacService<R, C>
where
    R: RbacRepository,
    C: RbacCache,
{
    pub const fn new(repository: R, cache: C) -> Self {
        Self { repository, cache }
    }

    pub async fn create_role(&self, input: RoleInput) -> RbacResult<Role> {
        let input = sanitize_role(input)?;
        reject_duplicate_role(&self.repository, &input, None).await?;
        let role = self.repository.create_role(input).await?;
        self.rebuild_cache().await?;
        Ok(role)
    }

    pub async fn replace_role(&self, id: &str, input: RoleInput) -> RbacResult<Role> {
        reject_system_role_update(&self.repository, id).await?;
        let input = sanitize_role(input)?;
        reject_duplicate_role(&self.repository, &input, Some(id)).await?;
        let role = self.repository.replace_role(id, input).await?;
        self.rebuild_cache().await?;
        Ok(role)
    }

    pub async fn update_role_status(&self, id: &str, status: String) -> RbacResult<Role> {
        reject_system_role_update(&self.repository, id).await?;
        let role = self.repository.update_role_status(id, required("status", status)?).await?;
        self.rebuild_cache().await?;
        Ok(role)
    }

    pub async fn update_role_data_scope(&self, id: &str, input: RoleDataScopeInput) -> RbacResult<Role> {
        reject_system_role_update(&self.repository, id).await?;
        let input = sanitize_role_data_scope(input)?;
        let role = self.repository.update_role_data_scope(id, input).await?;
        self.rebuild_cache().await?;
        Ok(role)
    }

    pub async fn delete_role(&self, id: &str) -> RbacResult<()> {
        reject_system_role_update(&self.repository, id).await?;
        reject_role_in_use(&self.repository, id).await?;
        self.repository.delete_role(id).await?;
        self.rebuild_cache().await
    }

    pub async fn delete_roles(&self, ids: Vec<String>) -> RbacResult<()> {
        if ids.is_empty() {
            return Err(RbacError::InvalidInput(localized("errors.rbac.ids_required")));
        }
        for id in &ids {
            reject_system_role_update(&self.repository, id).await?;
            reject_role_in_use(&self.repository, id).await?;
        }
        self.repository.delete_roles(&ids).await?;
        self.rebuild_cache().await
    }

    pub async fn get_role(&self, id: &str) -> RbacResult<Role> {
        self.repository.find_role(id).await?.ok_or(RbacError::NotFound)
    }

    pub async fn page_roles(&self, filter: RoleListFilter) -> RbacResult<CursorPage<Role>> {
        let filter = sanitize_role_filter(filter);
        validate_page(&filter.page)?;
        crate::application::cursor::RoleCursorCodec::new(&filter, None)?.decode(&filter.page)?;
        self.repository.page_roles(filter).await
    }

    pub async fn page_roles_scoped(&self, filter: RoleListFilter, scope: DataScopeFilter) -> RbacResult<CursorPage<Role>> {
        let filter = sanitize_role_filter(filter);
        validate_page(&filter.page)?;
        crate::application::cursor::RoleCursorCodec::new(&filter, Some(&scope))?.decode(&filter.page)?;
        self.repository.page_roles_scoped(filter, scope).await
    }

    pub async fn export_roles(&self, request: RoleExportRequest, sink: &mut dyn RoleExportSink) -> RbacResult<()> {
        if request.batch_size == 0 {
            return Err(RbacError::InvalidInput(localized("errors.common.invalid_input")));
        }
        self.repository
            .export_roles(
                RoleExportRequest {
                    filter: sanitize_role_filter(request.filter),
                    ..request
                },
                sink,
            )
            .await
    }

    pub async fn role_options(&self) -> RbacResult<Vec<RoleOption>> {
        self.repository.role_options().await
    }

    pub async fn page_role_users(&self, filter: RoleUserListFilter, scope: Option<DataScopeFilter>) -> RbacResult<CursorPage<RoleUser>> {
        let filter = sanitize_role_user_filter(filter);
        validate_page(&filter.page)?;
        crate::application::cursor::RoleUserCursorCodec::new(&filter, scope.as_ref())?.decode(&filter.page)?;
        ensure_role_exists(&self.repository, &filter.role_id).await?;
        self.repository.page_role_users(filter, scope).await
    }

    pub async fn ensure_user_ids_scoped(&self, user_ids: Vec<String>, scope: DataScopeFilter) -> RbacResult<()> {
        let user_ids = clean_ids(user_ids);
        reject_empty_ids(&user_ids)?;
        let scoped = self.repository.scoped_user_ids(&user_ids, scope).await?;
        reject_unscoped_user_ids(&user_ids, &scoped)
    }

    pub async fn replace_role_users(&self, role_id: &str, input: RoleUserBindingInput) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_id).await?;
        self.repository.replace_role_users(role_id, sanitize_role_users(input)).await
    }

    pub async fn delete_role_user(&self, role_id: &str, user_id: &str) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_id).await?;
        self.repository.delete_role_user(role_id, user_id).await
    }

    pub async fn delete_role_users(&self, role_id: &str, user_ids: Vec<String>) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_id).await?;
        let user_ids = clean_ids(user_ids);
        reject_empty_ids(&user_ids)?;
        self.repository.delete_role_users(role_id, &user_ids).await
    }

    pub async fn create_menu(&self, input: MenuInput) -> RbacResult<Menu> {
        let input = sanitize_menu(input)?;
        reject_invalid_menu(&self.repository, &input, None).await?;
        let menu = self.repository.create_menu(input).await?;
        self.rebuild_cache().await?;
        Ok(menu)
    }

    pub async fn replace_menu(&self, id: &str, input: MenuInput) -> RbacResult<Menu> {
        ensure_menu_exists(&self.repository, id).await?;
        let input = sanitize_menu(input)?;
        reject_invalid_menu(&self.repository, &input, Some(id)).await?;
        let menu = self.repository.replace_menu(id, input).await?;
        self.rebuild_cache().await?;
        Ok(menu)
    }

    pub async fn update_menu_sort(&self, id: &str, order_num: i64) -> RbacResult<Menu> {
        ensure_menu_exists(&self.repository, id).await?;
        let menu = self.repository.update_menu_sort(id, order_num).await?;
        self.rebuild_cache().await?;
        Ok(menu)
    }

    pub async fn update_menu_sorts(&self, input: SortBatchInput) -> RbacResult<Vec<Menu>> {
        let menus = self.repository.update_menu_sorts(input).await?;
        self.rebuild_cache().await?;
        Ok(menus)
    }

    pub async fn delete_menu(&self, id: &str) -> RbacResult<()> {
        reject_menu_delete(&self.repository, id).await?;
        self.repository.delete_menu(id).await?;
        self.rebuild_cache().await
    }

    pub async fn get_menu(&self, id: &str) -> RbacResult<Menu> {
        self.repository.find_menu(id).await?.ok_or(RbacError::NotFound)
    }

    pub async fn page_menus(&self, filter: MenuListFilter) -> RbacResult<CursorPage<Menu>> {
        let filter = sanitize_menu_filter(filter);
        validate_page(&filter.page)?;
        crate::application::cursor::MenuCursorCodec::new(&filter)?.decode(&filter.page)?;
        self.repository.page_menus(filter).await
    }

    pub async fn list_menus(&self) -> RbacResult<Vec<Menu>> {
        self.repository.list_menus().await
    }

    pub async fn replace_role_menus(&self, role_id: &str, input: RoleMenuBindingInput) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_id).await?;
        self.repository.replace_role_menus(role_id, input).await?;
        self.rebuild_cache().await
    }

    pub async fn replace_role_depts(&self, role_id: &str, input: RoleDeptBindingInput) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_id).await?;
        self.repository.replace_role_depts(role_id, input).await?;
        self.rebuild_cache().await
    }

    pub async fn navbar(&self, current_user: &CurrentUser) -> RbacResult<NavResponse> {
        self.cache.read_nav(&current_user.role_keys, current_user.admin).await
    }

    pub async fn authorize_api(&self, config: &AuthorizationConfig, request: ApiCheckRequest) -> RbacResult<()> {
        if is_whitelisted(config, &request.method, &request.path)? || request.admin {
            return Ok(());
        }
        let requirement = required_permission(config, &request)?;
        let has_wildcard = request.permissions.iter().any(|item| item == constants::system::ALL_PERMISSION);
        if has_wildcard || requirement.is_satisfied_by(&request.permissions) {
            return Ok(());
        }
        Err(RbacError::Forbidden)
    }

    pub async fn data_scope_filter(&self, user: &CurrentUser) -> RbacResult<DataScopeFilter> {
        let snapshot = self.cache.read_snapshot().await?;
        data_scope_filter(user, &snapshot)
    }

    pub fn validate_protected_handlers(&self, config: &AuthorizationConfig) -> RbacResult<()> {
        validate_protected_handlers(config)
    }

    pub fn is_whitelisted(&self, config: &AuthorizationConfig, method: &str, path: &str) -> RbacResult<bool> {
        is_whitelisted(config, method, path)
    }

    pub async fn role_menu_ids(&self, role_id: &str) -> RbacResult<Vec<String>> {
        ensure_role_exists(&self.repository, role_id).await?;
        self.repository.role_menu_ids(role_id).await
    }

    pub async fn role_dept_ids(&self, role_id: &str) -> RbacResult<Vec<String>> {
        ensure_role_exists(&self.repository, role_id).await?;
        self.repository.role_dept_ids(role_id).await
    }

    pub async fn rebuild_cache(&self) -> RbacResult<()> {
        let snapshot = self.repository.permission_snapshot().await?;
        self.cache.write_snapshot(&snapshot).await
    }
}

mod use_cases;

#[cfg(test)]
mod tests;
