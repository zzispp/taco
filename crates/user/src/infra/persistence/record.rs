use constants::auth::DEFAULT_AUTH_SOURCE;
use sqlx::FromRow;
use time::OffsetDateTime;
use types::user::{User, UserId};

use super::UserAuthRecord;

#[derive(Clone, Debug, PartialEq, FromRow)]
pub struct UserRecord {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub role: String,
    pub is_active: bool,
    pub is_deleted: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub last_login_at: Option<OffsetDateTime>,
    pub auth_source: String,
    pub email_verified: bool,
}

impl From<UserRecord> for User {
    fn from(value: UserRecord) -> Self {
        Self {
            id: UserId(value.id),
            username: value.username,
            email: value.email,
            role: value.role,
            is_active: value.is_active,
            auth_source: value.auth_source,
            email_verified: value.email_verified,
            system: false,
        }
    }
}

impl UserRecord {
    pub fn into_auth(self) -> UserAuthRecord {
        let password_hash = self.password_hash.clone();
        UserAuthRecord {
            user: self.into(),
            password_hash,
        }
    }

    pub fn local_auth_source() -> String {
        DEFAULT_AUTH_SOURCE.into()
    }
}
