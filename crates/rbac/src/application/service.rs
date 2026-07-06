use std::collections::HashSet;

use kernel::error::LocalizedError;
use kernel::pagination::{Page, PageRequest};
use matchit::Router;

use crate::api::CurrentUser;
use crate::application::{
    ApiCheckRequest, AuthorizationConfig, MenuListFilter, RbacCache, RbacError, RbacRepository, RbacResult, RoleListFilter, RoleUserListFilter,
};
use crate::domain::{
    DataScopeFilter, Menu, MenuInput, NavResponse, PermissionSnapshot, Role, RoleDataScopeInput, RoleDeptBindingInput, RoleInput, RoleMenuBindingInput,
    RoleOption, RoleUser, RoleUserBindingInput,
};
use types::system::SortBatchInput;

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

    pub async fn page_roles(&self, filter: RoleListFilter) -> RbacResult<Page<Role>> {
        let filter = sanitize_role_filter(filter);
        validate_page(filter.page)?;
        self.repository.page_roles(filter).await
    }

    pub async fn page_roles_scoped(&self, filter: RoleListFilter, scope: DataScopeFilter) -> RbacResult<Page<Role>> {
        let filter = sanitize_role_filter(filter);
        validate_page(filter.page)?;
        self.repository.page_roles_scoped(filter, scope).await
    }

    pub async fn role_options(&self) -> RbacResult<Vec<RoleOption>> {
        self.repository.role_options().await
    }

    pub async fn page_role_users(&self, filter: RoleUserListFilter, scope: Option<DataScopeFilter>) -> RbacResult<Page<RoleUser>> {
        let filter = sanitize_role_user_filter(filter);
        validate_page(filter.page)?;
        ensure_role_exists(&self.repository, &filter.role_id).await?;
        self.repository.page_role_users(filter, scope).await
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

    pub async fn page_menus(&self, filter: MenuListFilter) -> RbacResult<Page<Menu>> {
        let filter = sanitize_menu_filter(filter);
        validate_page(filter.page)?;
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
        let permission = required_permission(config, &request)?;
        if request
            .permissions
            .iter()
            .any(|item| item == permission || item == constants::system::ALL_PERMISSION)
        {
            return Ok(());
        }
        Err(RbacError::Forbidden)
    }

    pub async fn data_scope_filter(&self, user: &CurrentUser) -> RbacResult<DataScopeFilter> {
        let snapshot = self.cache.read_snapshot().await?;
        Ok(data_scope_filter(user, &snapshot))
    }

    pub fn validate_protected_handlers(&self, config: &AuthorizationConfig) -> RbacResult<()> {
        validate_protected_handlers(config)
    }

    pub fn validate_data_scope_handlers(&self, handlers: &[&str]) -> RbacResult<()> {
        validate_data_scope_handlers(handlers)
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

fn data_scope_filter(user: &CurrentUser, snapshot: &PermissionSnapshot) -> DataScopeFilter {
    let roles = snapshot
        .roles
        .iter()
        .filter(|role| role.status == constants::system::STATUS_NORMAL && user.role_keys.contains(&role.role_key));
    let data_scope = roles.clone().map(|role| role.data_scope.as_str()).min().unwrap_or("5");
    let dept_ids = roles.flat_map(|role| role.dept_ids.clone()).collect::<HashSet<_>>();
    DataScopeFilter {
        data_scope: if user.admin { "1".into() } else { data_scope.into() },
        user_id: user.id.clone(),
        dept_id: user.dept_id.clone(),
        dept_ids: dept_ids.into_iter().collect(),
    }
}

fn validate_protected_handlers(config: &AuthorizationConfig) -> RbacResult<()> {
    let declared = inventory::iter::<crate::application::ProtectedHandler>
        .into_iter()
        .map(|handler| (handler.function, handler.permission))
        .collect::<HashSet<_>>();
    for rule in &config.route_permissions {
        if !declared.contains(&(rule.handler, rule.permission.as_str())) {
            return Err(RbacError::InvalidInput(localized_param(
                "errors.rbac.missing_handler_permission",
                "handler",
                rule.handler,
            )));
        }
    }
    Ok(())
}

fn validate_data_scope_handlers(handlers: &[&str]) -> RbacResult<()> {
    let declared = inventory::iter::<crate::application::DataScopeHandler>
        .into_iter()
        .map(|handler| handler.function)
        .collect::<HashSet<_>>();
    for handler in handlers {
        if !declared.contains(handler) {
            return Err(RbacError::InvalidInput(localized_param("errors.rbac.missing_data_scope", "handler", *handler)));
        }
    }
    Ok(())
}

fn required_permission<'a>(config: &'a AuthorizationConfig, request: &ApiCheckRequest) -> RbacResult<&'a str> {
    config
        .route_permissions
        .iter()
        .find(|rule| route_rule_matches(rule, request).unwrap_or(false))
        .map(|rule| rule.permission.as_str())
        .ok_or(RbacError::Forbidden)
}

