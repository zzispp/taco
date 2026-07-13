use axum::{
    Extension, Json,
    extract::{Multipart, Path, State},
    http::{HeaderMap, header::AUTHORIZATION},
    response::Response,
};
use constants::system_config::{INIT_PASSWORD_KEY, REGISTER_USER_KEY};
use kernel::{error::LocalizedError, pagination::PageRequest};
use rbac::api::CurrentUser;
use rbac_macros::{data_scope, require_perms};
use types::{
    http::{RequestJson, RequestQuery, current_locale, xlsx_attachment},
    rbac::DataScopeFilter,
    system::BatchIdsInput,
};

use crate::{
    api::{
        ApiState, TokenIssueInput, TokenPair,
        dto::{
            AuthSessionResponse, AvatarResponse, ChangePasswordPayload, ListUsersQuery, MeResponse, OnlineSessionsQuery, OnlineSessionsResponse,
            ProfilePayload, ProfileResponse, RefreshTokenPayload, ResetPasswordPayload, SignInPayload, SignUpPayload, StatusPayload, TokenPairResponse,
            UserExportQuery, UserFormOptionsResponse, UserImportResponse, UserPayload, UserResponse, UserRolesPayload, UsersPageResponse,
        },
        error::ApiError,
        import_export::{export_users_xlsx, import_template_xlsx, parse_import_rows},
        user_list_filter::{export_filter_page, export_user_filter, list_user_filter},
    },
    application::{AppError, AvatarFile, OnlineSession, UserImportInput, UserListFilter},
    domain::{NewUser, User, UserId},
};

type ApiResult<T> = Result<T, ApiError>;
type ApiJson<T> = Json<T>;

mod online;
mod support;

pub use online::{force_logout_online_session, list_online_sessions};

use self::support::{
    AccountPasswordRequest, ExportUsersInput, ExportUsersRequest, ListUsersRequest, UserBatchRequest, UserJsonRequest, UserPathRequest, all_export_users,
    avatar_file, bearer_token, issue_tokens_for_user, new_admin_user, new_sign_up_user, ok, reject_disabled_registration, user_import_form,
    verify_account_captcha,
};

pub async fn sign_up(
    State(state): State<ApiState>,
    headers: HeaderMap,
    RequestJson(payload): RequestJson<SignUpPayload>,
) -> ApiResult<ApiJson<AuthSessionResponse>> {
    reject_disabled_registration(&state).await?;
    verify_account_captcha(&state, payload.captcha_token.as_deref()).await?;
    let user = state.users.sign_up(new_sign_up_user(payload)).await?;
    let tokens = issue_tokens_for_user(&state, &headers, &user).await?;
    Ok(ok(AuthSessionResponse::new(user.into(), tokens)))
}

pub async fn sign_in(
    State(state): State<ApiState>,
    headers: HeaderMap,
    RequestJson(payload): RequestJson<SignInPayload>,
) -> ApiResult<ApiJson<AuthSessionResponse>> {
    verify_account_captcha(&state, payload.captcha_token.as_deref()).await?;
    let user = state.users.sign_in(payload.into()).await?;
    let tokens = issue_tokens_for_user(&state, &headers, &user).await?;
    Ok(ok(AuthSessionResponse::new(user.into(), tokens)))
}

pub async fn refresh(State(state): State<ApiState>, RequestJson(payload): RequestJson<RefreshTokenPayload>) -> ApiResult<ApiJson<TokenPairResponse>> {
    let (user_id, tokens) = state.tokens.refresh(&payload.refresh_token).await?;
    state.users.authenticated_user(user_id).await?;
    Ok(ok(tokens.into()))
}

pub async fn me(State(state): State<ApiState>, headers: HeaderMap) -> ApiResult<ApiJson<MeResponse>> {
    let access_token = bearer_token(&headers)?;
    let user_id = state.tokens.validate_access(access_token).await?;
    let user = state.users.authenticated_user(user_id).await?;
    Ok(ok(MeResponse { user: user.into() }))
}

pub async fn logout(State(state): State<ApiState>, headers: HeaderMap) -> ApiResult<ApiJson<()>> {
    let access_token = bearer_token(&headers)?;
    state.tokens.logout_access(access_token).await?;
    Ok(ok(()))
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
    let (State(state), Extension(current_user), Extension(data_scope), RequestQuery(query)) = request;
    let filter = export_user_filter(&query)?;
    let users = all_export_users(ExportUsersInput {
        state: &state,
        current_user: &current_user,
        data_scope,
        filter,
    })
    .await?;
    let bytes = export_users_xlsx(&users, current_locale())?;
    Ok(xlsx_attachment("users.xlsx", bytes))
}

