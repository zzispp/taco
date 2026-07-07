use std::collections::HashSet;

use matchit::Router;

use crate::{
    api::CurrentUser,
    application::{ApiCheckRequest, AuthorizationConfig, RbacError, RbacResult},
    domain::{DataScopeFilter, PermissionSnapshot},
};

use super::localization::localized_param;

pub(super) fn data_scope_filter(user: &CurrentUser, snapshot: &PermissionSnapshot) -> DataScopeFilter {
    let roles = snapshot
        .roles
        .iter()
        .filter(|role| role.status == constants::system::STATUS_NORMAL && user.role_keys.contains(&role.role_key));
    let data_scope = roles.clone().map(|role| role.data_scope.as_str()).min().unwrap_or("5");
    let dept_ids = roles.flat_map(|role| role.dept_ids.clone()).collect::<HashSet<_>>();
    DataScopeFilter {
        data_scope: if user.admin { "1".into() } else { data_scope.into() },
        user_id: user.id.clone(),
        dept_id: user.dept_id.clone(),
        dept_ids: dept_ids.into_iter().collect(),
    }
}

pub(super) fn validate_protected_handlers(config: &AuthorizationConfig) -> RbacResult<()> {
    let declared = inventory::iter::<crate::application::ProtectedHandler>
        .into_iter()
        .map(|handler| (handler.function, handler.permission))
        .collect::<HashSet<_>>();
    for rule in &config.route_permissions {
        if !declared.contains(&(rule.handler, rule.permission.as_str())) {
            return Err(RbacError::InvalidInput(localized_param(
                "errors.rbac.missing_handler_permission",
                "handler",
                rule.handler,
            )));
        }
    }
    Ok(())
}

pub(super) fn validate_data_scope_handlers(handlers: &[&str]) -> RbacResult<()> {
    let declared = inventory::iter::<crate::application::DataScopeHandler>
        .into_iter()
        .map(|handler| handler.function)
        .collect::<HashSet<_>>();
    for handler in handlers {
        if !declared.contains(handler) {
            return Err(RbacError::InvalidInput(localized_param("errors.rbac.missing_data_scope", "handler", *handler)));
        }
    }
    Ok(())
}

pub(super) fn required_permission<'a>(config: &'a AuthorizationConfig, request: &ApiCheckRequest) -> RbacResult<&'a str> {
    config
        .route_permissions
        .iter()
        .find(|rule| route_rule_matches(rule, request).unwrap_or(false))
        .map(|rule| rule.permission.as_str())
        .ok_or(RbacError::Forbidden)
}

pub(super) fn route_rule_matches(rule: &types::rbac::RoutePermissionRule, request: &ApiCheckRequest) -> RbacResult<bool> {
    if !rule.methods.iter().any(|method| method.eq_ignore_ascii_case(&request.method)) {
        return Ok(false);
    }
    path_matches(&rule.path_pattern, &request.path)
}

pub(super) fn is_whitelisted(config: &AuthorizationConfig, method: &str, path: &str) -> RbacResult<bool> {
    let method = method.to_ascii_uppercase();
    config.whitelist.iter().try_fold(false, |matched, rule| {
        if matched || !rule.methods.iter().any(|item| item.eq_ignore_ascii_case(&method)) {
            return Ok(matched);
        }
        path_matches(&rule.path_pattern, path)
    })
}

pub(super) fn path_matches(pattern: &str, path: &str) -> RbacResult<bool> {
    let mut router = Router::new();
    router
        .insert(pattern, ())
        .map_err(|error| RbacError::InvalidInput(localized_param("errors.rbac.invalid_route_pattern", "error", error.to_string())))?;
    Ok(router.at(path).is_ok())
}
