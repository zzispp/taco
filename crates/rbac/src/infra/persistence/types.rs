#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleRecordInput {
    pub code: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub system: bool,
    pub sort_order: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiPermissionRecordInput {
    pub code: String,
    pub method: String,
    pub path_pattern: String,
    pub name: String,
    pub group: String,
    pub enabled: bool,
    pub system: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuSectionRecordInput {
    pub code: String,
    pub subheader: String,
    pub sort_order: i64,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuItemRecordInput {
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleApiBindingRecordInput {
    pub role_code: String,
    pub api_permission_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleMenuBindingRecordInput {
    pub role_code: String,
    pub menu_item_id: String,
}
