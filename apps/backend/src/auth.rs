use std::sync::Arc;

use audit_contract::{ActorSnapshot, EndpointAccess, OperationAuditContext};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use constants::system::SUPER_ADMIN_ROLE_KEY;
use rbac::{
    api::{CurrentUser, RbacApiError},
    application::{ApiCheckRequest, AuthorizationConfig, RbacError, RbacUseCase},
};
use system::application::SystemUseCase;
use user::{
    api::TokenService,
    application::{AppError, AuthorizationUser, UserUseCase},
};

use crate::composition::access_catalog::EndpointCatalog;

const UNEXPECTED_AUTHORIZATION_CURSOR_ERROR: &str = "infra.user.authorization.unexpected_cursor";

#[derive(Clone)]
pub struct AuthState {
    users: Arc<dyn UserUseCase>,
    tokens: TokenService,
    rbac: Arc<dyn RbacUseCase>,
    system: Arc<dyn SystemUseCase>,
    authorization: AuthorizationConfig,
    endpoints: EndpointCatalog,
}

pub struct AuthStateParts {
    pub users: Arc<dyn UserUseCase>,
    pub tokens: TokenService,
    pub rbac: Arc<dyn RbacUseCase>,
    pub system: Arc<dyn SystemUseCase>,
    pub authorization: AuthorizationConfig,
    pub endpoints: EndpointCatalog,
}

impl AuthState {
    pub fn new(parts: AuthStateParts) -> Self {
        Self {
            users: parts.users,
            tokens: parts.tokens,
            rbac: parts.rbac,
            system: parts.system,
            authorization: parts.authorization,
            endpoints: parts.endpoints,
        }
    }
}

pub async fn auth_middleware(State(state): State<AuthState>, mut request: Request, next: Next) -> Result<Response, RbacApiError> {
    let method = request.method().as_str().to_owned();
    let path = request.uri().path().to_owned();
    let access = state.endpoints.access(&method, &path);
    if allows_unauthenticated(access) || (access.is_none() && state.rbac.is_whitelisted(&state.authorization, &method, &path)?) {
        return Ok(next.run(request).await);
    }

    let current_user = authenticate_current_user(&state, request.headers()).await?;
    attach_current_user(&state, &mut request, current_user.clone()).await?;
    if accepts_authenticated_actor(access) {
        return Ok(next.run(request).await);
    }

    authorize_request(&state, user_check_request(&method, &path, &current_user)).await?;
    if requires_data_scope(access) {
        let data_scope = state.rbac.data_scope_filter(&current_user).await?;
        request.extensions_mut().insert(data_scope);
    }
    Ok(next.run(request).await)
}

fn allows_unauthenticated(access: Option<EndpointAccess>) -> bool {
    matches!(access, Some(EndpointAccess::Public))
}

fn accepts_authenticated_actor(access: Option<EndpointAccess>) -> bool {
    matches!(access, Some(EndpointAccess::SelfAuthenticated | EndpointAccess::Authenticated))
}

fn requires_data_scope(access: Option<EndpointAccess>) -> bool {
    matches!(access, Some(EndpointAccess::DataScopedPermission(_)))
}

async fn attach_current_user(state: &AuthState, request: &mut Request, current_user: CurrentUser) -> Result<(), RbacError> {
    if let Some(context) = request.extensions().get::<OperationAuditContext>().cloned() {
        context
            .set_actor(audit_actor(state, &current_user).await?)
            .map_err(|message| RbacError::Infrastructure(message.into()))?;
    }
    request.extensions_mut().insert(current_user);
    Ok(())
}

async fn audit_actor(state: &AuthState, user: &CurrentUser) -> Result<ActorSnapshot, RbacError> {
    let department_name = match user.dept_id.as_deref() {
        Some(id) => {
            state
                .system
                .get_dept(id)
                .await
                .map_err(|error| RbacError::Infrastructure(error.to_string()))?
                .dept_name
        }
        None => String::new(),
    };
    Ok(ActorSnapshot {
        user_id: Some(user.id.clone()),
        username: user.username.clone(),
        department_id: user.dept_id.clone(),
        department_name,
    })
}

