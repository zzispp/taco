use constants::auth::{PASSWORD_MAX_LENGTH, PASSWORD_MIN_LENGTH, USERNAME_MAX_LENGTH, USERNAME_MIN_LENGTH};
use constants::pagination::{MAX_PAGE_SIZE, MIN_PAGE_NUMBER, MIN_PAGE_SIZE};
use kernel::pagination::PageRequest;

use crate::application::{AppError, AppResult};
use crate::domain::{Credentials, NewUser, ReplaceUser};

pub(super) fn validate_credentials(input: &Credentials) -> AppResult<()> {
    reject_blank("identifier", &input.identifier)?;
    validate_password(&input.password)
}

pub(super) fn validate_new_user(input: &NewUser) -> AppResult<()> {
    validate_username(&input.username)?;
    validate_password(&input.password)?;
    reject_blank("nick_name", &input.nick_name)?;
    reject_blank("email", &input.email)?;
    reject_blank("status", &input.status)?;
    reject_empty_ids("role_ids", &input.role_ids)
}

pub(super) fn validate_replace_user(input: &ReplaceUser) -> AppResult<()> {
    validate_username(&input.username)?;
    if let Some(password) = &input.password {
        validate_password(password)?;
    }
    reject_blank("nick_name", &input.nick_name)?;
    reject_blank("email", &input.email)?;
    reject_blank("status", &input.status)?;
    reject_empty_ids("role_ids", &input.role_ids)
}

pub(super) fn validate_page(page: PageRequest) -> AppResult<()> {
    if page.page < MIN_PAGE_NUMBER {
        return Err(AppError::InvalidInput("page must be greater than 0".into()));
    }
    if page.page_size < MIN_PAGE_SIZE {
        return Err(AppError::InvalidInput("page_size must be greater than 0".into()));
    }
    if page.page_size > MAX_PAGE_SIZE {
        return Err(AppError::InvalidInput(format!("page_size must be less than or equal to {MAX_PAGE_SIZE}")));
    }
    Ok(())
}

pub(super) fn sanitize_credentials(input: Credentials) -> Credentials {
    Credentials {
        identifier: input.identifier.trim().into(),
        password: input.password.trim().into(),
    }
}

pub(super) fn sanitize_new_user(input: NewUser) -> NewUser {
    NewUser {
        username: input.username.trim().into(),
        password: input.password.trim().into(),
        nick_name: trim_required(input.nick_name),
        dept_id: trim_optional(input.dept_id),
        email: input.email.trim().into(),
        phonenumber: trim_optional(input.phonenumber),
        sex: trim_required(input.sex),
        status: trim_required(input.status),
        remark: trim_optional(input.remark),
        role_ids: trim_ids(input.role_ids),
        post_ids: trim_ids(input.post_ids),
    }
}

pub(super) fn sanitize_replace_user(input: ReplaceUser) -> ReplaceUser {
    ReplaceUser {
        username: input.username.trim().into(),
        password: input.password.map(|password| password.trim().into()),
        nick_name: trim_required(input.nick_name),
        dept_id: trim_optional(input.dept_id),
        email: input.email.trim().into(),
        phonenumber: trim_optional(input.phonenumber),
        sex: trim_required(input.sex),
        status: trim_required(input.status),
        remark: trim_optional(input.remark),
        role_ids: trim_ids(input.role_ids),
        post_ids: trim_ids(input.post_ids),
    }
}

fn validate_username(username: &str) -> AppResult<()> {
    reject_length("username", username, USERNAME_MIN_LENGTH, USERNAME_MAX_LENGTH)?;
    if !username.chars().all(is_username_character) {
        return Err(AppError::InvalidInput(
            "username can only contain letters, numbers, underscores, and hyphens".into(),
        ));
    }
    if !has_alphanumeric_edges(username) {
        return Err(AppError::InvalidInput("username must start and end with a letter or number".into()));
    }
    Ok(())
}

pub(super) fn validate_password(password: &str) -> AppResult<()> {
    reject_length("password", password, PASSWORD_MIN_LENGTH, PASSWORD_MAX_LENGTH)
}

fn reject_length(field: &str, value: &str, min: usize, max: usize) -> AppResult<()> {
    let length = value.chars().count();
    if length < min || length > max {
        return Err(AppError::InvalidInput(format!("{field} must be between {min} and {max} characters")));
    }
    Ok(())
}

fn is_username_character(value: char) -> bool {
    value.is_ascii_alphanumeric() || matches!(value, '_' | '-')
}

fn has_alphanumeric_edges(value: &str) -> bool {
    value
        .chars()
        .next()
        .zip(value.chars().next_back())
        .is_some_and(|(first, last)| first.is_ascii_alphanumeric() && last.is_ascii_alphanumeric())
}

fn reject_blank(field: &str, value: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        return Err(AppError::InvalidInput(format!("{field} cannot be blank")));
    }
    Ok(())
}

fn reject_empty_ids(field: &str, values: &[String]) -> AppResult<()> {
    if values.is_empty() {
        return Err(AppError::InvalidInput(format!("{field} cannot be empty")));
    }
    Ok(())
}

fn trim_ids(values: Vec<String>) -> Vec<String> {
    values.into_iter().map(trim_required).filter(|value| !value.is_empty()).collect()
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.map(trim_required).filter(|value| !value.is_empty())
}

fn trim_required(value: String) -> String {
    value.trim().into()
}
