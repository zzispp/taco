use crate::domain::{
    ApiPermission, ApiPermissionInput, ApiPermissionSnapshot, MenuItem, MenuItemInput, MenuSection, MenuSectionInput, NavItemResponse, NavSectionResponse,
    PermissionSnapshot, Role, RoleInput, RoleMenuSnapshot,
};
use kernel::pagination::{Page, PageRequest};

pub(super) fn permission_snapshot() -> PermissionSnapshot {
    PermissionSnapshot {
        api_permissions: vec![ApiPermissionSnapshot {
            method: "PUT".into(),
            path_pattern: "/api/users/{id}".into(),
            role_codes: vec!["admin".into()],
        }],
        menus: vec![RoleMenuSnapshot {
            role_code: "admin".into(),
            sections: vec![NavSectionResponse {
                code: "management".into(),
                subheader: "Management".into(),
                items: vec![NavItemResponse {
                    code: "users".into(),
                    title: "Users".into(),
                    path: "/dashboard/user/list".into(),
                    icon: Some("icon.user".into()),
                    caption: None,
                    deep_match: false,
                    children: vec![],
                }],
            }],
        }],
    }
}

pub(super) fn role_input(code: &str) -> RoleInput {
    RoleInput {
        code: code.into(),
        name: code.into(),
        description: String::new(),
        enabled: true,
        sort_order: 0,
    }
}

pub(super) fn role_from_input(input: RoleInput) -> Role {
    Role {
        code: input.code,
        name: input.name,
        description: input.description,
        enabled: input.enabled,
        system: false,
        sort_order: input.sort_order,
    }
}

pub(super) fn api_input(code: &str) -> ApiPermissionInput {
    ApiPermissionInput {
        code: code.into(),
        method: "GET".into(),
        path_pattern: "/api/users".into(),
        name: code.into(),
        group: "Users".into(),
        enabled: true,
    }
}

pub(super) fn api_permission(id: u64, input: ApiPermissionInput) -> ApiPermission {
    ApiPermission {
        id: rbac_id(id),
        code: input.code,
        method: input.method,
        path_pattern: input.path_pattern,
        name: input.name,
        group: input.group,
        enabled: input.enabled,
        system: false,
    }
}

pub(super) fn menu_item_input(code: &str) -> MenuItemInput {
    MenuItemInput {
        section_id: rbac_id(1),
        parent_id: None,
        code: code.into(),
        title: code.into(),
        path: "/dashboard/users".into(),
        icon: None,
        caption: None,
        deep_match: false,
        sort_order: 0,
        enabled: true,
    }
}

pub(super) fn menu_section(id: u64, input: MenuSectionInput) -> MenuSection {
    MenuSection {
        id: rbac_id(id),
        code: input.code,
        subheader: input.subheader,
        sort_order: input.sort_order,
        enabled: input.enabled,
    }
}

pub(super) fn menu_item(id: u64, input: MenuItemInput) -> MenuItem {
    MenuItem {
        id: rbac_id(id),
        section_id: input.section_id,
        parent_id: input.parent_id,
        code: input.code,
        title: input.title,
        path: input.path,
        icon: input.icon,
        caption: input.caption,
        deep_match: input.deep_match,
        sort_order: input.sort_order,
        enabled: input.enabled,
    }
}

pub(super) fn rbac_id(id: u64) -> String {
    format!("018f0000-0000-7000-9000-{id:012}")
}

pub(super) fn page_items<T: Clone>(items: Vec<T>, page: PageRequest) -> Page<T> {
    Page {
        total: items.len() as u64,
        items: items.into_iter().take(page.page_size as usize).collect(),
        page: page.page,
        page_size: page.page_size,
    }
}
