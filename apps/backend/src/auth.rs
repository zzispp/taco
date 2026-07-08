use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{HeaderMap, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};
use rbac::{
    api::{CurrentUser, RbacApiError},
    application::{ApiCheckRequest, AuthorizationConfig, RbacError, RbacUseCase},
};
use user::{
    api::TokenService,
    application::{AppError, UserUseCase},
    domain::User,
};

const AUTHENTICATED_ONLY_ROUTES: &[AuthenticatedOnlyRoute] = &[
    AuthenticatedOnlyRoute {
        method: "GET",
        path: "/api/navbar",
    },
    AuthenticatedOnlyRoute {
        method: "POST",
        path: "/api/auth/logout",
    },
    AuthenticatedOnlyRoute {
        method: "GET",
        path: "/api/account/profile",
    },
    AuthenticatedOnlyRoute {
        method: "PUT",
        path: "/api/account/profile",
    },
    AuthenticatedOnlyRoute {
        method: "PUT",
        path: "/api/account/profile/password",
    },
    AuthenticatedOnlyRoute {
        method: "POST",
        path: "/api/account/profile/avatar",
    },
];

struct AuthenticatedOnlyRoute {
    method: &'static str,
    path: &'static str,
}

#[derive(Clone)]
pub struct AuthState {
    users: Arc<dyn UserUseCase>,
    tokens: TokenService,
    rbac: Arc<dyn RbacUseCase>,
    authorization: AuthorizationConfig,
}

pub struct AuthStateParts {
    pub users: Arc<dyn UserUseCase>,
    pub tokens: TokenService,
    pub rbac: Arc<dyn RbacUseCase>,
    pub authorization: AuthorizationConfig,
}

impl AuthState {
    pub fn new(parts: AuthStateParts) -> Self {
        Self {
            users: parts.users,
            tokens: parts.tokens,
            rbac: parts.rbac,
            authorization: parts.authorization,
        }
    }
}

pub async fn auth_middleware(State(state): State<AuthState>, mut request: Request, next: Next) -> Result<Response, RbacApiError> {
    let method = request.method().as_str().to_owned();
    let path = request.uri().path().to_owned();
    if state.rbac.is_whitelisted(&state.authorization, &method, &path)? {
        return Ok(next.run(request).await);
    }

    let current_user = authenticate_current_user(&state, request.headers()).await?;
    if is_authenticated_only_route(&method, &path) {
        request.extensions_mut().insert(current_user);
        return Ok(next.run(request).await);
    }

    authorize_request(&state, user_check_request(&method, &path, &current_user)).await?;
    let data_scope = state.rbac.data_scope_filter(&current_user).await?;
    request.extensions_mut().insert(data_scope);
    request.extensions_mut().insert(current_user);
    Ok(next.run(request).await)
}

fn is_authenticated_only_route(method: &str, path: &str) -> bool {
    AUTHENTICATED_ONLY_ROUTES
        .iter()
        .any(|route| route.path == path && route.method.eq_ignore_ascii_case(method))
}

async fn authenticate_current_user(state: &AuthState, headers: &HeaderMap) -> Result<CurrentUser, RbacError> {
    let token = bearer_token(headers)?;
    let user_id = state.tokens.validate_access(token).await.map_err(|_| RbacError::Unauthorized)?;
    let user = state.users.authenticated_user(user_id).await.map_err(user_error)?;
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

fn current_user(user: User) -> CurrentUser {
    let role_keys = user.roles.iter().map(|role| role.role_key.clone()).collect::<Vec<_>>();
    let admin = user.system || role_keys.iter().any(|role_key| role_key == "admin");
    CurrentUser {
        system: user.system,
        id: user.id.0,
        username: user.username,
        role_keys,
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
        AppError::InvalidInput(_) | AppError::Unauthorized | AppError::Forbidden(_) | AppError::Conflict(_) | AppError::NotFound => RbacError::Unauthorized,
    }
}

#[cfg(test)]
mod tests {
    use super::is_authenticated_only_route;

    #[test]
    fn authenticated_only_route_matches_navbar() {
        assert!(is_authenticated_only_route("GET", "/api/navbar"));
        assert!(is_authenticated_only_route("get", "/api/navbar"));
        assert!(!is_authenticated_only_route("POST", "/api/navbar"));
        assert!(!is_authenticated_only_route("GET", "/api/system/users"));
    }

    #[test]
    fn authenticated_only_route_matches_account_profile() {
        assert!(is_authenticated_only_route("GET", "/api/account/profile"));
        assert!(is_authenticated_only_route("PUT", "/api/account/profile"));
        assert!(is_authenticated_only_route("PUT", "/api/account/profile/password"));
        assert!(is_authenticated_only_route("POST", "/api/account/profile/avatar"));
    }
}
