use kernel::pagination::PageRequest;

use crate::{
    application::{MenuListFilter, RbacError, RbacRepository, RbacResult, RoleListFilter, RoleUserListFilter},
    domain::{Menu, MenuInput, RoleDataScopeInput, RoleInput, RoleUserBindingInput},
};

use super::localization::{localized, localized_param};

pub(super) fn sanitize_role(input: RoleInput) -> RbacResult<RoleInput> {
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

pub(super) fn sanitize_menu(input: MenuInput) -> RbacResult<MenuInput> {
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

pub(super) fn sanitize_role_data_scope(input: RoleDataScopeInput) -> RbacResult<RoleDataScopeInput> {
    Ok(RoleDataScopeInput {
        data_scope: required("data_scope", input.data_scope)?,
        dept_check_strictly: input.dept_check_strictly,
        dept_ids: clean_ids(input.dept_ids),
    })
}

pub(super) fn sanitize_role_filter(input: RoleListFilter) -> RoleListFilter {
    RoleListFilter {
        page: input.page,
        role_name: trim_optional(input.role_name),
        role_key: trim_optional(input.role_key),
        status: trim_optional(input.status),
        system: input.system,
        begin_time: input.begin_time,
        end_time: input.end_time,
    }
}

pub(super) fn sanitize_menu_filter(input: MenuListFilter) -> MenuListFilter {
    MenuListFilter {
        page: input.page,
        menu_name: trim_optional(input.menu_name),
        status: trim_optional(input.status),
        begin_time: input.begin_time,
        end_time: input.end_time,
    }
}

pub(super) fn sanitize_role_user_filter(input: RoleUserListFilter) -> RoleUserListFilter {
    RoleUserListFilter {
        page: input.page,
        role_id: input.role_id.trim().into(),
        username: trim_optional(input.username),
        phonenumber: trim_optional(input.phonenumber),
        allocated: input.allocated,
    }
}

pub(super) fn sanitize_role_users(input: RoleUserBindingInput) -> RoleUserBindingInput {
    RoleUserBindingInput {
        user_ids: clean_ids(input.user_ids),
    }
}

pub(super) fn clean_ids(ids: Vec<String>) -> Vec<String> {
    ids.into_iter().map(|id| id.trim().into()).filter(|id: &String| !id.is_empty()).collect()
}

pub(super) fn reject_empty_ids(ids: &[String]) -> RbacResult<()> {
    if ids.is_empty() {
        return Err(RbacError::InvalidInput(localized("errors.rbac.ids_empty")));
    }
    Ok(())
}

pub(super) fn reject_unscoped_user_ids(requested: &[String], scoped: &[String]) -> RbacResult<()> {
    if requested.iter().all(|id| scoped.contains(id)) {
        return Ok(());
    }
    Err(RbacError::Forbidden)
}

pub(super) fn validate_page(page: PageRequest) -> RbacResult<()> {
    if page.page == 0 || page.page_size == 0 {
        return Err(RbacError::InvalidInput(localized("errors.validation.page_and_size_positive")));
    }
    Ok(())
}

pub(super) fn required(field: &str, value: String) -> RbacResult<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty())
        .then(|| trimmed.into())
        .ok_or_else(|| RbacError::InvalidInput(localized_param("errors.validation.field_blank", "field", field)))
}

pub(super) fn trim_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().into()).filter(|item: &String| !item.is_empty())
}

pub(super) async fn reject_system_role_update<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    let role = repository.find_role(id).await?.ok_or(RbacError::NotFound)?;
    if role.system {
        return Err(RbacError::Conflict(localized("errors.rbac.system_role_immutable")));
    }
    Ok(())
}

pub(super) async fn reject_role_in_use<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    if repository.role_has_users(id).await? {
        return Err(RbacError::Conflict(localized("errors.rbac.role_assigned_to_users")));
    }
    Ok(())
}

pub(super) async fn reject_duplicate_role<R: RbacRepository>(repository: &R, input: &RoleInput, current_id: Option<&str>) -> RbacResult<()> {
    if repository.role_name_exists(&input.role_name, current_id).await? {
        return Err(RbacError::Conflict(localized("errors.rbac.role_name_exists")));
    }
    if repository.role_key_exists(&input.role_key, current_id).await? {
        return Err(RbacError::Conflict(localized("errors.rbac.role_key_exists")));
    }
    Ok(())
}

pub(super) async fn reject_menu_delete<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    ensure_menu_exists(repository, id).await?;
    if repository.menu_has_children(id).await? || repository.menu_has_role_bindings(id).await? {
        return Err(RbacError::Conflict(localized("errors.rbac.menu_has_children_or_bindings")));
    }
    Ok(())
}

pub(super) async fn reject_invalid_menu<R: RbacRepository>(repository: &R, input: &MenuInput, current_id: Option<&str>) -> RbacResult<()> {
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

pub(super) async fn ensure_role_exists<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    repository.find_role(id).await?.map(|_| ()).ok_or(RbacError::NotFound)
}

pub(super) async fn ensure_menu_exists<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    repository.find_menu(id).await?.map(|_| ()).ok_or(RbacError::NotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::format_description::well_known::Rfc3339;

    #[test]
    fn sanitize_role_filter_trims_text_and_preserves_system_filter() {
        let begin_time = time::OffsetDateTime::parse("2026-07-01T00:00:00Z", &Rfc3339).unwrap();
        let filter = RoleListFilter {
            page: PageRequest { page: 1, page_size: 10 },
            role_name: Some(" 管理员 ".into()),
            role_key: Some("   ".into()),
            status: Some(" 0 ".into()),
            system: Some(false),
            begin_time: Some(begin_time),
            end_time: None,
        };

        assert_eq!(
            sanitize_role_filter(filter),
            RoleListFilter {
                page: PageRequest { page: 1, page_size: 10 },
                role_name: Some("管理员".into()),
                role_key: None,
                status: Some("0".into()),
                system: Some(false),
                begin_time: Some(begin_time),
                end_time: None,
            }
        );
    }
}
