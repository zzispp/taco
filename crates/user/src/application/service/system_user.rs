use constants::pagination::MIN_PAGE_NUMBER;
use kernel::error::LocalizedError;
use kernel::pagination::{Page, PageRequest, PageSliceRequest};

use crate::application::{AppError, AppResult, SystemUserProvider, SystemUserRecord, UserAuthRecord, UserListFilter, UserRepository};
use crate::domain::{User, UserId};

pub(super) fn reject_conflicting_system_user<S: SystemUserProvider>(system_users: &S, username: &str, email: &str) -> AppResult<()> {
    let Some(system) = system_users.system_user().map(|record| record.user) else {
        return Ok(());
    };
    reject_conflicting_field(username == system.username, "username")?;
    reject_conflicting_field(email == system.email, "email")
}

pub(super) fn reject_system_user_id<S: SystemUserProvider>(system_users: &S, id: &UserId) -> AppResult<()> {
    if system_user_by_id(system_users, id).is_some() {
        return Err(AppError::Conflict(localized("errors.user.system_user_immutable")));
    }
    Ok(())
}

pub(super) fn reject_protected_user_id<S: SystemUserProvider>(system_users: &S, id: &UserId) -> AppResult<()> {
    if id.0 == constants::system::SUPER_ADMIN_USER_ID {
        return Err(AppError::Conflict(localized("errors.user.system_user_immutable")));
    }
    reject_system_user_id(system_users, id)
}

pub(super) fn system_user_by_id<S: SystemUserProvider>(system_users: &S, id: &UserId) -> Option<SystemUserRecord> {
    system_users.system_user().filter(|system_user| system_user.user.id == *id)
}

pub(super) async fn find_auth_by_identifier<R, S>(repository: &R, system_users: &S, identifier: &str) -> AppResult<Option<UserAuthRecord>>
where
    R: UserRepository,
    S: SystemUserProvider,
{
    if let Some(found) = system_auth_by_identifier(system_users, identifier) {
        return Ok(Some(found));
    }
    if let Some(found) = repository.find_auth_by_username(identifier).await? {
        return Ok(Some(found));
    }
    repository.find_auth_by_email(identifier).await
}

pub(super) async fn list_with_system_user<R: UserRepository>(repository: &R, filter: UserListFilter, system_user: User) -> AppResult<Page<User>> {
    let page = filter.page;
    let matched = system_user_matches(&system_user, &filter);
    if !matched {
        return repository.list(filter).await;
    }

    let mut users = repository.list_slice(filter, system_user_slice(page)).await?;
    users.total += 1;
    if page.page == MIN_PAGE_NUMBER {
        users.items.insert(0, system_user);
    }
    Ok(users)
}

fn system_user_matches(user: &User, filter: &UserListFilter) -> bool {
    contains_filter(&user.username, &filter.username)
        && contains_filter(&user.nick_name, &filter.nick_name)
        && contains_filter(&user.email, &filter.email)
        && exact_filter(&user.sex, &filter.sex)
        && exact_filter(&user.status, &filter.status)
        && any_id_filter(&user.role_ids, &filter.role_ids)
        && filter.phonenumber.is_none()
        && filter.dept_id.is_none()
        && filter.dept_name.is_none()
        && filter.post_ids.is_empty()
        && filter.begin_time.is_none()
        && filter.end_time.is_none()
}

fn contains_filter(value: &str, filter: &Option<String>) -> bool {
    filter.as_ref().is_none_or(|needle| case_insensitive_contains(value, needle))
}

fn exact_filter(value: &str, filter: &Option<String>) -> bool {
    filter.as_ref().is_none_or(|expected| value == expected)
}

fn case_insensitive_contains(value: &str, needle: &str) -> bool {
    value.to_lowercase().contains(&needle.to_lowercase())
}

fn any_id_filter(values: &[String], filter: &[String]) -> bool {
    filter.is_empty() || filter.iter().any(|expected| values.contains(expected))
}

fn system_auth_by_identifier<S: SystemUserProvider>(system_users: &S, identifier: &str) -> Option<UserAuthRecord> {
    let system_user = system_users.system_user()?;
    let user = system_user.user;
    if identifier != user.username && identifier != user.email {
        return None;
    }
    Some(UserAuthRecord {
        user,
        password_hash: system_user.password_hash,
    })
}

fn system_user_slice(page: PageRequest) -> PageSliceRequest {
    if page.page == MIN_PAGE_NUMBER {
        return PageSliceRequest {
            offset: 0,
            limit: page.page_size.saturating_sub(1),
            page: page.page,
            page_size: page.page_size,
        };
    }
    PageSliceRequest {
        offset: (page.page - MIN_PAGE_NUMBER) * page.page_size - MIN_PAGE_NUMBER,
        limit: page.page_size,
        page: page.page,
        page_size: page.page_size,
    }
}

fn reject_conflicting_field(conflicting: bool, field: &str) -> AppResult<()> {
    if conflicting {
        return Err(AppError::Conflict(localized_param("errors.user.duplicate_field", "field", field)));
    }
    Ok(())
}

fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
}

fn localized_param(key: &'static str, param: &'static str, value: impl Into<String>) -> LocalizedError {
    LocalizedError::new(key).with_param(param, value)
}
