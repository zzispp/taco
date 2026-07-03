use matchit::Router;

use crate::application::{ApiCheckRequest, AuthorizationConfig, RbacError, RbacResult};
use crate::domain::ApiPermissionSnapshot;

pub(super) fn is_whitelisted(config: &AuthorizationConfig, method: &str, path: &str) -> RbacResult<bool> {
    let method = method.to_ascii_uppercase();
    config.whitelist.iter().try_fold(false, |matched, rule| {
        if matched || !rule.methods.iter().any(|item| item.eq_ignore_ascii_case(&method)) {
            return Ok(matched);
        }
        path_matches(&rule.path_pattern, path)
    })
}

pub(super) fn authorize_snapshot(permissions: &[ApiPermissionSnapshot], request: &ApiCheckRequest) -> RbacResult<()> {
    let permission = permissions
        .iter()
        .find(|permission| api_permission_matches(permission, request).unwrap_or(false))
        .ok_or(RbacError::Forbidden)?;
    if permission.role_codes.iter().any(|code| code == &request.role_code) {
        return Ok(());
    }
    Err(RbacError::Forbidden)
}

fn api_permission_matches(permission: &ApiPermissionSnapshot, request: &ApiCheckRequest) -> RbacResult<bool> {
    if !permission.method.eq_ignore_ascii_case(&request.method) {
        return Ok(false);
    }
    path_matches(&permission.path_pattern, &request.path)
}

fn path_matches(pattern: &str, path: &str) -> RbacResult<bool> {
    let mut router = Router::new();
    router
        .insert(pattern, ())
        .map_err(|error| RbacError::InvalidInput(format!("invalid path pattern {pattern}: {error}")))?;
    Ok(router.at(path).is_ok())
}
