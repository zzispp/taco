use audit_contract::OperationAuditContext;
use axum::{
    Extension, Json,
    extract::{Path, State},
};
use rbac::{api::CurrentUser, domain::DataScopeFilter};
use rbac_macros::require_perms;
use types::http::RequestJson;

use crate::api::{
    FileApiError, FileApiState,
    dto::{BatchIdsPayload, CreateFolderPayload, SpaceQuotaPayload, UpdateFilePayload, file_scope, parse_file_id, parse_space_id},
};
use crate::application::{FileEntryView, FileSpaceView, PurgeReport};

type ApiResult<T> = Result<Json<T>, FileApiError>;
type AuditedScopedState = (
    State<FileApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    Option<Extension<OperationAuditContext>>,
);

#[require_perms("file:folder:add")]
pub async fn create_file_folder(
    (State(state), Extension(user), Extension(scope), audit): AuditedScopedState,
    RequestJson(payload): RequestJson<CreateFolderPayload>,
) -> ApiResult<FileEntryView> {
    let entry = state.files.create_folder(file_scope(&user, &scope), payload.into_command(&user)?).await?;
    state.record_operation(audit).await?;
    Ok(Json(entry))
}

#[require_perms("file:asset:edit")]
pub async fn update_file(
    (State(state), Extension(user), Extension(scope), audit): AuditedScopedState,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<UpdateFilePayload>,
) -> ApiResult<FileEntryView> {
    let command = payload.into_command(parse_file_id(id)?, &user)?;
    let entry = state.files.update_entry(file_scope(&user, &scope), command).await?;
    state.record_operation(audit).await?;
    Ok(Json(entry))
}

#[require_perms("file:asset:remove")]
pub async fn trash_file(request: AuditedScopedState, Path(id): Path<String>) -> ApiResult<()> {
    let (State(state), Extension(user), Extension(scope), audit) = request;
    state.files.trash(file_scope(&user, &scope), vec![parse_file_id(id)?]).await?;
    state.record_operation(audit).await?;
    Ok(Json(()))
}

#[require_perms("file:asset:restore")]
pub async fn restore_file(request: AuditedScopedState, Path(id): Path<String>) -> ApiResult<()> {
    let (State(state), Extension(user), Extension(scope), audit) = request;
    state.files.restore(file_scope(&user, &scope), vec![parse_file_id(id)?]).await?;
    state.record_operation(audit).await?;
    Ok(Json(()))
}

#[require_perms("file:asset:purge")]
pub async fn purge_file(request: AuditedScopedState, Path(id): Path<String>) -> ApiResult<PurgeReport> {
    let (State(state), Extension(user), Extension(scope), audit) = request;
    let report = state.files.purge(file_scope(&user, &scope), vec![parse_file_id(id)?]).await?;
    state.record_operation(audit).await?;
    Ok(Json(report))
}

#[require_perms("file:asset:remove")]
pub async fn trash_files(request: AuditedScopedState, RequestJson(payload): RequestJson<BatchIdsPayload>) -> ApiResult<()> {
    let (State(state), Extension(user), Extension(scope), audit) = request;
    state.files.trash(file_scope(&user, &scope), payload.parse()?.ids).await?;
    state.record_operation(audit).await?;
    Ok(Json(()))
}

#[require_perms("file:asset:restore")]
pub async fn restore_files(request: AuditedScopedState, RequestJson(payload): RequestJson<BatchIdsPayload>) -> ApiResult<()> {
    let (State(state), Extension(user), Extension(scope), audit) = request;
    state.files.restore(file_scope(&user, &scope), payload.parse()?.ids).await?;
    state.record_operation(audit).await?;
    Ok(Json(()))
}

#[require_perms("file:asset:purge")]
pub async fn purge_files(request: AuditedScopedState, RequestJson(payload): RequestJson<BatchIdsPayload>) -> ApiResult<PurgeReport> {
    let (State(state), Extension(user), Extension(scope), audit) = request;
    let report = state.files.purge(file_scope(&user, &scope), payload.parse()?.ids).await?;
    state.record_operation(audit).await?;
    Ok(Json(report))
}

#[require_perms("file:space:quota")]
pub async fn update_file_space(
    (State(state), Extension(user), Extension(scope), audit): AuditedScopedState,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<SpaceQuotaPayload>,
) -> ApiResult<FileSpaceView> {
    let space = state.files.update_space(file_scope(&user, &scope), parse_space_id(id)?, payload.into()).await?;
    state.record_operation(audit).await?;
    Ok(Json(space))
}
