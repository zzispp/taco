use types::user::User;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserRecordInput {
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserAuthRecord {
    pub user: User,
    pub password_hash: String,
}
