use constants::pagination::{MAX_PAGE_SIZE, MIN_PAGE_NUMBER, MIN_PAGE_SIZE};
use kernel::pagination::PageRequest;

use crate::application::{RbacError, RbacResult};
use crate::domain::{ApiPermissionInput, MenuItemInput, MenuSectionInput, RoleInput};

pub(super) fn sanitize_role(input: RoleInput) -> RbacResult<RoleInput> {
    let input = RoleInput {
        code: trim_required("code", input.code)?,
        name: trim_required("name", input.name)?,
        description: input.description.trim().into(),
        enabled: input.enabled,
        sort_order: input.sort_order,
    };
    Ok(input)
}

pub(super) fn sanitize_api(input: ApiPermissionInput) -> RbacResult<ApiPermissionInput> {
    Ok(ApiPermissionInput {
        code: trim_required("code", input.code)?,
        method: trim_required("method", input.method)?.to_ascii_uppercase(),
        path_pattern: trim_required("path_pattern", input.path_pattern)?,
        name: trim_required("name", input.name)?,
        group: input.group.trim().into(),
        enabled: input.enabled,
    })
}

pub(super) fn sanitize_menu_section(input: MenuSectionInput) -> RbacResult<MenuSectionInput> {
    Ok(MenuSectionInput {
        code: trim_required("code", input.code)?,
        subheader: trim_required("subheader", input.subheader)?,
        sort_order: input.sort_order,
        enabled: input.enabled,
    })
}

pub(super) fn sanitize_menu_item(input: MenuItemInput) -> RbacResult<MenuItemInput> {
    Ok(MenuItemInput {
        section_id: input.section_id,
        parent_id: input.parent_id,
        code: trim_required("code", input.code)?,
        title: trim_required("title", input.title)?,
        path: trim_required("path", input.path)?,
        icon: trim_optional(input.icon),
        caption: trim_optional(input.caption),
        deep_match: input.deep_match,
        sort_order: input.sort_order,
        enabled: input.enabled,
    })
}

pub(super) fn validate_page(page: PageRequest) -> RbacResult<()> {
    if page.page < MIN_PAGE_NUMBER {
        return Err(RbacError::InvalidInput("page must be greater than 0".into()));
    }
    if page.page_size < MIN_PAGE_SIZE {
        return Err(RbacError::InvalidInput("page_size must be greater than 0".into()));
    }
    if page.page_size > MAX_PAGE_SIZE {
        return Err(RbacError::InvalidInput(format!("page_size must be less than or equal to {MAX_PAGE_SIZE}")));
    }
    Ok(())
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().into()).filter(|item: &String| !item.is_empty())
}

fn trim_required(field: &'static str, value: String) -> RbacResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(RbacError::InvalidInput(format!("{field} cannot be blank")));
    }
    Ok(trimmed.into())
}
