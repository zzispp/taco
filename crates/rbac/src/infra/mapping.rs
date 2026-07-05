use std::collections::HashMap;

use storage::StorageError;

use crate::{
    application::RbacError,
    domain::{Menu, NavItemResponse, NavSectionResponse, PermissionSnapshot, Role, RoleMenuSnapshot, RoleOption, RolePermissionSnapshot, RoleUser},
};

use super::records::{MenuRecord, RoleDeptRecord, RoleMenuRecord, RoleOptionRecord, RolePermissionRecord, RoleRecord, RoleUserRecord};

pub fn role(record: RoleRecord) -> Role {
    Role {
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
        create_time: record.create_time,
    }
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

pub fn permission_snapshot(permission_rows: Vec<RolePermissionRecord>, dept_rows: Vec<RoleDeptRecord>, menu_rows: Vec<RoleMenuRecord>) -> PermissionSnapshot {
    PermissionSnapshot {
        roles: role_permissions(permission_rows, dept_rows),
        menus: role_menus(menu_rows),
    }
}

pub fn storage_error(error: StorageError) -> RbacError {
    match error {
        StorageError::NotFound => RbacError::NotFound,
        StorageError::Conflict(message) => RbacError::Conflict(message),
        StorageError::Database(message) => RbacError::Infrastructure(message),
    }
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
            sections: vec![NavSectionResponse {
                code: "system_management".into(),
                subheader: "System Management".into(),
                items: nav_items(rows),
            }],
        })
        .collect()
}

fn nav_items(rows: Vec<RoleMenuRecord>) -> Vec<NavItemResponse> {
    rows.into_iter()
        .filter(|row| row.parent_id == "1")
        .map(|row| NavItemResponse {
            code: row.menu_id,
            title: row.menu_name,
            path: dashboard_path(&row.path),
            icon: Some(row.icon),
            caption: None,
            deep_match: true,
            children: vec![],
        })
        .collect()
}

fn dashboard_path(path: &str) -> String {
    match path {
        "user" => "/dashboard/admin/users".into(),
        "role" => "/dashboard/admin/roles".into(),
        "menu" => "/dashboard/admin/menus".into(),
        "dept" => "/dashboard/admin/depts".into(),
        "post" => "/dashboard/admin/posts".into(),
        "dict" => "/dashboard/admin/dicts".into(),
        "config" => "/dashboard/admin/configs".into(),
        value => format!("/dashboard/admin/{value}"),
    }
}
