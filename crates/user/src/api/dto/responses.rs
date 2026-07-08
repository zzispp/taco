use kernel::pagination::Page;
use serde::Serialize;
use types::http::{current_locale, translate_message_with_params};

use crate::{
    application::{OnlineSession, UserImportMessage},
    domain::{User, UserFormOptions},
};

const IMPORT_SUCCESS_SUMMARY_KEY: &str = "messages.user.import_success_summary";
const IMPORT_DETAIL_SEPARATOR: &str = "; ";
const USERNAME_PARAM: &str = "username";
const COUNT_PARAM: &str = "count";
const DETAILS_PARAM: &str = "details";

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineSessionResponse {
    pub token_id: String,
    pub dept_name: Option<String>,
    pub user_name: String,
    pub ipaddr: String,
    pub login_location: String,
    pub browser: String,
    pub os: String,
    pub login_time: i64,
}

#[derive(Debug, Serialize)]
pub struct OnlineSessionsResponse {
    pub rows: Vec<OnlineSessionResponse>,
    pub total: usize,
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
        let message = localized_import_summary(value.success_count, &value.messages);
        Self {
            success_count: value.success_count,
            message,
        }
    }
}

fn localized_import_summary(success_count: usize, messages: &[UserImportMessage]) -> String {
    let locale = current_locale();
    let count = success_count.to_string();
    let details = messages
        .iter()
        .map(|message| translate_message_with_params(locale, message.key, &[(USERNAME_PARAM, message.username.clone())]))
        .collect::<Vec<_>>()
        .join(IMPORT_DETAIL_SEPARATOR);
    translate_message_with_params(locale, IMPORT_SUCCESS_SUMMARY_KEY, &[(COUNT_PARAM, count), (DETAILS_PARAM, details)])
}

impl From<OnlineSession> for OnlineSessionResponse {
    fn from(value: OnlineSession) -> Self {
        Self {
            token_id: value.token_id,
            dept_name: value.dept_name,
            user_name: value.user_name,
            ipaddr: value.ipaddr,
            login_location: value.login_location,
            browser: value.browser,
            os: value.os,
            login_time: value.login_time,
        }
    }
}

impl From<Vec<OnlineSession>> for OnlineSessionsResponse {
    fn from(value: Vec<OnlineSession>) -> Self {
        let total = value.len();
        Self {
            rows: value.into_iter().map(OnlineSessionResponse::from).collect(),
            total,
        }
    }
}
