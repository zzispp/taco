use std::collections::HashSet;

use matchit::Router;
use types::rbac::{DATA_SCOPE_ALL, DATA_SCOPE_SELF};

use crate::{
    api::CurrentUser,
    application::{ApiCheckRequest, AuthorizationConfig, PermissionRequirement, RbacError, RbacResult, RoutePermissionRule},
    domain::{DataScopeFilter, PermissionSnapshot},
};

use super::localization::localized_param;

pub(super) fn data_scope_filter(user: &CurrentUser, snapshot: &PermissionSnapshot) -> DataScopeFilter {
    let roles = snapshot
        .roles
        .iter()
        .filter(|role| role.status == constants::system::STATUS_NORMAL && user.role_keys.contains(&role.role_key));
    let data_scope = roles.clone().map(|role| role.data_scope.as_str()).min().unwrap_or(DATA_SCOPE_SELF);
    let dept_ids = roles.flat_map(|role| role.dept_ids.clone()).collect::<HashSet<_>>();
    DataScopeFilter {
        data_scope: if user.admin { DATA_SCOPE_ALL.into() } else { data_scope.into() },
        user_id: user.id.clone(),
        dept_id: user.dept_id.clone(),
        dept_ids: dept_ids.into_iter().collect(),
    }
}

pub(super) fn validate_protected_handlers(config: &AuthorizationConfig) -> RbacResult<()> {
    let declared = inventory::iter::<crate::application::ProtectedHandler>.into_iter().collect::<Vec<_>>();
    for rule in &config.route_permissions {
        let matches = declared
            .iter()
            .any(|handler| handler.function == rule.handler && handler.requirement.is_equivalent_to(rule.requirement));
        if !matches {
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

pub(super) fn required_permission(config: &AuthorizationConfig, request: &ApiCheckRequest) -> RbacResult<PermissionRequirement> {
    for rule in &config.route_permissions {
        if route_rule_matches(rule, request)? {
            return Ok(rule.requirement);
        }
    }
    Err(RbacError::Forbidden)
}

pub(super) fn route_rule_matches(rule: &RoutePermissionRule, request: &ApiCheckRequest) -> RbacResult<bool> {
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
