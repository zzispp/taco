use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Role {
    pub code: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub system: bool,
    pub sort_order: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct ApiPermission {
    pub id: String,
    pub code: String,
    pub method: String,
    pub path_pattern: String,
    pub name: String,
    pub group: String,
    pub enabled: bool,
    pub system: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct MenuSection {
    pub id: String,
    pub code: String,
    pub subheader: String,
    pub sort_order: i64,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct MenuItem {
    pub id: String,
    pub section_id: String,
    pub parent_id: Option<String>,
    pub code: String,
    pub title: String,
    pub path: String,
    pub icon: Option<String>,
    pub caption: Option<String>,
    pub deep_match: bool,
    pub sort_order: i64,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleInput {
    pub code: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub sort_order: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct ApiPermissionInput {
    pub code: String,
    pub method: String,
    pub path_pattern: String,
    pub name: String,
    pub group: String,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct MenuSectionInput {
    pub code: String,
    pub subheader: String,
    pub sort_order: i64,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct MenuItemInput {
    pub section_id: String,
    pub parent_id: Option<String>,
    pub code: String,
    pub title: String,
    pub path: String,
    pub icon: Option<String>,
    pub caption: Option<String>,
    pub deep_match: bool,
    pub sort_order: i64,
    pub enabled: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleApiBindingInput {
    pub api_permission_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleMenuBindingInput {
    pub menu_item_ids: Vec<String>,
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
    pub api_permissions: Vec<ApiPermissionSnapshot>,
    pub menus: Vec<RoleMenuSnapshot>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct ApiPermissionSnapshot {
    pub method: String,
    pub path_pattern: String,
    pub role_codes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct RoleMenuSnapshot {
    pub role_code: String,
    pub sections: Vec<NavSectionResponse>,
}
