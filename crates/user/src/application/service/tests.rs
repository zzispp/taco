use crate::{
    application::{AppError, UserService, UserUseCase},
    domain::Credentials,
    test_support::{MemoryUserRepository, TestPasswordHasher, VALID_PASSWORD, new_user, replace_user, stored_user, user_id},
};
use constants::pagination::MAX_PAGE_SIZE;
use types::rbac::{DATA_SCOPE_CUSTOM, DATA_SCOPE_SELF, DataScopeFilter};

mod admin;
mod auth;
mod profile;
mod support;

pub(super) use support::WithPassword;
use support::{profile_update, super_admin_user_id, user_filter};
