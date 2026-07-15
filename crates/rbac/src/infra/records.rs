use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct RoleRecord {
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
    pub create_time: OffsetDateTime,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct MenuRecord {
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
    pub create_time: OffsetDateTime,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct RolePermissionRecord {
    pub role_key: String,
    pub status: String,
    pub data_scope: String,
    pub perms: Option<String>,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct RoleDeptRecord {
    pub role_key: String,
    pub dept_id: String,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct RoleMenuRecord {
    pub role_key: String,
    pub menu_id: String,
    pub menu_name: String,
    pub parent_id: String,
    pub path: String,
    pub menu_type: String,
    pub icon: String,
    pub order_num: i64,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct RoleOptionRecord {
    pub role_id: String,
    pub role_name: String,
    pub role_key: String,
    pub status: String,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct RoleUserRecord {
    pub user_id: String,
    pub username: String,
    pub nick_name: String,
    pub dept_id: Option<String>,
    pub phonenumber: Option<String>,
    pub email: String,
    pub status: String,
    pub create_time: OffsetDateTime,
}
