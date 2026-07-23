use crate::domain::{AvatarFileId, NewUser, ReplaceUser, User, UserId};

use super::{StoredUser, VALID_PASSWORD, business_role, role_summary, user_id};

impl StoredUser {
    pub(crate) fn with_id(mut self, id: UserId) -> Self {
        self.user.id = id;
        self
    }

    pub(crate) fn with_dept_id(mut self, dept_id: &str) -> Self {
        self.user.dept_id = Some(dept_id.into());
        self
    }

    pub(crate) fn with_nick_name(mut self, nick_name: &str) -> Self {
        self.user.nick_name = nick_name.into();
        self
    }

    pub(crate) fn with_email(mut self, email: &str) -> Self {
        self.user.email = email.into();
        self
    }

    pub(crate) fn with_sex(mut self, sex: &str) -> Self {
        self.user.sex = sex.into();
        self
    }

    pub(crate) fn with_status(mut self, status: &str) -> Self {
        self.user.status = status.into();
        self
    }

    pub(crate) fn with_avatar_file_id(mut self, file_id: &str) -> Self {
        self.user.avatar_file_id = Some(AvatarFileId::new(file_id).expect("avatar fixture id must be valid"));
        self.user.avatar_version = 1;
        self
    }

    pub(crate) fn with_role_ids(mut self, ids: Vec<&str>) -> Self {
        self.user.role_ids = ids.iter().map(|id| (*id).into()).collect();
        self.user.roles = self.user.role_ids.iter().map(|id| role_summary(id)).collect();
        self
    }

    pub(crate) fn with_post_ids(mut self, ids: Vec<&str>) -> Self {
        self.user.post_ids = ids.into_iter().map(str::to_owned).collect();
        self
    }
}

pub(crate) fn new_user(username: &str) -> NewUser {
    NewUser {
        username: username.into(),
        password: VALID_PASSWORD.into(),
        nick_name: username.trim().into(),
        dept_id: Some("103".into()),
        email: format!("{}@example.com", username.trim()),
        phonenumber: Some("15888888888".into()),
        sex: "2".into(),
        status: "0".into(),
        remark: None,
        role_ids: vec!["1".into()],
        post_ids: vec!["1".into()],
    }
}

pub(crate) fn replace_user(username: &str, is_active: bool) -> ReplaceUser {
    ReplaceUser {
        username: username.into(),
        password: Some(VALID_PASSWORD.into()),
        nick_name: username.trim().into(),
        dept_id: Some("103".into()),
        email: format!("{}@example.com", username.trim()),
        phonenumber: Some("15888888888".into()),
        sex: "2".into(),
        status: if is_active { "0".into() } else { "1".into() },
        remark: None,
        role_ids: vec!["1".into()],
        post_ids: vec!["1".into()],
    }
}

pub(crate) fn stored_user(id: u64, username: &str, password_hash: &str) -> StoredUser {
    StoredUser {
        user: User {
            id: user_id(id),
            username: username.into(),
            nick_name: username.into(),
            dept_id: Some("103".into()),
            email: format!("{username}@example.com"),
            phonenumber: Some("15888888888".into()),
            sex: "2".into(),
            avatar_file_id: None,
            avatar_version: 0,
            status: "0".into(),
            auth_source: "local".into(),
            email_verified: false,
            remark: None,
            roles: vec![business_role()],
            role_ids: vec!["1".into()],
            post_ids: vec!["1".into()],
            permissions: vec!["system:user:list".into()],
            create_time: String::new(),
        },
        password_hash: password_hash.into(),
    }
}
