use serde::{Deserialize, Serialize};

use crate::{
    rbac::{RoleOption, RoleSummary},
    system::{Post, TreeSelectNode},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub nick_name: String,
    pub dept_id: Option<String>,
    pub email: String,
    pub phonenumber: Option<String>,
    pub sex: String,
    pub avatar: Option<String>,
    pub status: String,
    pub is_installation_owner: bool,
    pub auth_source: String,
    pub email_verified: bool,
    pub remark: Option<String>,
    pub roles: Vec<RoleSummary>,
    pub role_ids: Vec<String>,
    pub post_ids: Vec<String>,
    pub permissions: Vec<String>,
    pub create_time: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UserFormOptions {
    pub roles: Vec<RoleOption>,
    pub posts: Vec<Post>,
    pub depts: Vec<TreeSelectNode>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UserProfile {
    pub user: User,
    pub role_group: String,
    pub post_group: String,
    pub dept_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserProfileGroups {
    pub role_group: String,
    pub post_group: String,
    pub dept_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub nick_name: String,
    pub dept_id: Option<String>,
    pub email: String,
    pub phonenumber: Option<String>,
    pub sex: String,
    pub status: String,
    pub remark: Option<String>,
    pub role_ids: Vec<String>,
    pub post_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceUser {
    pub username: String,
    pub password: Option<String>,
    pub nick_name: String,
    pub dept_id: Option<String>,
    pub email: String,
    pub phonenumber: Option<String>,
    pub sex: String,
    pub status: String,
    pub remark: Option<String>,
    pub role_ids: Vec<String>,
    pub post_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileUpdate {
    pub nick_name: String,
    pub phonenumber: Option<String>,
    pub email: String,
    pub sex: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Credentials {
    pub identifier: String,
    pub password: String,
}
