use axum::{
    Extension, Json,
    extract::{Multipart, Path, Query, State},
    http::{HeaderMap, header::AUTHORIZATION},
    response::Response,
};
use constants::system_config::{INIT_PASSWORD_KEY, REGISTER_USER_KEY};
use kernel::error::LocalizedError;
use rbac::api::CurrentUser;
use rbac_macros::{data_scope, require_perms};
use types::rbac::DataScopeFilter;
use types::{
    http::{RequestJson, xlsx_attachment},
    system::BatchIdsInput,
};

use crate::{
    api::{
        ApiState, TokenPair,
        dto::{
            AuthSessionResponse, AvatarResponse, ChangePasswordPayload, ListUsersQuery, MeResponse, ProfilePayload, ProfileResponse, RefreshTokenPayload,
            ResetPasswordPayload, SignInPayload, SignUpPayload, StatusPayload, TokenPairResponse, UserExportQuery, UserFormOptionsResponse, UserImportResponse,
            UserPayload, UserResponse, UserRolesPayload, UsersPageResponse,
        },
        error::ApiError,
        import_export::{export_query_page, export_users_xlsx, import_template_xlsx, parse_import_rows},
    },
    application::{AppError, AvatarFile, UserImportInput},
    domain::{NewUser, User, UserId},
};

type ApiResult<T> = Result<T, ApiError>;
type ApiJson<T> = Json<T>;

mod support;

use self::support::{
    ExportUsersInput, all_export_users, avatar_file, bearer_token, new_admin_user, new_sign_up_user, ok, reject_disabled_registration, user_import_form,
    verify_account_captcha,
};

type AccountPasswordRequest = (State<ApiState>, Extension<CurrentUser>, RequestJson<ChangePasswordPayload>);
type ExportUsersRequest = (State<ApiState>, Extension<CurrentUser>, Extension<DataScopeFilter>, Query<UserExportQuery>);
type ResetPasswordRequest = (State<ApiState>, Path<String>, RequestJson<ResetPasswordPayload>);
type ListUsersRequest = (State<ApiState>, Extension<CurrentUser>, Extension<DataScopeFilter>, Query<ListUsersQuery>);

pub async fn sign_up(State(state): State<ApiState>, RequestJson(payload): RequestJson<SignUpPayload>) -> ApiResult<ApiJson<AuthSessionResponse>> {
    reject_disabled_registration(&state).await?;
    verify_account_captcha(&state, payload.captcha_token.as_deref()).await?;
    let user = state.users.sign_up(new_sign_up_user(payload)).await?;
    let tokens = state.tokens.issue_pair(user.id.clone()).await?;
    Ok(ok(AuthSessionResponse::new(user.into(), tokens)))
}

pub async fn sign_in(State(state): State<ApiState>, RequestJson(payload): RequestJson<SignInPayload>) -> ApiResult<ApiJson<AuthSessionResponse>> {
    verify_account_captcha(&state, payload.captcha_token.as_deref()).await?;
    let user = state.users.sign_in(payload.into()).await?;
    let tokens = state.tokens.issue_pair(user.id.clone()).await?;
    Ok(ok(AuthSessionResponse::new(user.into(), tokens)))
}

pub async fn refresh(State(state): State<ApiState>, RequestJson(payload): RequestJson<RefreshTokenPayload>) -> ApiResult<ApiJson<TokenPairResponse>> {
    let (user_id, tokens) = state.tokens.refresh(&payload.refresh_token).await?;
    state.users.authenticated_user(user_id).await?;
    Ok(ok(tokens.into()))
}

pub async fn me(State(state): State<ApiState>, headers: HeaderMap) -> ApiResult<ApiJson<MeResponse>> {
    let access_token = bearer_token(&headers)?;
    let user_id = state.tokens.validate_access(access_token)?;
    let user = state.users.authenticated_user(user_id).await?;
    Ok(ok(MeResponse { user: user.into() }))
}

pub async fn account_profile(State(state): State<ApiState>, Extension(current_user): Extension<CurrentUser>) -> ApiResult<ApiJson<ProfileResponse>> {
    let profile = state.users.profile(UserId(current_user.id)).await?;
    Ok(ok(profile.into()))
}

pub async fn update_account_profile(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    RequestJson(payload): RequestJson<ProfilePayload>,
) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.update_profile(UserId(current_user.id), payload.into()).await?;
    Ok(ok(user.into()))
}

pub async fn change_account_password((State(state), Extension(current_user), RequestJson(payload)): AccountPasswordRequest) -> ApiResult<ApiJson<()>> {
    state
        .users
        .change_password(UserId(current_user.id), payload.old_password, payload.new_password)
        .await?;
    Ok(ok(()))
}

pub async fn upload_account_avatar(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    multipart: Multipart,
) -> ApiResult<ApiJson<AvatarResponse>> {
    let avatar = avatar_file(multipart).await?;
    let max_bytes = state.avatar_config.avatar_config().await?.max_bytes;
    let img_url = state.avatar_storage.store_avatar(avatar, max_bytes).await?;
    let user = state.users.update_avatar(UserId(current_user.id), img_url.clone()).await?;
    Ok(ok(AvatarResponse { img_url, user: user.into() }))
}

#[require_perms("system:user:export")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn export_users(request: ExportUsersRequest) -> ApiResult<Response> {
    let (State(state), Extension(current_user), Extension(data_scope), Query(query)) = request;
    let users = all_export_users(ExportUsersInput {
        state: &state,
        current_user: &current_user,
        data_scope,
        query: &query,
    })
    .await?;
    let bytes = export_users_xlsx(&users)?;
    Ok(xlsx_attachment("users.xlsx", bytes))
}

#[require_perms("system:user:import")]
pub async fn import_users(State(state): State<ApiState>, multipart: Multipart) -> ApiResult<ApiJson<UserImportResponse>> {
    let form = user_import_form(multipart).await?;
    let default_password = state.config.config_by_key(INIT_PASSWORD_KEY).await?;
    let rows = parse_import_rows(&form.file)?;
    let report = state
        .users
        .import_users(UserImportInput {
            rows,
            update_support: form.update_support,
            default_password,
        })
        .await?;
    Ok(ok(report.into()))
}

#[require_perms("system:user:import")]
pub async fn user_import_template() -> ApiResult<Response> {
    Ok(xlsx_attachment("user_template.xlsx", import_template_xlsx()?))
}

#[require_perms("system:user:add")]
pub async fn create_user(State(state): State<ApiState>, RequestJson(payload): RequestJson<UserPayload>) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.create_user(new_admin_user(&state, payload).await?).await?;
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
pub async fn reset_user_password((State(state), Path(id), RequestJson(payload)): ResetPasswordRequest) -> ApiResult<ApiJson<()>> {
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
pub async fn list_users(request: ListUsersRequest) -> ApiResult<ApiJson<UsersPageResponse>> {
    let (State(state), Extension(current_user), Extension(data_scope), Query(query)) = request;
    let page = if current_user.admin {
        state.users.list_users(query.into()).await?
    } else {
        state.users.list_users_scoped(query.into(), data_scope).await?
    };
    Ok(ok(page.into()))
}
