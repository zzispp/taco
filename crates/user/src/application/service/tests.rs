use crate::{
    application::{AppError, AppResult, UserService, UserUseCase},
    domain::Credentials,
    test_support::{
        MemoryLoginFailureStore, MemoryUserRepository, TestLoginLockConfigProvider, TestPasswordHasher, VALID_PASSWORD, new_user, replace_user, stored_user,
        user_id, user_service_with_login_security,
    },
};
use kernel::pagination::MAX_CURSOR_LIMIT;
use rbac::domain::{DataScope, DataScopeFilter};

mod admin;
mod audited;
mod auth;
mod installation_owner;
mod owner_protection;
mod profile;
mod support;

pub(super) use support::WithPassword;
use support::{profile_update, user_filter};
