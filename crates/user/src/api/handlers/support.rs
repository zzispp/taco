use super::*;
use crate::{
    api::dto::{AuthSessionResponse, SignUpPayload},
    application::AppResult,
};
use audit_contract::{AuditOutboxRecord, OperationAuditContext};

const MISSING_OPERATION_AUDIT_ACTOR: &str = "authenticated operation audit actor is missing";
const MISSING_OPERATION_AUDIT_CONTEXT: &str = "operation audit context is missing";

pub(super) type AccountPasswordRequest = (
    State<ApiState>,
    Extension<CurrentUser>,
    Option<Extension<OperationAuditContext>>,
    RequestJson<ChangePasswordPayload>,
);
pub(super) type ExportUsersRequest = (
    State<ApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    RequestQuery<UserExportQuery>,
);
pub(super) type ListUsersRequest = (
    State<ApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    RequestQuery<ListUsersQuery>,
);
pub(super) struct SuccessfulOperationAudit {
    context: OperationAuditContext,
    record: AuditOutboxRecord,
}

impl SuccessfulOperationAudit {
    pub(super) fn record(&self) -> AuditOutboxRecord {
        self.record.clone()
    }

    pub(super) fn mark_persisted(&self) {
        self.context.mark_persisted();
    }
}

pub(super) fn successful_operation_audit(context: Option<Extension<OperationAuditContext>>) -> ApiResult<SuccessfulOperationAudit> {
    let Extension(context) = context.ok_or_else(|| ApiError(AppError::Infrastructure(MISSING_OPERATION_AUDIT_CONTEXT.into())))?;
    let record = context
        .success_record()
        .map_err(|error| ApiError(AppError::Infrastructure(error.to_string())))?
        .ok_or_else(|| ApiError(AppError::Infrastructure(MISSING_OPERATION_AUDIT_ACTOR.into())))?;
    Ok(SuccessfulOperationAudit { context, record })
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

pub(super) async fn issue_tokens_for_user(state: &ApiState, client: &client_info::ClientInfo, user: &User) -> AppResult<TokenPair> {
    let ipaddr = client.ipaddr();
    let location = state.ip_location_resolver.resolve_ip_location(&ipaddr).await.map_err(client_info_error)?;
    let login_location = login_location(location, current_locale());
    let profile = state.users.profile(user.id.clone()).await?;
    state
        .tokens
        .issue_pair(TokenIssueInput {
            user_id: user.id.clone(),
            dept_name: profile.dept_name,
            user_name: user.username.clone(),
            ipaddr,
            login_location,
            browser: client.browser.clone(),
            os: client.os.clone(),
        })
        .await
}

fn login_location(location: client_info::IpLocation, locale: types::http::Locale) -> String {
    match location {
        client_info::IpLocation::Resolved(value) => value,
        client_info::IpLocation::Internal => types::http::translate_message(locale, "messages.client_info.location.internal"),
        client_info::IpLocation::Unknown => types::http::translate_message(locale, "messages.client_info.location.unknown"),
    }
}

fn client_info_error(error: client_info::ClientInfoError) -> AppError {
    taco_tracing::error_with_fields!("client information resolution failed", &error, component = "ip_location");
    AppError::Infrastructure(error.to_string())
}

pub(super) async fn verify_account_captcha(state: &ApiState, token: Option<&str>) -> AppResult<()> {
    state.account_verifier.verify_account(token).await
}

pub(super) async fn reject_disabled_registration(state: &ApiState) -> AppResult<()> {
    if !state.config.config_by_key(REGISTER_USER_KEY).await?.trim().eq_ignore_ascii_case("true") {
        return Err(AppError::Forbidden(localized("errors.user.registration_disabled")));
    }
    Ok(())
}

pub(super) fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
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
        }
    }
}

impl From<TokenPair> for TokenPairResponse {
    fn from(value: TokenPair) -> Self {
        Self {
            access_token: value.access_token,
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

#[cfg(test)]
mod location_tests {
    use client_info::IpLocation;
    use types::http::Locale;

    use super::login_location;

    #[test]
    fn online_session_location_is_rendered_at_the_user_api_boundary() {
        assert_eq!(login_location(IpLocation::Internal, Locale::ZhCn), "内网IP");
        assert_eq!(login_location(IpLocation::Internal, Locale::En), "Intranet IP");
        assert_eq!(login_location(IpLocation::Internal, Locale::ZhTw), "內網IP");
        assert_eq!(login_location(IpLocation::Unknown, Locale::En), "Unknown");
        assert_eq!(login_location(IpLocation::Resolved("Provider Text".into()), Locale::ZhCn), "Provider Text");
    }
}

#[cfg(test)]
mod operation_audit_tests {
    use super::*;

    #[test]
    fn successful_operation_audit_rejects_a_missing_request_context() {
        assert!(matches!(
            successful_operation_audit(None),
            Err(ApiError(AppError::Infrastructure(message))) if message == MISSING_OPERATION_AUDIT_CONTEXT
        ));
    }
}
