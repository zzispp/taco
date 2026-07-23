use constants::system::STATUS_NORMAL;

use crate::domain::NewUser;

const UNKNOWN_SEX: &str = "2";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootstrapAdministratorInput {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl BootstrapAdministratorInput {
    pub fn into_new_user(self) -> NewUser {
        NewUser {
            nick_name: self.username.clone(),
            username: self.username,
            email: self.email,
            password: self.password,
            dept_id: None,
            phonenumber: None,
            sex: UNKNOWN_SEX.into(),
            status: STATUS_NORMAL.into(),
            remark: None,
            role_ids: Vec::new(),
            post_ids: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootstrapAdministratorRecord {
    pub username: String,
    pub nick_name: String,
    pub email: String,
    pub password_hash: String,
}

impl BootstrapAdministratorRecord {
    pub fn from_new_user(user: NewUser, password_hash: String) -> Self {
        Self {
            username: user.username,
            nick_name: user.nick_name,
            email: user.email,
            password_hash,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BootstrapAdministratorOutcome {
    Created,
    AlreadyPresent,
}
