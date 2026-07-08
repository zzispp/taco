use kernel::pagination::PageRequest;
use serde::{Deserialize, Serialize};

use crate::{
    application::{OnlineSessionFilter, UserListFilter},
    domain::{Credentials, NewUser, ReplaceUser},
};

#[derive(Debug, Deserialize)]
pub struct UserPayload {
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

#[derive(Debug, Deserialize)]
pub struct SignUpPayload {
    pub username: String,
    pub password: String,
    pub email: String,
    #[serde(default)]
    pub captcha_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignInPayload {
    pub identifier: String,
    pub password: String,
    #[serde(default)]
    pub captcha_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenPayload {
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub page: u64,
    pub page_size: u64,
    pub username: Option<String>,
    pub phonenumber: Option<String>,
    pub status: Option<String>,
    pub dept_id: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct UserExportQuery {
    pub username: Option<String>,
    pub phonenumber: Option<String>,
    pub status: Option<String>,
    pub dept_id: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct OnlineSessionsQuery {
    pub ipaddr: Option<String>,
    #[serde(rename = "userName")]
    pub user_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordPayload {
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct StatusPayload {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ProfilePayload {
    pub nick_name: String,
    pub phonenumber: Option<String>,
    pub email: String,
    pub sex: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordPayload {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserRolesPayload {
    pub role_ids: Vec<String>,
}

impl From<UserPayload> for NewUser {
    fn from(value: UserPayload) -> Self {
        Self {
            username: value.username,
            password: value.password.unwrap_or_default(),
            nick_name: value.nick_name,
            dept_id: value.dept_id,
            email: value.email,
            phonenumber: value.phonenumber,
            sex: value.sex,
            status: value.status,
            remark: value.remark,
            role_ids: value.role_ids,
            post_ids: value.post_ids,
        }
    }
}

impl From<UserPayload> for ReplaceUser {
    fn from(value: UserPayload) -> Self {
        Self {
            username: value.username,
            password: value.password.filter(|password| !password.trim().is_empty()),
            nick_name: value.nick_name,
            dept_id: value.dept_id,
            email: value.email,
            phonenumber: value.phonenumber,
            sex: value.sex,
            status: value.status,
            remark: value.remark,
            role_ids: value.role_ids,
            post_ids: value.post_ids,
        }
    }
}

impl From<ProfilePayload> for types::user::ProfileUpdate {
    fn from(value: ProfilePayload) -> Self {
        Self {
            nick_name: value.nick_name,
            phonenumber: value.phonenumber,
            email: value.email,
            sex: value.sex,
        }
    }
}

impl From<SignInPayload> for Credentials {
    fn from(value: SignInPayload) -> Self {
        Self {
            identifier: value.identifier,
            password: value.password,
        }
    }
}

impl From<ListUsersQuery> for UserListFilter {
    fn from(value: ListUsersQuery) -> Self {
        Self {
            page: PageRequest {
                page: value.page,
                page_size: value.page_size,
            },
            username: value.username,
            phonenumber: value.phonenumber,
            status: value.status,
            dept_id: value.dept_id,
            begin_time: value.begin_time,
            end_time: value.end_time,
        }
    }
}

impl From<OnlineSessionsQuery> for OnlineSessionFilter {
    fn from(value: OnlineSessionsQuery) -> Self {
        Self {
            ipaddr: trim_optional(value.ipaddr),
            user_name: trim_optional(value.user_name),
        }
    }
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty())
}
