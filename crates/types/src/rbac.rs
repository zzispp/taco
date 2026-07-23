use serde::{Deserialize, Serialize};

use crate::system::TreeSelectNode;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Role {
    pub role_id: String,
    pub role_name: String,
    pub role_key: String,
    pub role_sort: i64,
    pub data_scope: String,
    pub menu_check_strictly: bool,
    pub dept_check_strictly: bool,
    pub status: String,
    pub system: bool,
    pub remark: Option<String>,
    pub create_time: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleInput {
    pub role_name: String,
    pub role_key: String,
    pub role_sort: i64,
    pub data_scope: String,
    pub menu_check_strictly: bool,
    pub dept_check_strictly: bool,
    pub status: String,
    pub remark: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleSummary {
    pub role_id: String,
    pub role_name: String,
    pub role_key: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleOption {
    pub role_id: String,
    pub role_name: String,
    pub role_key: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleUser {
    pub user_id: String,
    pub username: String,
    pub nick_name: String,
    pub dept_id: Option<String>,
    pub phonenumber: Option<String>,
    pub email: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Menu {
    pub menu_id: String,
    pub menu_name: String,
    pub parent_id: String,
    pub order_num: i64,
    pub path: String,
    pub component: Option<String>,
    pub query: Option<String>,
    pub route_name: String,
    pub is_frame: bool,
    pub is_cache: bool,
    pub menu_type: String,
    pub visible: String,
    pub status: String,
    pub perms: Option<String>,
    pub icon: String,
    pub remark: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct MenuInput {
    pub menu_name: String,
    pub parent_id: String,
    pub order_num: i64,
    pub path: String,
    pub component: Option<String>,
    pub query: Option<String>,
    pub route_name: String,
    pub is_frame: bool,
    pub is_cache: bool,
    pub menu_type: String,
    pub visible: String,
    pub status: String,
    pub perms: Option<String>,
    pub icon: String,
    pub remark: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleMenuBindingInput {
    pub menu_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleDeptBindingInput {
    pub dept_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleDataScopeInput {
    pub data_scope: String,
    pub dept_check_strictly: bool,
    pub dept_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleUserBindingInput {
    pub user_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleMenuTreeSelect {
    pub menus: Vec<TreeSelectNode>,
    pub checked_keys: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleDeptTreeSelect {
    pub depts: Vec<TreeSelectNode>,
    pub checked_keys: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct NavResponse {
    pub nav_items: Vec<NavSectionResponse>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct NavSectionResponse {
    pub code: String,
    pub subheader: String,
    pub items: Vec<NavItemResponse>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct NavItemResponse {
    pub code: String,
    pub title: String,
    pub path: String,
    pub icon: Option<String>,
    pub caption: Option<String>,
    pub deep_match: bool,
    pub children: Vec<NavItemResponse>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct PermissionSnapshot {
    pub roles: Vec<RolePermissionSnapshot>,
    pub menus: Vec<RoleMenuSnapshot>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RolePermissionSnapshot {
    pub role_key: String,
    pub status: String,
    pub permissions: Vec<String>,
    pub data_scope: String,
    pub dept_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleMenuSnapshot {
    pub role_key: String,
    pub sections: Vec<NavSectionResponse>,
}
