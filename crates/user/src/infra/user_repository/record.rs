use sqlx::FromRow;

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
    pub auth_source: String,
    pub email_verified: bool,
    pub remark: Option<String>,
    pub create_time: String,
}

#[derive(Clone, Debug, FromRow, PartialEq, Eq)]
pub struct RoleSummaryRecord {
    pub role_id: String,
    pub role_name: String,
    pub role_key: String,
}
