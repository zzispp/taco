use sqlx::FromRow;
use time::OffsetDateTime;
use types::rbac::{ApiPermission, MenuItem, MenuSection, Role};

#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct RoleRecord {
    pub code: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub system: bool,
    pub sort_order: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct ApiPermissionRecord {
    pub id: String,
    pub code: String,
    pub method: String,
    pub path_pattern: String,
    pub name: String,
    pub group: String,
    pub enabled: bool,
    pub system: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct MenuSectionRecord {
    pub id: String,
    pub code: String,
    pub subheader: String,
    pub sort_order: i64,
    pub enabled: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct MenuItemRecord {
    pub id: String,
    pub section_id: String,
    pub parent_id: Option<String>,
    pub code: String,
    pub title: String,
    pub route_path: String,
    pub icon: Option<String>,
    pub caption: Option<String>,
    pub deep_match: bool,
    pub sort_order: i64,
    pub enabled: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct RoleApiPermissionRecord {
    pub role_code: String,
    pub api_permission_id: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq, FromRow)]
pub struct RoleMenuPermissionRecord {
    pub role_code: String,
    pub menu_item_id: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl From<RoleRecord> for Role {
    fn from(value: RoleRecord) -> Self {
        Self {
            code: value.code,
            name: value.name,
            description: value.description,
            enabled: value.enabled,
            system: value.system,
            sort_order: value.sort_order,
        }
    }
}

impl From<ApiPermissionRecord> for ApiPermission {
    fn from(value: ApiPermissionRecord) -> Self {
        Self {
            id: value.id,
            code: value.code,
            method: value.method,
            path_pattern: value.path_pattern,
            name: value.name,
            group: value.group,
            enabled: value.enabled,
            system: value.system,
        }
    }
}

impl From<MenuSectionRecord> for MenuSection {
    fn from(value: MenuSectionRecord) -> Self {
        Self {
            id: value.id,
            code: value.code,
            subheader: value.subheader,
            sort_order: value.sort_order,
            enabled: value.enabled,
        }
    }
}

impl From<MenuItemRecord> for MenuItem {
    fn from(value: MenuItemRecord) -> Self {
        Self {
            id: value.id,
            section_id: value.section_id,
            parent_id: value.parent_id,
            code: value.code,
            title: value.title,
            path: value.route_path,
            icon: value.icon,
            caption: value.caption,
            deep_match: value.deep_match,
            sort_order: value.sort_order,
            enabled: value.enabled,
        }
    }
}
