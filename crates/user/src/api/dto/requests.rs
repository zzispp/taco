use kernel::pagination::DEFAULT_CURSOR_LIMIT;
use serde::{Deserialize, Serialize};

use crate::{
    application::OnlineSessionFilter,
    domain::{Credentials, NewUser, ReplaceUser},
};

#[derive(Debug, Deserialize)]
pub struct CreateUserPayload {
    pub password: String,
    #[serde(flatten)]
    fields: UserFieldsPayload,
}

#[derive(Debug, Deserialize)]
pub struct ReplaceUserPayload {
    pub password: Option<String>,
    #[serde(flatten)]
    fields: UserFieldsPayload,
}

#[derive(Debug, Deserialize)]
struct UserFieldsPayload {
    username: String,
    nick_name: String,
    dept_id: Option<String>,
    email: String,
    phonenumber: Option<String>,
    sex: String,
    status: String,
    remark: Option<String>,
    role_ids: Vec<String>,
    post_ids: Vec<String>,
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

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ListUsersQuery {
    #[serde(default = "default_limit")]
    pub limit: u64,
    #[serde(default)]
    pub cursor: Option<String>,
    pub username: Option<String>,
    pub nick_name: Option<String>,
    pub phonenumber: Option<String>,
    pub email: Option<String>,
    pub sex: Option<String>,
    pub status: Option<String>,
    pub dept_id: Option<String>,
    pub dept_name: Option<String>,
    pub post_ids: Option<String>,
    pub role_ids: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserExportQuery {
    pub username: Option<String>,
    pub nick_name: Option<String>,
    pub phonenumber: Option<String>,
    pub email: Option<String>,
    pub sex: Option<String>,
    pub status: Option<String>,
    pub dept_id: Option<String>,
    pub dept_name: Option<String>,
    pub post_ids: Option<String>,
    pub role_ids: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OnlineSessionsQuery {
    #[serde(default = "default_limit")]
    pub limit: u64,
    #[serde(default)]
    pub cursor: Option<String>,
    pub ipaddr: Option<String>,
    #[serde(rename = "userName")]
    pub user_name: Option<String>,
    #[serde(rename = "loginLocation")]
    pub login_location: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

const fn default_limit() -> u64 {
    DEFAULT_CURSOR_LIMIT
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

impl From<CreateUserPayload> for NewUser {
    fn from(value: CreateUserPayload) -> Self {
        let fields = value.fields;
        Self {
            username: fields.username,
            password: value.password,
            nick_name: fields.nick_name,
            dept_id: fields.dept_id,
            email: fields.email,
            phonenumber: fields.phonenumber,
            sex: fields.sex,
            status: fields.status,
            remark: fields.remark,
            role_ids: fields.role_ids,
            post_ids: fields.post_ids,
        }
    }
}

impl From<ReplaceUserPayload> for ReplaceUser {
    fn from(value: ReplaceUserPayload) -> Self {
        let fields = value.fields;
        Self {
            username: fields.username,
            password: value.password.filter(|password| !password.trim().is_empty()),
            nick_name: fields.nick_name,
            dept_id: fields.dept_id,
            email: fields.email,
            phonenumber: fields.phonenumber,
            sex: fields.sex,
            status: fields.status,
            remark: fields.remark,
            role_ids: fields.role_ids,
            post_ids: fields.post_ids,
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

impl From<OnlineSessionsQuery> for OnlineSessionFilter {
    fn from(value: OnlineSessionsQuery) -> Self {
        Self {
            ipaddr: trim_optional(value.ipaddr),
            user_name: trim_optional(value.user_name),
            login_location: trim_optional(value.login_location),
            browser: trim_optional(value.browser),
            os: trim_optional(value.os),
            begin_time: trim_optional(value.begin_time),
            end_time: trim_optional(value.end_time),
        }
    }
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.map(|item| item.trim().to_owned()).filter(|item| !item.is_empty())
}
