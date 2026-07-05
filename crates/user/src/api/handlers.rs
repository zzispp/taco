use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::{HeaderMap, header::AUTHORIZATION},
};
use rbac::api::CurrentUser;
use rbac_macros::{data_scope, require_perms};
use types::rbac::DataScopeFilter;
use types::{http::RequestJson, system::BatchIdsInput};

use crate::{
    api::{
        ApiState, TokenPair,
        dto::{
            AuthSessionResponse, ListUsersQuery, MeResponse, RefreshTokenPayload, ResetPasswordPayload, SignInPayload, SignUpPayload, StatusPayload,
            TokenPairResponse, UserFormOptionsResponse, UserPayload, UserResponse, UserRolesPayload, UsersPageResponse,
        },
        error::ApiError,
    },
    domain::{NewUser, UserId},
};

type ApiResult<T> = Result<T, ApiError>;
type ApiJson<T> = Json<T>;

pub async fn sign_up(State(state): State<ApiState>, RequestJson(payload): RequestJson<SignUpPayload>) -> ApiResult<ApiJson<AuthSessionResponse>> {
    let user = state.users.sign_up(new_sign_up_user(payload)).await?;
    let tokens = state.tokens.issue_pair(user.id.clone())?;
    Ok(ok(AuthSessionResponse::new(user.into(), tokens)))
}

pub async fn sign_in(State(state): State<ApiState>, RequestJson(payload): RequestJson<SignInPayload>) -> ApiResult<ApiJson<AuthSessionResponse>> {
    let user = state.users.sign_in(payload.into()).await?;
    let tokens = state.tokens.issue_pair(user.id.clone())?;
    Ok(ok(AuthSessionResponse::new(user.into(), tokens)))
}

pub async fn refresh(State(state): State<ApiState>, RequestJson(payload): RequestJson<RefreshTokenPayload>) -> ApiResult<ApiJson<TokenPairResponse>> {
    let (user_id, tokens) = state.tokens.refresh(&payload.refresh_token)?;
    state.users.authenticated_user(user_id).await?;
    Ok(ok(tokens.into()))
}

pub async fn me(State(state): State<ApiState>, headers: HeaderMap) -> ApiResult<ApiJson<MeResponse>> {
    let access_token = bearer_token(&headers)?;
    let user_id = state.tokens.validate_access(access_token)?;
    let user = state.users.authenticated_user(user_id).await?;
    Ok(ok(MeResponse { user: user.into() }))
}

#[require_perms("system:user:add")]
pub async fn create_user(State(state): State<ApiState>, RequestJson(payload): RequestJson<UserPayload>) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.create_user(payload.into()).await?;
    Ok(ok(user.into()))
}

#[require_perms("system:user:edit")]
pub async fn replace_user(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<UserPayload>,
) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.replace_user(UserId(id), payload.into()).await?;
    Ok(ok(user.into()))
}

#[require_perms("system:user:remove")]
pub async fn delete_user(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.users.delete_user(UserId(id)).await?;
    Ok(ok(()))
}

#[require_perms("system:user:remove")]
pub async fn delete_users(State(state): State<ApiState>, RequestJson(payload): RequestJson<BatchIdsInput>) -> ApiResult<ApiJson<()>> {
    state.users.delete_users(payload.ids.into_iter().map(UserId).collect()).await?;
    Ok(ok(()))
}

#[require_perms("system:user:query")]
pub async fn get_user(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.get_user(UserId(id)).await?;
    Ok(ok(user.into()))
}

#[require_perms("system:user:resetPwd")]
pub async fn reset_user_password(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<ResetPasswordPayload>,
) -> ApiResult<ApiJson<()>> {
    state.users.reset_password(UserId(id), payload.password).await?;
    Ok(ok(()))
}

#[require_perms("system:user:edit")]
pub async fn update_user_status(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<StatusPayload>,
) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.update_status(UserId(id), payload.status).await?;
    Ok(ok(user.into()))
}

#[require_perms("system:user:query")]
pub async fn user_roles(State(state): State<ApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<UserRolesPayload>> {
    let user = state.users.get_user(UserId(id)).await?;
    Ok(ok(UserRolesPayload { role_ids: user.role_ids }))
}

#[require_perms("system:user:edit")]
pub async fn replace_user_roles(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<UserRolesPayload>,
) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.replace_roles(UserId(id), payload.role_ids).await?;
    Ok(ok(user.into()))
}

#[require_perms("system:user:list")]
pub async fn user_form_options(State(state): State<ApiState>) -> ApiResult<ApiJson<UserFormOptionsResponse>> {
    let response: UserFormOptionsResponse = state.users.form_options().await?.into();
    Ok(ok(response))
}

#[require_perms("system:user:list")]
pub async fn user_dept_tree(State(state): State<ApiState>) -> ApiResult<ApiJson<Vec<types::system::TreeSelectNode>>> {
    Ok(ok(state.users.form_options().await?.depts))
}

#[require_perms("system:user:list")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn list_users(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(data_scope): Extension<DataScopeFilter>,
    Query(query): Query<ListUsersQuery>,
) -> ApiResult<ApiJson<UsersPageResponse>> {
    let page = if current_user.admin {
        state.users.list_users(query.into()).await?
    } else {
        state.users.list_users_scoped(query.into(), data_scope).await?
    };
    Ok(ok(page.into()))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(data)
}

fn new_sign_up_user(payload: SignUpPayload) -> NewUser {
    NewUser {
        nick_name: payload.username.clone(),
        username: payload.username,
        password: payload.password,
        dept_id: None,
        email: payload.email,
        phonenumber: None,
        sex: "2".into(),
        status: "0".into(),
        remark: None,
        role_ids: vec!["2".into()],
        post_ids: vec![],
    }
}

impl AuthSessionResponse {
    pub fn new(user: UserResponse, tokens: TokenPair) -> Self {
        Self {
            user,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        }
    }
}

impl From<TokenPair> for TokenPairResponse {
    fn from(value: TokenPair) -> Self {
        Self {
            access_token: value.access_token,
            refresh_token: value.refresh_token,
        }
    }
}

fn bearer_token(headers: &HeaderMap) -> ApiResult<&str> {
    let value = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError(crate::application::AppError::Unauthorized))?;

    value.strip_prefix("Bearer ").ok_or(ApiError(crate::application::AppError::Unauthorized))
}
