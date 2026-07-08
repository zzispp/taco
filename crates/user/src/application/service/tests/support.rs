use kernel::pagination::PageRequest;

use crate::domain::{NewUser, ProfileUpdate, UserId};

pub(in crate::application::service) trait WithPassword {
    fn with_password(self, password: &str) -> Self;
    fn with_email(self, email: &str) -> Self;
}

impl WithPassword for NewUser {
    fn with_password(self, password: &str) -> Self {
        Self {
            password: password.into(),
            ..self
        }
    }

    fn with_email(self, email: &str) -> Self {
        Self { email: email.into(), ..self }
    }
}

pub(super) fn user_filter(page: u64, page_size: u64) -> crate::application::UserListFilter {
    crate::application::UserListFilter {
        page: PageRequest { page, page_size },
        username: None,
        nick_name: None,
        phonenumber: None,
        email: None,
        sex: None,
        status: None,
        dept_id: None,
        dept_name: None,
        post_ids: vec![],
        role_ids: vec![],
        begin_time: None,
        end_time: None,
    }
}

pub(super) fn profile_update(email: &str, phonenumber: Option<&str>) -> ProfileUpdate {
    ProfileUpdate {
        nick_name: "Alice".into(),
        phonenumber: phonenumber.map(str::to_owned),
        email: email.into(),
        sex: "2".into(),
    }
}

pub(super) fn super_admin_user_id() -> UserId {
    UserId(constants::system::SUPER_ADMIN_USER_ID.into())
}