#[require_perms("system:user:import")]
pub async fn import_users(State(state): State<ApiState>, multipart: Multipart) -> ApiResult<ApiJson<UserImportResponse>> {
    let form = user_import_form(multipart).await?;
    let default_password = state.config.config_by_key(INIT_PASSWORD_KEY).await?;
    let rows = parse_import_rows(&form.file, current_locale())?;
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
    Ok(xlsx_attachment("user_template.xlsx", import_template_xlsx(current_locale())?))
}

#[require_perms("system:user:add")]
pub async fn create_user(State(state): State<ApiState>, RequestJson(payload): RequestJson<UserPayload>) -> ApiResult<ApiJson<UserResponse>> {
    let user = state.users.create_user(new_admin_user(&state, payload).await?).await?;
    Ok(ok(user.into()))
}

#[require_perms("system:user:edit")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn replace_user(request: UserJsonRequest<RequestJson<UserPayload>>) -> ApiResult<ApiJson<UserResponse>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id), RequestJson(payload)) = request;
    UserScopeGuard::new(&state, &current_user, data_scope).ensure_one(&id).await?;
    let user = state.users.replace_user(UserId(id), payload.into()).await?;
    Ok(ok(user.into()))
}

#[require_perms("system:user:remove")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn delete_user(request: UserPathRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id)) = request;
    UserScopeGuard::new(&state, &current_user, data_scope).ensure_one(&id).await?;
    state.users.delete_user(UserId(id)).await?;
    Ok(ok(()))
}

#[require_perms("system:user:remove")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn delete_users(request: UserBatchRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(current_user), Extension(data_scope), RequestJson(payload)) = request;
    let ids = user_ids(payload.ids);
    UserScopeGuard::new(&state, &current_user, data_scope).ensure_many(ids.clone()).await?;
    state.users.delete_users(ids).await?;
    Ok(ok(()))
}

#[require_perms("system:user:query")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn get_user(request: UserPathRequest) -> ApiResult<ApiJson<UserResponse>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id)) = request;
    UserScopeGuard::new(&state, &current_user, data_scope).ensure_one(&id).await?;
    let user = state.users.get_user(UserId(id)).await?;
    Ok(ok(user.into()))
}

#[require_perms("system:user:resetPwd")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn reset_user_password(request: UserJsonRequest<RequestJson<ResetPasswordPayload>>) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id), RequestJson(payload)) = request;
    UserScopeGuard::new(&state, &current_user, data_scope).ensure_one(&id).await?;
    state.users.reset_password(UserId(id), payload.password).await?;
    Ok(ok(()))
}

#[require_perms("system:user:edit")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn update_user_status(request: UserJsonRequest<RequestJson<StatusPayload>>) -> ApiResult<ApiJson<UserResponse>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id), RequestJson(payload)) = request;
    UserScopeGuard::new(&state, &current_user, data_scope).ensure_one(&id).await?;
    let user = state.users.update_status(UserId(id), payload.status).await?;
    Ok(ok(user.into()))
}

#[require_perms("system:user:query")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn user_roles(request: UserPathRequest) -> ApiResult<ApiJson<UserRolesPayload>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id)) = request;
    UserScopeGuard::new(&state, &current_user, data_scope).ensure_one(&id).await?;
    let user = state.users.get_user(UserId(id)).await?;
    Ok(ok(UserRolesPayload { role_ids: user.role_ids }))
}

#[require_perms("system:user:edit")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn replace_user_roles(request: UserJsonRequest<RequestJson<UserRolesPayload>>) -> ApiResult<ApiJson<UserResponse>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id), RequestJson(payload)) = request;
    UserScopeGuard::new(&state, &current_user, data_scope).ensure_one(&id).await?;
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
    let (State(state), Extension(current_user), Extension(data_scope), RequestQuery(query)) = request;
    let filter = list_user_filter(query)?;
    let page = if current_user.admin {
        state.users.list_users(filter).await?
    } else {
        state.users.list_users_scoped(filter, data_scope).await?
    };
    Ok(ok(page.into()))
}

struct UserScopeGuard<'a> {
    state: &'a ApiState,
    current_user: &'a CurrentUser,
    data_scope: DataScopeFilter,
}

impl<'a> UserScopeGuard<'a> {
    const fn new(state: &'a ApiState, current_user: &'a CurrentUser, data_scope: DataScopeFilter) -> Self {
        Self {
            state,
            current_user,
            data_scope,
        }
    }

    async fn ensure_one(&self, id: &str) -> ApiResult<()> {
        self.ensure_many(vec![UserId(id.into())]).await
    }

    async fn ensure_many(&self, ids: Vec<UserId>) -> ApiResult<()> {
        if self.current_user.admin {
            return Ok(());
        }
        self.state.users.ensure_user_ids_scoped(ids, self.data_scope.clone()).await.map_err(ApiError)
    }
}

fn user_ids(ids: Vec<String>) -> Vec<UserId> {
    ids.into_iter().map(UserId).collect()
}
