use std::collections::HashMap;

use kernel::error::LocalizedError;
use storage::StorageError;

use crate::{
    application::RbacError,
    domain::{
        MENU_TYPE_DIRECTORY, MENU_TYPE_MENU, Menu, NavItemResponse, NavSectionResponse, PermissionSnapshot, Role, RoleMenuSnapshot, RoleOption,
        RolePermissionSnapshot, RoleUser,
    },
};

use super::records::{MenuRecord, RoleDeptRecord, RoleMenuRecord, RoleOptionRecord, RolePermissionRecord, RoleRecord, RoleUserRecord};

const MENU_ROOT_PARENT_ID: &str = "0";
const EXTERNAL_HTTP_SCHEME: &str = "http://";
const EXTERNAL_HTTPS_SCHEME: &str = "https://";
const ROLE_NAME_UNIQUE_CONSTRAINT: &str = "idx_sys_role_name";
const ROLE_KEY_UNIQUE_CONSTRAINT: &str = "idx_sys_role_key";
const MENU_NAME_UNIQUE_CONSTRAINT: &str = "idx_sys_menu_parent_name";
const MENU_PATH_UNIQUE_CONSTRAINT: &str = "idx_sys_menu_parent_path";
const MENU_ROUTE_NAME_UNIQUE_CONSTRAINT: &str = "idx_sys_menu_route_name";

pub fn role(record: RoleRecord) -> Result<Role, StorageError> {
    let create_time = types::http::format_utc_rfc3339_millis(record.create_time).map_err(|error| StorageError::Database(error.to_string()))?;
    Ok(Role {
        role_id: record.role_id,
        role_name: record.role_name,
        role_key: record.role_key,
        role_sort: record.role_sort,
        data_scope: record.data_scope,
        menu_check_strictly: record.menu_check_strictly,
        dept_check_strictly: record.dept_check_strictly,
        status: record.status,
        system: record.system,
        remark: record.remark,
        create_time,
    })
}

pub fn role_option(record: RoleOptionRecord) -> RoleOption {
    RoleOption {
        role_id: record.role_id,
        role_name: record.role_name,
        role_key: record.role_key,
        status: record.status,
    }
}

pub fn role_user(record: RoleUserRecord) -> RoleUser {
    RoleUser {
        user_id: record.user_id,
        username: record.username,
        nick_name: record.nick_name,
        dept_id: record.dept_id,
        phonenumber: record.phonenumber,
        email: record.email,
        status: record.status,
    }
}

pub fn menu(record: MenuRecord) -> Menu {
    Menu {
        menu_id: record.menu_id,
        menu_name: record.menu_name,
        parent_id: record.parent_id,
        order_num: record.order_num,
        path: record.path,
        component: record.component,
        query: record.query,
        route_name: record.route_name,
        is_frame: record.is_frame,
        is_cache: record.is_cache,
        menu_type: record.menu_type,
        visible: record.visible,
        status: record.status,
        perms: record.perms,
        icon: record.icon,
        remark: record.remark,
    }
}

pub fn permission_snapshot(
    permission_rows: Vec<RolePermissionRecord>,
    dept_rows: Vec<RoleDeptRecord>,
    menu_rows: Vec<RoleMenuRecord>,
    installation_owner_menu_rows: Vec<RoleMenuRecord>,
) -> PermissionSnapshot {
    PermissionSnapshot {
        roles: role_permissions(permission_rows, dept_rows),
        menus: role_menus(menu_rows),
        installation_owner_menus: nav_sections(installation_owner_menu_rows),
    }
}

pub fn storage_error(error: StorageError) -> RbacError {
    match error {
        StorageError::NotFound => RbacError::NotFound,
        StorageError::Conflict(_) => RbacError::Conflict(LocalizedError::new("errors.common.conflict")),
        StorageError::UniqueViolation { constraint, message } => rbac_unique_violation(constraint.as_deref(), message),
        StorageError::Database(message) => RbacError::Infrastructure(message),
    }
}

fn rbac_unique_violation(constraint: Option<&str>, message: String) -> RbacError {
    let key = match constraint {
        Some(ROLE_NAME_UNIQUE_CONSTRAINT) => "errors.rbac.role_name_exists",
        Some(ROLE_KEY_UNIQUE_CONSTRAINT) => "errors.rbac.role_key_exists",
        Some(MENU_NAME_UNIQUE_CONSTRAINT) => "errors.rbac.menu_name_exists",
        Some(MENU_PATH_UNIQUE_CONSTRAINT) => "errors.rbac.menu_path_exists",
        Some(MENU_ROUTE_NAME_UNIQUE_CONSTRAINT) => "errors.rbac.route_name_exists",
        _ => return RbacError::Infrastructure(message),
    };
    RbacError::Conflict(LocalizedError::new(key))
}

