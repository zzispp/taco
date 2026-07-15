use crate::{application::UserListFilter, domain::User};
use rbac::domain::{DataScope, DataScopeFilter};

pub(super) fn memory_scope_matches(user: &User, scope: &DataScopeFilter) -> bool {
    match scope.data_scope {
        DataScope::All => true,
        DataScope::Custom => user.dept_id.as_ref().is_some_and(|id| scope.dept_ids.contains(id)),
        DataScope::Department => user.dept_id == scope.dept_id,
        DataScope::SelfOnly => user.id.0 == scope.user_id,
        DataScope::DepartmentAndChildren => user.dept_id == scope.dept_id || user.dept_id.as_ref().is_some_and(|id| scope.dept_ids.contains(id)),
    }
}

pub(super) fn memory_filter_matches(user: &User, filter: &UserListFilter) -> bool {
    contains_filter(&user.username, &filter.username)
        && contains_filter(&user.nick_name, &filter.nick_name)
        && contains_filter(&user.email, &filter.email)
        && contains_optional_filter(&user.phonenumber, &filter.phonenumber)
        && contains_optional_filter(&dept_name(user), &filter.dept_name)
        && exact_filter(&user.sex, &filter.sex)
        && exact_filter(&user.status, &filter.status)
        && exact_optional_filter(&user.dept_id, &filter.dept_id)
        && any_id_filter(&user.post_ids, &filter.post_ids)
        && any_id_filter(&user.role_ids, &filter.role_ids)
}

fn dept_name(user: &User) -> Option<String> {
    user.dept_id.as_ref().map(|id| format!("部门{id}"))
}

fn contains_filter(value: &str, filter: &Option<String>) -> bool {
    filter.as_ref().is_none_or(|needle| case_insensitive_contains(value, needle))
}

fn contains_optional_filter(value: &Option<String>, filter: &Option<String>) -> bool {
    filter
        .as_ref()
        .is_none_or(|needle| value.as_ref().is_some_and(|item| case_insensitive_contains(item, needle)))
}

fn exact_filter(value: &str, filter: &Option<String>) -> bool {
    filter.as_ref().is_none_or(|expected| value == expected)
}

fn exact_optional_filter(value: &Option<String>, filter: &Option<String>) -> bool {
    filter.as_ref().is_none_or(|expected| value.as_deref() == Some(expected.as_str()))
}

fn any_id_filter(values: &[String], filter: &[String]) -> bool {
    filter.is_empty() || filter.iter().any(|expected| values.contains(expected))
}

fn case_insensitive_contains(value: &str, needle: &str) -> bool {
    value.to_lowercase().contains(&needle.to_lowercase())
}