fn route_rule_matches(rule: &types::rbac::RoutePermissionRule, request: &ApiCheckRequest) -> RbacResult<bool> {
    if !rule.methods.iter().any(|method| method.eq_ignore_ascii_case(&request.method)) {
        return Ok(false);
    }
    path_matches(&rule.path_pattern, &request.path)
}

fn is_whitelisted(config: &AuthorizationConfig, method: &str, path: &str) -> RbacResult<bool> {
    let method = method.to_ascii_uppercase();
    config.whitelist.iter().try_fold(false, |matched, rule| {
        if matched || !rule.methods.iter().any(|item| item.eq_ignore_ascii_case(&method)) {
            return Ok(matched);
        }
        path_matches(&rule.path_pattern, path)
    })
}

fn path_matches(pattern: &str, path: &str) -> RbacResult<bool> {
    let mut router = Router::new();
    router
        .insert(pattern, ())
        .map_err(|error| RbacError::InvalidInput(localized_param("errors.rbac.invalid_route_pattern", "error", error.to_string())))?;
    Ok(router.at(path).is_ok())
}

fn sanitize_role(input: RoleInput) -> RbacResult<RoleInput> {
    Ok(RoleInput {
        role_name: required("role_name", input.role_name)?,
        role_key: required("role_key", input.role_key)?,
        role_sort: input.role_sort,
        data_scope: required("data_scope", input.data_scope)?,
        menu_check_strictly: input.menu_check_strictly,
        dept_check_strictly: input.dept_check_strictly,
        status: required("status", input.status)?,
        remark: trim_optional(input.remark),
    })
}

fn sanitize_menu(input: MenuInput) -> RbacResult<MenuInput> {
    Ok(MenuInput {
        menu_name: required("menu_name", input.menu_name)?,
        parent_id: required("parent_id", input.parent_id)?,
        order_num: input.order_num,
        path: input.path.trim().into(),
        component: trim_optional(input.component),
        query: trim_optional(input.query),
        route_name: input.route_name.trim().into(),
        is_frame: input.is_frame,
        is_cache: input.is_cache,
        menu_type: required("menu_type", input.menu_type)?,
        visible: required("visible", input.visible)?,
        status: required("status", input.status)?,
        perms: trim_optional(input.perms),
        icon: required("icon", input.icon)?,
        remark: trim_optional(input.remark),
    })
}

fn sanitize_role_data_scope(input: RoleDataScopeInput) -> RbacResult<RoleDataScopeInput> {
    Ok(RoleDataScopeInput {
        data_scope: required("data_scope", input.data_scope)?,
        dept_check_strictly: input.dept_check_strictly,
        dept_ids: clean_ids(input.dept_ids),
    })
}

fn sanitize_role_filter(input: RoleListFilter) -> RoleListFilter {
    RoleListFilter {
        page: input.page,
        role_name: trim_optional(input.role_name),
        role_key: trim_optional(input.role_key),
        status: trim_optional(input.status),
        begin_time: trim_optional(input.begin_time),
        end_time: trim_optional(input.end_time),
    }
}

fn sanitize_menu_filter(input: MenuListFilter) -> MenuListFilter {
    MenuListFilter {
        page: input.page,
        menu_name: trim_optional(input.menu_name),
        status: trim_optional(input.status),
    }
}

fn sanitize_role_user_filter(input: RoleUserListFilter) -> RoleUserListFilter {
    RoleUserListFilter {
        page: input.page,
        role_id: input.role_id.trim().into(),
        username: trim_optional(input.username),
        phonenumber: trim_optional(input.phonenumber),
        allocated: input.allocated,
    }
}

fn sanitize_role_users(input: RoleUserBindingInput) -> RoleUserBindingInput {
    RoleUserBindingInput {
        user_ids: clean_ids(input.user_ids),
    }
}

fn clean_ids(ids: Vec<String>) -> Vec<String> {
    ids.into_iter().map(|id| id.trim().into()).filter(|id: &String| !id.is_empty()).collect()
}

fn reject_empty_ids(ids: &[String]) -> RbacResult<()> {
    if ids.is_empty() {
        return Err(RbacError::InvalidInput(localized("errors.rbac.ids_empty")));
    }
    Ok(())
}

