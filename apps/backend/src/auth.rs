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
    authorize_request(&state, &method, &path, &current_user).await?;
    request.extensions_mut().insert(current_user);
    Ok(next.run(request).await)
}

async fn authenticate_current_user(state: &AuthState, headers: &HeaderMap) -> Result<CurrentUser, RbacError> {
    let token = bearer_token(headers)?;
    let user_id = state.tokens.validate_access(token).map_err(|_| RbacError::Unauthorized)?;
    let user = state.users.authenticated_user(user_id).await.map_err(user_error)?;
    Ok(current_user(user))
}

async fn authorize_request(state: &AuthState, method: &str, path: &str, current_user: &CurrentUser) -> Result<(), RbacError> {
    state
        .rbac
        .authorize_api(&state.authorization, user_check_request(method, path, current_user))
        .await
}

fn user_check_request(method: &str, path: &str, current_user: &CurrentUser) -> ApiCheckRequest {
    ApiCheckRequest {
        method: method.into(),
        path: path.into(),
        role_code: current_user.role.clone(),
        system: current_user.system,
    }
}

fn current_user(user: User) -> CurrentUser {
    CurrentUser {
        system: user.system,
        id: user.id.0,
        username: user.username,
        role: user.role,
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
        AppError::InvalidInput(_) | AppError::Unauthorized | AppError::Conflict(_) | AppError::NotFound => RbacError::Unauthorized,
    }
}
