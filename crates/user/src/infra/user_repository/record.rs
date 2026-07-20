use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct UserRecord {
    pub user_id: String,
    pub dept_id: Option<String>,
    pub user_name: String,
    pub nick_name: String,
    pub email: String,
    pub phonenumber: Option<String>,
    pub sex: String,
    pub avatar: Option<String>,
    pub password: String,
    pub status: String,
    pub is_installation_owner: bool,
    pub auth_source: String,
    pub email_verified: bool,
    pub remark: Option<String>,
    pub create_time: OffsetDateTime,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct UserRoleRecord {
    pub user_id: String,
    pub role_id: String,
    pub role_name: String,
    pub role_key: String,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct UserRelationValueRecord {
    pub user_id: String,
    pub value: String,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct AuthorizationUserRecord {
    pub user_id: String,
    pub user_name: String,
    pub dept_id: Option<String>,
    pub status: String,
    pub is_installation_owner: bool,
    pub role_keys: Vec<String>,
    pub permissions: Vec<String>,
}