async fn authenticate_current_user(state: &AuthState, headers: &HeaderMap) -> Result<CurrentUser, RbacError> {
    let token = bearer_token(headers)?;
    let user_id = state.tokens.validate_access(token).await.map_err(|_| RbacError::Unauthorized)?;
    let user = state.users.authorization_user(user_id).await.map_err(user_error)?;
    Ok(current_user(user))
}

async fn authorize_request(state: &AuthState, request: ApiCheckRequest) -> Result<(), RbacError> {
    state.rbac.authorize_api(&state.authorization, request).await
}

fn user_check_request(method: &str, path: &str, current_user: &CurrentUser) -> ApiCheckRequest {
    ApiCheckRequest {
        method: method.into(),
        path: path.into(),
        role_keys: current_user.role_keys.clone(),
        permissions: current_user.permissions.clone(),
        admin: current_user.admin,
    }
}

fn current_user(user: AuthorizationUser) -> CurrentUser {
    let admin = user.role_keys.iter().any(|role_key| role_key == SUPER_ADMIN_ROLE_KEY);
    CurrentUser {
        id: user.id.0,
        username: user.username,
        role_keys: user.role_keys,
        permissions: user.permissions,
        dept_id: user.dept_id,
        admin,
    }
}

fn bearer_token(headers: &HeaderMap) -> Result<&str, RbacError> {
    let value = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(RbacError::Unauthorized)?;
    value.strip_prefix("Bearer ").ok_or(RbacError::Unauthorized)
}

fn user_error(error: AppError) -> RbacError {
    match error {
        AppError::Infrastructure(message) => RbacError::Infrastructure(message),
        AppError::InvalidCursor => RbacError::Infrastructure(UNEXPECTED_AUTHORIZATION_CURSOR_ERROR.into()),
        AppError::InvalidInput(_)
        | AppError::ImportValidation(_)
        | AppError::Unauthorized
        | AppError::AccountDisabled
        | AppError::AccountLocked { .. }
        | AppError::Forbidden(_)
        | AppError::Conflict(_)
        | AppError::NotFound => RbacError::Unauthorized,
    }
}

#[cfg(test)]
mod tests {
    use audit_contract::EndpointAccess;
    use constants::system::SUPER_ADMIN_ROLE_KEY;
    use types::user::UserId;
    use user::application::AuthorizationUser;

    use super::{accepts_authenticated_actor, allows_unauthenticated, current_user, requires_data_scope};

    #[test]
    fn self_authenticated_routes_require_a_token_without_permission_authorization() {
        assert!(!allows_unauthenticated(Some(EndpointAccess::SelfAuthenticated)));
        assert!(accepts_authenticated_actor(Some(EndpointAccess::SelfAuthenticated)));
    }

    #[test]
    fn public_routes_do_not_require_a_token() {
        assert!(allows_unauthenticated(Some(EndpointAccess::Public)));
        assert!(!accepts_authenticated_actor(Some(EndpointAccess::Public)));
    }

    #[test]
    fn only_explicit_data_scoped_permissions_require_scope_injection() {
        let permission = audit_contract::EndpointPermission {
            handler: "list_users",
            requirement: audit_contract::EndpointPermissionRequirement::all_of(&["system:user:list"]),
        };

        assert!(requires_data_scope(Some(EndpointAccess::DataScopedPermission(permission))));
        assert!(!requires_data_scope(Some(EndpointAccess::Permission(permission))));
        assert!(!requires_data_scope(Some(EndpointAccess::Authenticated)));
    }

    #[test]
    fn current_user_uses_the_shared_super_admin_role_key() {
        assert!(current_user(user_with_role(SUPER_ADMIN_ROLE_KEY)).admin);
        assert!(!current_user(user_with_role("administrator")).admin);
    }

    fn user_with_role(role_key: &str) -> AuthorizationUser {
        AuthorizationUser {
            id: UserId("test-user".into()),
            username: "tester".into(),
            dept_id: None,
            status: "0".into(),
            role_keys: vec![role_key.into()],
            permissions: Vec::new(),
        }
    }
}