fn role_permissions(rows: Vec<RolePermissionRecord>, depts: Vec<RoleDeptRecord>) -> Vec<RolePermissionSnapshot> {
    let mut roles = HashMap::<String, RolePermissionSnapshot>::new();
    for row in rows {
        let role = roles.entry(row.role_key.clone()).or_insert_with(|| RolePermissionSnapshot {
            role_key: row.role_key,
            status: row.status,
            permissions: vec![],
            data_scope: row.data_scope,
            dept_ids: vec![],
        });
        if let Some(perms) = row.perms.filter(|value| !value.is_empty()) {
            role.permissions.push(perms);
        }
    }
    for row in depts {
        roles.entry(row.role_key).and_modify(|role| role.dept_ids.push(row.dept_id));
    }
    roles.into_values().collect()
}

fn role_menus(rows: Vec<RoleMenuRecord>) -> Vec<RoleMenuSnapshot> {
    let mut grouped = HashMap::<String, Vec<RoleMenuRecord>>::new();
    for row in rows {
        grouped.entry(row.role_key.clone()).or_default().push(row);
    }
    grouped
        .into_iter()
        .map(|(role_key, rows)| RoleMenuSnapshot {
            role_key,
            sections: nav_sections(rows),
        })
        .collect()
}

fn nav_sections(rows: Vec<RoleMenuRecord>) -> Vec<NavSectionResponse> {
    rows.iter()
        .filter(|row| row.parent_id == MENU_ROOT_PARENT_ID)
        .filter_map(|row| root_section(row, &rows))
        .collect()
}

fn root_section(root: &RoleMenuRecord, rows: &[RoleMenuRecord]) -> Option<NavSectionResponse> {
    match root.menu_type.as_str() {
        MENU_TYPE_DIRECTORY => directory_section(root, rows),
        MENU_TYPE_MENU => Some(single_menu_section(root, rows)),
        _ => None,
    }
}

fn single_menu_section(menu: &RoleMenuRecord, rows: &[RoleMenuRecord]) -> NavSectionResponse {
    NavSectionResponse {
        code: menu.menu_id.clone(),
        subheader: menu.menu_name.clone(),
        items: vec![nav_item(menu, child_nav_items(rows, &menu.menu_id))],
    }
}

fn directory_section(directory: &RoleMenuRecord, rows: &[RoleMenuRecord]) -> Option<NavSectionResponse> {
    let items = child_nav_items(rows, &directory.menu_id);
    if items.is_empty() {
        return None;
    }
    Some(NavSectionResponse {
        code: directory.menu_id.clone(),
        subheader: directory.menu_name.clone(),
        items,
    })
}

fn child_nav_items(rows: &[RoleMenuRecord], parent_id: &str) -> Vec<NavItemResponse> {
    rows.iter()
        .filter(|row| row.parent_id == parent_id)
        .filter_map(|row| nav_tree_item(row, rows))
        .collect()
}

fn nav_tree_item(row: &RoleMenuRecord, rows: &[RoleMenuRecord]) -> Option<NavItemResponse> {
    let children = child_nav_items(rows, &row.menu_id);
    match row.menu_type.as_str() {
        MENU_TYPE_DIRECTORY if children.is_empty() => None,
        MENU_TYPE_DIRECTORY | MENU_TYPE_MENU => Some(nav_item(row, children)),
        _ => None,
    }
}

fn nav_item(row: &RoleMenuRecord, children: Vec<NavItemResponse>) -> NavItemResponse {
    let deep_match = !children.is_empty();
    NavItemResponse {
        code: row.menu_id.clone(),
        title: row.menu_name.clone(),
        path: nav_path(&row.path),
        icon: Some(row.icon.clone()),
        caption: None,
        deep_match,
        children,
    }
}

fn nav_path(path: &str) -> String {
    let path = path.trim();
    if path.starts_with('/') || is_external_path(path) {
        return path.into();
    }
    format!("/{path}")
}

fn is_external_path(path: &str) -> bool {
    path.starts_with(EXTERNAL_HTTP_SCHEME) || path.starts_with(EXTERNAL_HTTPS_SCHEME)
}

#[cfg(test)]
mod tests;