fn validate_page(page: PageRequest) -> RbacResult<()> {
    if page.page == 0 || page.page_size == 0 {
        return Err(RbacError::InvalidInput(localized("errors.validation.page_and_size_positive")));
    }
    Ok(())
}

fn required(field: &str, value: String) -> RbacResult<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty())
        .then(|| trimmed.into())
        .ok_or_else(|| RbacError::InvalidInput(localized_param("errors.validation.field_blank", "field", field)))
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().into()).filter(|item: &String| !item.is_empty())
}

fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
}

fn localized_param(key: &'static str, param: &'static str, value: impl Into<String>) -> LocalizedError {
    LocalizedError::new(key).with_param(param, value)
}

async fn reject_system_role_update<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    let role = repository.find_role(id).await?.ok_or(RbacError::NotFound)?;
    if role.system {
        return Err(RbacError::Conflict(localized("errors.rbac.system_role_immutable")));
    }
    Ok(())
}

async fn reject_role_in_use<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    if repository.role_has_users(id).await? {
        return Err(RbacError::Conflict(localized("errors.rbac.role_assigned_to_users")));
    }
    Ok(())
}

async fn reject_duplicate_role<R: RbacRepository>(repository: &R, input: &RoleInput, current_id: Option<&str>) -> RbacResult<()> {
    if repository.role_name_exists(&input.role_name, current_id).await? {
        return Err(RbacError::Conflict(localized("errors.rbac.role_name_exists")));
    }
    if repository.role_key_exists(&input.role_key, current_id).await? {
        return Err(RbacError::Conflict(localized("errors.rbac.role_key_exists")));
    }
    Ok(())
}

async fn reject_menu_delete<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    ensure_menu_exists(repository, id).await?;
    if repository.menu_has_children(id).await? || repository.menu_has_role_bindings(id).await? {
        return Err(RbacError::Conflict(localized("errors.rbac.menu_has_children_or_bindings")));
    }
    Ok(())
}

async fn reject_invalid_menu<R: RbacRepository>(repository: &R, input: &MenuInput, current_id: Option<&str>) -> RbacResult<()> {
    reject_menu_parent(input, current_id)?;
    reject_external_link(input)?;
    reject_duplicate_menu(repository, input, current_id).await
}

fn reject_menu_parent(input: &MenuInput, current_id: Option<&str>) -> RbacResult<()> {
    if current_id.is_some_and(|id| input.parent_id == id) {
        return Err(RbacError::Conflict(localized("errors.rbac.menu_parent_self")));
    }
    Ok(())
}

fn reject_external_link(input: &MenuInput) -> RbacResult<()> {
    if input.is_frame && !input.path.starts_with("http://") && !input.path.starts_with("https://") {
        return Err(RbacError::InvalidInput(localized("errors.rbac.external_link_scheme")));
    }
    Ok(())
}

async fn reject_duplicate_menu<R: RbacRepository>(repository: &R, input: &MenuInput, current_id: Option<&str>) -> RbacResult<()> {
    let menus = repository.list_menus().await?;
    if menus.iter().any(|menu| same_parent_name(menu, input, current_id)) {
        return Err(RbacError::Conflict(localized("errors.rbac.menu_name_exists")));
    }
    if menus.iter().any(|menu| same_parent_path(menu, input, current_id)) {
        return Err(RbacError::Conflict(localized("errors.rbac.menu_path_exists")));
    }
    if menus.iter().any(|menu| same_route_name(menu, input, current_id)) {
        return Err(RbacError::Conflict(localized("errors.rbac.route_name_exists")));
    }
    Ok(())
}

fn same_parent_name(menu: &Menu, input: &MenuInput, current_id: Option<&str>) -> bool {
    menu.parent_id == input.parent_id && menu.menu_name == input.menu_name && Some(menu.menu_id.as_str()) != current_id
}

fn same_parent_path(menu: &Menu, input: &MenuInput, current_id: Option<&str>) -> bool {
    !input.path.is_empty() && menu.parent_id == input.parent_id && menu.path == input.path && Some(menu.menu_id.as_str()) != current_id
}

fn same_route_name(menu: &Menu, input: &MenuInput, current_id: Option<&str>) -> bool {
    !input.route_name.is_empty() && menu.route_name == input.route_name && Some(menu.menu_id.as_str()) != current_id
}

async fn ensure_role_exists<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    repository.find_role(id).await?.map(|_| ()).ok_or(RbacError::NotFound)
}

async fn ensure_menu_exists<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    repository.find_menu(id).await?.map(|_| ()).ok_or(RbacError::NotFound)
}

#[path = "service/use_cases.rs"]
mod use_cases;

#[cfg(test)]
#[path = "service/tests.rs"]
mod tests;
