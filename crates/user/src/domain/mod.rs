use std::fmt;

use serde::{Deserialize, Serialize};
use types::rbac::RoleSummary;

pub use types::user::{Credentials, NewUser, ProfileUpdate, ReplaceUser, UserFormOptions, UserId, UserProfileGroups};

const AVATAR_FILE_ID_BLANK: &str = "avatar file identifier cannot be blank";

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct AvatarFileId(String);

impl AvatarFileId {
    pub fn new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(AVATAR_FILE_ID_BLANK);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AvatarFileId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub nick_name: String,
    pub dept_id: Option<String>,
    pub email: String,
    pub phonenumber: Option<String>,
    pub sex: String,
    pub avatar_file_id: Option<AvatarFileId>,
    pub avatar_version: u64,
    pub status: String,
    pub auth_source: String,
    pub email_verified: bool,
    pub remark: Option<String>,
    pub roles: Vec<RoleSummary>,
    pub role_ids: Vec<String>,
    pub post_ids: Vec<String>,
    pub permissions: Vec<String>,
    pub create_time: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct UserProfile {
    pub user: User,
    pub role_group: String,
    pub post_group: String,
    pub dept_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avatar_file_id_rejects_blank_values() {
        assert_eq!(AvatarFileId::new("  ").unwrap_err(), AVATAR_FILE_ID_BLANK);
    }
}
