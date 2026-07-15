use kernel::error::LocalizedError;
use kernel::pagination::CursorPageRequest;
use regex::Regex;
use std::sync::LazyLock;

use crate::application::{AppError, AppResult, PasswordPolicy, UserListFilter};
use crate::domain::{Credentials, NewUser, ProfileUpdate, ReplaceUser};

static EMAIL_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").expect("email regex must compile"));
static PHONE_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^1[3-9]\d{9}$").expect("phone regex must compile"));

const USERNAME_MIN_LENGTH: usize = 3;
const USERNAME_MAX_LENGTH: usize = 30;

pub(super) fn validate_credentials(input: &Credentials) -> AppResult<()> {
    reject_blank("identifier", &input.identifier)?;
    reject_blank("password", &input.password)
}

pub(super) fn validate_new_user(input: &NewUser, policy: &PasswordPolicy) -> AppResult<()> {
    validate_username(&input.username)?;
    validate_password(&input.password, policy, Some(&input.username))?;
    reject_blank("nick_name", &input.nick_name)?;
    reject_blank("email", &input.email)?;
    validate_email(&input.email)?;
    validate_optional_phone(input.phonenumber.as_deref())?;
    reject_blank("status", &input.status)
}

pub(super) fn validate_replace_user(input: &ReplaceUser, policy: &PasswordPolicy) -> AppResult<()> {
    validate_username(&input.username)?;
    if let Some(password) = &input.password {
        validate_password(password, policy, Some(&input.username))?;
    }
    reject_blank("nick_name", &input.nick_name)?;
    reject_blank("email", &input.email)?;
    validate_email(&input.email)?;
    validate_optional_phone(input.phonenumber.as_deref())?;
    reject_blank("status", &input.status)
}

pub(super) fn validate_profile_update(input: &ProfileUpdate) -> AppResult<()> {
    reject_blank("nick_name", &input.nick_name)?;
    reject_blank("email", &input.email)?;
    validate_email(&input.email)?;
    validate_optional_phone(input.phonenumber.as_deref())?;
    reject_blank("sex", &input.sex)
}

pub(super) fn validate_page(page: &CursorPageRequest) -> AppResult<()> {
    crate::application::cursor::validate_cursor_request(page)
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
        email: normalize_email(input.email),
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
        email: normalize_email(input.email),
        phonenumber: trim_optional(input.phonenumber),
        sex: trim_required(input.sex),
        status: trim_required(input.status),
        remark: trim_optional(input.remark),
        role_ids: trim_ids(input.role_ids),
        post_ids: trim_ids(input.post_ids),
    }
}

pub(super) fn sanitize_profile_update(input: ProfileUpdate) -> ProfileUpdate {
    ProfileUpdate {
        nick_name: trim_required(input.nick_name),
        phonenumber: trim_optional(input.phonenumber),
        email: normalize_email(input.email),
        sex: trim_required(input.sex),
    }
}

pub(super) fn sanitize_filter(input: UserListFilter) -> UserListFilter {
    UserListFilter {
        page: input.page,
        username: trim_optional(input.username),
        nick_name: trim_optional(input.nick_name),
        phonenumber: trim_optional(input.phonenumber),
        email: trim_optional(input.email),
        sex: trim_optional(input.sex),
        status: trim_optional(input.status),
        dept_id: trim_optional(input.dept_id),
        dept_name: trim_optional(input.dept_name),
        post_ids: trim_ids(input.post_ids),
        role_ids: trim_ids(input.role_ids),
        begin_time: input.begin_time,
        end_time: input.end_time,
    }
}

fn validate_username(username: &str) -> AppResult<()> {
    reject_length(username, LengthRule::new("username", USERNAME_MIN_LENGTH, USERNAME_MAX_LENGTH))?;
    if !username.chars().all(is_username_character) {
        return Err(AppError::InvalidInput(localized("errors.user.username_chars")));
    }
    if !has_alphanumeric_edges(username) {
        return Err(AppError::InvalidInput(localized("errors.user.username_edges")));
    }
    Ok(())
}

fn validate_email(email: &str) -> AppResult<()> {
    if !EMAIL_PATTERN.is_match(email) {
        return Err(AppError::InvalidInput(localized("errors.validation.email_format")));
    }
    Ok(())
}

fn validate_optional_phone(phone: Option<&str>) -> AppResult<()> {
    if phone.is_some_and(|value| !PHONE_PATTERN.is_match(value)) {
        return Err(AppError::InvalidInput(localized("errors.validation.phone_format")));
    }
    Ok(())
}

pub(super) fn validate_password(password: &str, policy: &PasswordPolicy, username: Option<&str>) -> AppResult<()> {
    reject_length(password, LengthRule::new("password", policy.min_length, policy.max_length))?;
    reject_password_character_rules(password, policy)?;
    reject_password_contains_username(password, username, policy)
}

fn reject_password_character_rules(password: &str, policy: &PasswordPolicy) -> AppResult<()> {
    if policy.require_letter && !password.chars().any(|value| value.is_ascii_alphabetic()) {
        return Err(AppError::InvalidInput(localized("errors.user.password_letter_required")));
    }
    if policy.require_number && !password.chars().any(|value| value.is_ascii_digit()) {
        return Err(AppError::InvalidInput(localized("errors.user.password_number_required")));
    }
    if policy.require_symbol && !password.chars().any(is_symbol) {
        return Err(AppError::InvalidInput(localized("errors.user.password_symbol_required")));
    }
    Ok(())
}

fn reject_password_contains_username(password: &str, username: Option<&str>, policy: &PasswordPolicy) -> AppResult<()> {
    if !policy.forbid_username_contains {
        return Ok(());
    }
    let username = username.map(str::trim).filter(|value| !value.is_empty());
    if username.is_some_and(|value| password.to_lowercase().contains(&value.to_lowercase())) {
        return Err(AppError::InvalidInput(localized("errors.user.password_contains_username")));
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct LengthRule {
    field: &'static str,
    min: usize,
    max: usize,
}

impl LengthRule {
    const fn new(field: &'static str, min: usize, max: usize) -> Self {
        Self { field, min, max }
    }
}

fn reject_length(value: &str, rule: LengthRule) -> AppResult<()> {
    let length = value.chars().count();
    if length < rule.min || length > rule.max {
        return Err(AppError::InvalidInput(
            localized_param("errors.validation.length_between", "field", rule.field)
                .with_param("min", rule.min.to_string())
                .with_param("max", rule.max.to_string()),
        ));
    }
    Ok(())
}

fn is_symbol(value: char) -> bool {
    value.is_ascii_punctuation()
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
        return Err(AppError::InvalidInput(localized_param("errors.validation.field_blank", "field", field)));
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

fn normalize_email(value: String) -> String {
    value.trim().to_lowercase()
}

fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
}

fn localized_param(key: &'static str, param: &'static str, value: impl Into<String>) -> LocalizedError {
    LocalizedError::new(key).with_param(param, value)
}
