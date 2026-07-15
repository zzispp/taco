use std::collections::HashSet;

use crate::{
    api::CurrentUser,
    application::{ApiCheckRequest, AuthorizationConfig, PermissionRequirement, RbacError, RbacResult, RoutePermissionRule},
    domain::{DataScope, DataScopeFilter, PermissionSnapshot},
};

use super::localization::{localized, localized_param};

pub(super) fn data_scope_filter(user: &CurrentUser, snapshot: &PermissionSnapshot) -> RbacResult<DataScopeFilter> {
    if user.admin {
        return Ok(scope_filter(user, DataScope::All, HashSet::new()));
    }
    let mut selected = DataScope::SelfOnly;
    let mut dept_ids = HashSet::new();
    for role in snapshot
        .roles
        .iter()
        .filter(|role| role.status == constants::system::STATUS_NORMAL && user.role_keys.contains(&role.role_key))
    {
        let scope = DataScope::try_from(role.data_scope.as_str()).map_err(|_| RbacError::InvalidInput(localized("errors.rbac.invalid_data_scope")))?;
        selected = selected.min(scope);
        if scope == DataScope::Custom {
            dept_ids.extend(role.dept_ids.iter().cloned());
        }
    }
    Ok(scope_filter(user, selected, dept_ids))
}

fn scope_filter(user: &CurrentUser, data_scope: DataScope, dept_ids: HashSet<String>) -> DataScopeFilter {
    DataScopeFilter {
        data_scope,
        user_id: user.id.clone(),
        dept_id: user.dept_id.clone(),
        dept_ids: dept_ids.into_iter().collect(),
    }
}

pub(super) fn validate_protected_handlers(config: &AuthorizationConfig) -> RbacResult<()> {
    let declared = inventory::iter::<crate::application::ProtectedHandler>.into_iter().collect::<Vec<_>>();
    for rule in config.route_permissions() {
        if !has_matching_declaration(&declared, rule) {
            return Err(RbacError::InvalidInput(localized_param(
                "errors.rbac.missing_handler_permission",
                "handler",
                rule.handler,
            )));
        }
    }
    for handler in declared {
        if !config.route_permissions().iter().any(|rule| has_matching_declaration(&[handler], rule)) {
            return Err(RbacError::InvalidInput(localized_param(
                "errors.rbac.missing_handler_permission",
                "handler",
                handler.function,
            )));
        }
    }
    Ok(())
}

fn has_matching_declaration(declared: &[&crate::application::ProtectedHandler], rule: &RoutePermissionRule) -> bool {
    declared
        .iter()
        .any(|handler| handler.function == rule.handler && handler.requirement.is_equivalent_to(rule.requirement))
}

pub(super) fn required_permission(config: &AuthorizationConfig, request: &ApiCheckRequest) -> RbacResult<PermissionRequirement> {
    config.required_permission(&request.method, &request.path).ok_or(RbacError::Forbidden)
}

pub(super) fn is_whitelisted(config: &AuthorizationConfig, method: &str, path: &str) -> RbacResult<bool> {
    Ok(config.is_whitelisted(method, path))
}
