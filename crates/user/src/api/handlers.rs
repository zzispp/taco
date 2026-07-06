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

pub async fn change_account_password(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    RequestJson(payload): RequestJson<ChangePasswordPayload>,
) -> ApiResult<ApiJson<()>> {
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
pub async fn export_users(
    State(state): State<ApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(data_scope): Extension<DataScopeFilter>,
    Query(query): Query<UserExportQuery>,
) -> ApiResult<Response> {
    let users = all_export_users(&state, &current_user, data_scope, &query).await?;
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

async fn all_export_users(state: &ApiState, current_user: &CurrentUser, data_scope: DataScopeFilter, query: &UserExportQuery) -> ApiResult<Vec<User>> {
    let export_page_size = state.export_config.export_batch_config().await?.page_size;
    let mut page = 1;
    let mut users = Vec::new();
    loop {
        let filter = export_query_page(query, page, export_page_size);
        let current = if current_user.admin {
            state.users.list_users(filter).await?
        } else {
            state.users.list_users_scoped(filter, data_scope.clone()).await?
        };
        let is_last = current.items.is_empty() || users.len() + current.items.len() >= current.total as usize;
        users.extend(current.items);
        if is_last {
            return Ok(users);
        }
        page += 1;
    }
}

struct UserImportForm {
    file: Vec<u8>,
    update_support: bool,
}

async fn avatar_file(mut multipart: Multipart) -> ApiResult<AvatarFile> {
    while let Some(field) = multipart.next_field().await.map_err(multipart_error)? {
        if field.name() != Some("avatarfile") {
            continue;
        }
        let filename = field.file_name().map(str::to_owned);
        let content_type = field.content_type().map(str::to_owned);
        let bytes = field.bytes().await.map_err(multipart_error)?;
        return Ok(AvatarFile {
            filename,
            content_type,
            bytes: bytes.to_vec(),
        });
    }
    Err(ApiError(AppError::InvalidInput(localized("errors.user.avatarfile_required"))))
}

async fn user_import_form(mut multipart: Multipart) -> ApiResult<UserImportForm> {
    let mut file = None;
    let mut update_support = false;
    while let Some(field) = multipart.next_field().await.map_err(multipart_error)? {
        let name = field.name().unwrap_or_default().to_owned();
        let bytes = field.bytes().await.map_err(multipart_error)?;
        match name.as_str() {
            "file" => file = Some(bytes.to_vec()),
            "update_support" | "updateSupport" => update_support = parse_bool(&bytes)?,
            _ => {}
        }
    }
    Ok(UserImportForm {
        file: file.ok_or_else(|| ApiError(AppError::InvalidInput(localized("errors.user.file_required"))))?,
        update_support,
    })
}

fn parse_bool(bytes: &[u8]) -> ApiResult<bool> {
    let value = std::str::from_utf8(bytes).map_err(|_| ApiError(AppError::InvalidInput(localized("errors.user.invalid_form_value"))))?;
    Ok(matches!(value.trim().to_ascii_lowercase().as_str(), "true" | "1" | "yes" | "y"))
}

fn multipart_error(error: axum::extract::multipart::MultipartError) -> ApiError {
    let _ = error;
    ApiError(AppError::InvalidInput(localized("errors.user.invalid_multipart")))
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(data)
}

async fn verify_account_captcha(state: &ApiState, token: Option<&str>) -> ApiResult<()> {
    state.account_verifier.verify_account(token).await.map_err(ApiError)
}

async fn reject_disabled_registration(state: &ApiState) -> ApiResult<()> {
    if !state.config.config_by_key(REGISTER_USER_KEY).await?.trim().eq_ignore_ascii_case("true") {
        return Err(ApiError(crate::application::AppError::Forbidden(localized(
            "errors.user.registration_disabled",
        ))));
    }
    Ok(())
}

fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
}

async fn new_admin_user(state: &ApiState, payload: UserPayload) -> ApiResult<NewUser> {
    let mut user: NewUser = payload.into();
    if user.password.trim().is_empty() {
        user.password = state.config.config_by_key(INIT_PASSWORD_KEY).await?;
    }
    Ok(user)
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
