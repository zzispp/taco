use axum::{
    Extension, Json,
    extract::{Multipart, State},
    http::{HeaderMap, header::AUTHORIZATION},
    response::Response,
};
use constants::system_config::REGISTER_USER_KEY;
use rbac::{api::CurrentUser, domain::DataScopeFilter};
use rbac_macros::require_perms;
use types::http::{RequestJson, RequestQuery, current_locale, xlsx_attachment, xlsx_file_attachment};

use crate::{
    api::{
        ApiState, TokenIssueInput, TokenPair,
        dto::{
            AvatarResponse, ChangePasswordPayload, ListUsersQuery, MeResponse, OnlineSessionsQuery, OnlineSessionsResponse, ProfilePayload, ProfileResponse,
            TokenPairResponse, UserExportQuery, UserImportResponse, UserResponse, UsersPageResponse,
        },
        error::ApiError,
        import_export::{UserXlsxExport, import_template_xlsx, parse_import_rows},
        user_list_filter::{export_user_filter, list_user_filter},
    },
    application::{AppError, AuditedPasswordChange, AvatarFile, AvatarOwner, OnlineSession, UserExportRequest, UserImportInput, normalize_avatar},
    domain::{AvatarFileId, NewUser, User, UserId},
};

type ApiResult<T> = Result<T, ApiError>;
type ApiJson<T> = Json<T>;
type CookieApiJson<T> = (HeaderMap, Json<T>);
type UpdateAccountProfileRequest = (
    State<ApiState>,
    Extension<CurrentUser>,
    Option<Extension<audit_contract::OperationAuditContext>>,
    RequestJson<ProfilePayload>,
);
type UploadAccountAvatarRequest = (
    State<ApiState>,
    Extension<CurrentUser>,
    Option<Extension<audit_contract::OperationAuditContext>>,
    Multipart,
);

mod admin;
mod auth;
mod auth_events;
pub(crate) mod auth_json;
mod online;
mod support;

pub use admin::{
    create_user, delete_user, delete_users, get_user, replace_user, replace_user_roles, reset_user_password, update_user_status, user_dept_tree,
    user_form_options, user_roles,
};
pub use auth::{logout, refresh, sign_in, sign_up};
pub use auth_events::{AuthEventPublisher, AuthenticationEventContext};
pub use online::{force_logout_online_session, list_online_sessions};

use self::support::{
    AccountPasswordRequest, ExportUsersRequest, ListUsersRequest, avatar_file, bearer_token, ok, successful_operation_audit, user_import_form,
};

pub async fn me(State(state): State<ApiState>, headers: HeaderMap) -> ApiResult<ApiJson<MeResponse>> {
    let access_token = bearer_token(&headers)?;
    let user_id = state.tokens.validate_access(access_token).await?;
    let user = state.users.authenticated_user(user_id).await?;
    Ok(ok(MeResponse { user: user.into() }))
}

pub async fn account_profile(State(state): State<ApiState>, Extension(current_user): Extension<CurrentUser>) -> ApiResult<ApiJson<ProfileResponse>> {
    let profile = state.users.profile(UserId(current_user.id)).await?;
    Ok(ok(profile.into()))
}

pub async fn update_account_profile(
    (State(state), Extension(current_user), audit_context, RequestJson(payload)): UpdateAccountProfileRequest,
) -> ApiResult<ApiJson<UserResponse>> {
    let audit = successful_operation_audit(audit_context)?;
    let user = state
        .users
        .update_profile_with_audit(UserId(current_user.id), payload.into(), audit.record())
        .await?;
    audit.mark_persisted();
    Ok(ok(user.into()))
}

pub async fn change_account_password(
    (State(state), Extension(current_user), audit_context, RequestJson(payload)): AccountPasswordRequest,
) -> ApiResult<ApiJson<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state
        .users
        .change_password_with_audit(AuditedPasswordChange {
            user_id: UserId(current_user.id),
            old_password: payload.old_password,
            new_password: payload.new_password,
            audit: audit.record(),
        })
        .await?;
    audit.mark_persisted();
    Ok(ok(()))
}

pub async fn upload_account_avatar(
    (State(state), Extension(current_user), audit_context, multipart): UploadAccountAvatarRequest,
) -> ApiResult<ApiJson<AvatarResponse>> {
    let owner = AvatarOwner {
        user_id: current_user.id.clone(),
        department_id: current_user.dept_id.clone(),
    };
    let avatar = avatar_file(multipart).await?;
    let max_bytes = state.avatar_config.avatar_config().await?.max_bytes;
    let avatar = normalize_avatar(avatar, max_bytes).await?;
    let file_id = state.avatar_storage.store_avatar(owner.clone(), avatar).await?;
    match bind_avatar(&state, owner.clone(), file_id.clone(), audit_context).await {
        Ok(response) => Ok(ok(response)),
        Err(error) => {
            state.avatar_storage.trash_avatar(owner, file_id).await?;
            Err(error)
        }
    }
}

async fn bind_avatar(
    state: &ApiState,
    owner: AvatarOwner,
    next: AvatarFileId,
    audit_context: Option<Extension<audit_contract::OperationAuditContext>>,
) -> ApiResult<AvatarResponse> {
    let audit = successful_operation_audit(audit_context)?;
    let user = state
        .users
        .update_avatar_with_audit(UserId(owner.user_id.clone()), next.clone(), audit.record())
        .await?;
    audit.mark_persisted();
    let user = crate::api::UserResponse::from(user);
    let img_url = user
        .avatar
        .clone()
        .ok_or_else(|| AppError::Infrastructure("persisted avatar has no public projection".into()))?;
    Ok(AvatarResponse { img_url, user })
}

#[require_perms("system:user:export")]
pub async fn export_users(request: ExportUsersRequest) -> ApiResult<Response> {
    let (State(state), Extension(data_scope), RequestQuery(query)) = request;
    let filter = export_user_filter(&query)?;
    let batch_size = state.export_config.export_batch_config().await?.page_size;
    let mut export = UserXlsxExport::new(current_locale())?;
    state
        .users
        .export_users(
            UserExportRequest {
                filter,
                scope: data_scope,
                batch_size,
            },
            &mut export,
        )
        .await?;
    Ok(xlsx_file_attachment("users.xlsx", export.finish()?))
}

#[require_perms("system:user:import")]
pub async fn import_users(
    State(state): State<ApiState>,
    audit_context: Option<Extension<audit_contract::OperationAuditContext>>,
    multipart: Multipart,
) -> ApiResult<ApiJson<UserImportResponse>> {
    let form = user_import_form(multipart).await?;
    let rows = parse_import_rows(&form.file, current_locale())?;
    let audit = successful_operation_audit(audit_context)?;
    let report = state
        .users
        .import_users_with_audit(
            UserImportInput {
                rows,
                update_support: form.update_support,
            },
            audit.record(),
        )
        .await?;
    audit.mark_persisted();
    Ok(ok(report.into()))
}

#[require_perms("system:user:import")]
pub async fn user_import_template() -> ApiResult<Response> {
    Ok(xlsx_attachment("user_template.xlsx", import_template_xlsx(current_locale())?))
}

#[require_perms("system:user:list")]
pub async fn list_users(request: ListUsersRequest) -> ApiResult<ApiJson<UsersPageResponse>> {
    let (State(state), Extension(data_scope), RequestQuery(query)) = request;
    let filter = list_user_filter(query)?;
    let page = state.users.list_users_scoped(filter, data_scope).await?;
    Ok(ok(page.map(UserResponse::from)))
}
