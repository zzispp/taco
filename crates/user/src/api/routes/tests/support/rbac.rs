use rbac::api::CurrentUser;
use types::rbac::{DATA_SCOPE_ALL, DATA_SCOPE_SELF, DataScopeFilter};

use crate::test_support::user_id;

pub(crate) fn admin_current_user() -> CurrentUser {
    current_user(CurrentUserFixture {
        id: user_id(1).0,
        username: "alice".into(),
        dept_id: Some("103".into()),
        admin: true,
    })
}

pub(crate) fn self_current_user(user: u64, username: &str, dept_id: &str) -> CurrentUser {
    current_user(CurrentUserFixture {
        id: user_id(user).0,
        username: username.into(),
        dept_id: Some(dept_id.into()),
        admin: false,
    })
}

pub(crate) fn all_data_scope() -> DataScopeFilter {
    DataScopeFilter {
        data_scope: DATA_SCOPE_ALL.into(),
        user_id: user_id(1).0,
        dept_id: Some("103".into()),
        dept_ids: vec![],
    }
}

pub(crate) fn self_data_scope(user: u64, dept_id: &str) -> DataScopeFilter {
    DataScopeFilter {
        data_scope: DATA_SCOPE_SELF.into(),
        user_id: user_id(user).0,
        dept_id: Some(dept_id.into()),
        dept_ids: vec![],
    }
}

struct CurrentUserFixture {
    id: String,
    username: String,
    dept_id: Option<String>,
    admin: bool,
}

fn current_user(fixture: CurrentUserFixture) -> CurrentUser {
    CurrentUser {
        id: fixture.id,
        username: fixture.username,
        role_keys: vec!["admin".into()],
        permissions: vec!["system:online:list".into(), "system:online:forceLogout".into()],
        dept_id: fixture.dept_id,
        admin: fixture.admin,
        system: false,
    }
}
