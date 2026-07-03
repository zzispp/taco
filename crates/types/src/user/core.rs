use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub auth_source: String,
    pub email_verified: bool,
    pub system: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceUser {
    pub username: String,
    pub password: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Credentials {
    pub identifier: String,
    pub password: String,
}
