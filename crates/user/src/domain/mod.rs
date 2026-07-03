use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct UserId(pub String);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
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

impl From<types::user::UserId> for UserId {
    fn from(value: types::user::UserId) -> Self {
        Self(value.0)
    }
}

impl From<UserId> for types::user::UserId {
    fn from(value: UserId) -> Self {
        Self(value.0)
    }
}

impl From<types::user::User> for User {
    fn from(value: types::user::User) -> Self {
        Self {
            id: value.id.into(),
            username: value.username,
            email: value.email,
            role: value.role,
            is_active: value.is_active,
            auth_source: value.auth_source,
            email_verified: value.email_verified,
            system: value.system,
        }
    }
}

impl From<User> for types::user::User {
    fn from(value: User) -> Self {
        Self {
            id: value.id.into(),
            username: value.username,
            email: value.email,
            role: value.role,
            is_active: value.is_active,
            auth_source: value.auth_source,
            email_verified: value.email_verified,
            system: value.system,
        }
    }
}
