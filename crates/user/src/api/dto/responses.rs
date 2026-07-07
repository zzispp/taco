use kernel::pagination::Page;
use serde::Serialize;

use crate::domain::{User, UserFormOptions};

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub user_id: String,
    pub username: String,
    pub nick_name: String,
    pub dept_id: Option<String>,
    pub email: String,
    pub phonenumber: Option<String>,
    pub sex: String,
    pub avatar: Option<String>,
    pub status: String,
    pub is_active: bool,
    pub auth_source: String,
    pub email_verified: bool,
    pub system: bool,
    pub remark: Option<String>,
    pub roles: Vec<types::rbac::RoleSummary>,
    pub role_ids: Vec<String>,
    pub post_ids: Vec<String>,
    pub permissions: Vec<String>,
    pub create_time: String,
}

#[derive(Debug, Serialize)]
pub struct UsersPageResponse {
    pub items: Vec<UserResponse>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}

#[derive(Debug, Serialize)]
pub struct AuthSessionResponse {
    pub user: UserResponse,
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct TokenPairResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct MeResponse {
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserFormOptionsResponse {
    pub roles: Vec<types::rbac::RoleOption>,
    pub posts: Vec<types::system::Post>,
    pub depts: Vec<types::system::TreeSelectNode>,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub user: UserResponse,
    pub role_group: String,
    pub post_group: String,
    pub dept_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AvatarResponse {
    pub img_url: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize)]
pub struct UserImportResponse {
    pub success_count: usize,
    pub message: String,
}

impl From<User> for UserResponse {
    fn from(value: User) -> Self {
        Self {
            user_id: value.id.0,
            username: value.username,
            nick_name: value.nick_name,
            dept_id: value.dept_id,
            email: value.email,
            phonenumber: value.phonenumber,
            sex: value.sex,
            avatar: value.avatar,
            is_active: value.status == "0",
            status: value.status,
            auth_source: value.auth_source,
            email_verified: value.email_verified,
            system: value.system,
            remark: value.remark,
            roles: value.roles,
            role_ids: value.role_ids,
            post_ids: value.post_ids,
            permissions: value.permissions,
            create_time: value.create_time,
        }
    }
}

impl From<Page<User>> for UsersPageResponse {
    fn from(value: Page<User>) -> Self {
        Self {
            items: value.items.into_iter().map(UserResponse::from).collect(),
            total: value.total,
            page: value.page,
            page_size: value.page_size,
        }
    }
}

impl From<UserFormOptions> for UserFormOptionsResponse {
    fn from(value: UserFormOptions) -> Self {
        Self {
            roles: value.roles,
            posts: value.posts,
            depts: value.depts,
        }
    }
}

impl From<types::user::UserProfile> for ProfileResponse {
    fn from(value: types::user::UserProfile) -> Self {
        Self {
            user: value.user.into(),
            role_group: value.role_group,
            post_group: value.post_group,
            dept_name: value.dept_name,
        }
    }
}

impl From<crate::application::UserImportReport> for UserImportResponse {
    fn from(value: crate::application::UserImportReport) -> Self {
        Self {
            success_count: value.success_count,
            message: value.message,
        }
    }
}
