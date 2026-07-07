use super::*;

pub(super) struct ExportUsersInput<'a> {
    pub(super) state: &'a ApiState,
    pub(super) current_user: &'a CurrentUser,
    pub(super) data_scope: DataScopeFilter,
    pub(super) query: &'a UserExportQuery,
}

pub(super) async fn all_export_users(input: ExportUsersInput<'_>) -> ApiResult<Vec<User>> {
    let export_page_size = input.state.export_config.export_batch_config().await?.page_size;
    let mut page = 1;
    let mut users = Vec::new();
    loop {
        let filter = export_query_page(input.query, page, export_page_size);
        let current = if input.current_user.admin {
            input.state.users.list_users(filter).await?
        } else {
            input.state.users.list_users_scoped(filter, input.data_scope.clone()).await?
        };
        let is_last = current.items.is_empty() || users.len() + current.items.len() >= current.total as usize;
        users.extend(current.items);
        if is_last {
            return Ok(users);
        }
        page += 1;
    }
}

pub(super) struct UserImportForm {
    pub(super) file: Vec<u8>,
    pub(super) update_support: bool,
}

pub(super) async fn avatar_file(mut multipart: Multipart) -> ApiResult<AvatarFile> {
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

pub(super) async fn user_import_form(mut multipart: Multipart) -> ApiResult<UserImportForm> {
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

pub(super) fn parse_bool(bytes: &[u8]) -> ApiResult<bool> {
    let value = std::str::from_utf8(bytes).map_err(|_| ApiError(AppError::InvalidInput(localized("errors.user.invalid_form_value"))))?;
    Ok(matches!(value.trim().to_ascii_lowercase().as_str(), "true" | "1" | "yes" | "y"))
}

pub(super) fn multipart_error(error: axum::extract::multipart::MultipartError) -> ApiError {
    let _ = error;
    ApiError(AppError::InvalidInput(localized("errors.user.invalid_multipart")))
}

pub(super) fn ok<T>(data: T) -> ApiJson<T> {
    Json(data)
}

pub(super) async fn verify_account_captcha(state: &ApiState, token: Option<&str>) -> ApiResult<()> {
    state.account_verifier.verify_account(token).await.map_err(ApiError)
}

pub(super) async fn reject_disabled_registration(state: &ApiState) -> ApiResult<()> {
    if !state.config.config_by_key(REGISTER_USER_KEY).await?.trim().eq_ignore_ascii_case("true") {
        return Err(ApiError(crate::application::AppError::Forbidden(localized(
            "errors.user.registration_disabled",
        ))));
    }
    Ok(())
}

pub(super) fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
}

pub(super) async fn new_admin_user(state: &ApiState, payload: UserPayload) -> ApiResult<NewUser> {
    let mut user: NewUser = payload.into();
    if user.password.trim().is_empty() {
        user.password = state.config.config_by_key(INIT_PASSWORD_KEY).await?;
    }
    Ok(user)
}

pub(super) fn new_sign_up_user(payload: SignUpPayload) -> NewUser {
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

pub(super) fn bearer_token(headers: &HeaderMap) -> ApiResult<&str> {
    let value = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(ApiError(crate::application::AppError::Unauthorized))?;

    value.strip_prefix("Bearer ").ok_or(ApiError(crate::application::AppError::Unauthorized))
}
