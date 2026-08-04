use audit_contract::OperationAuditContext;
use axum::{Extension, Json, body::Body, extract::State, http::HeaderMap};
use futures_util::StreamExt;
use rbac::{api::CurrentUser, domain::DataScopeFilter};
use rbac_macros::{require_any_perms, require_perms};

use crate::api::{
    FileApiError, FileApiState,
    dto::{BeginUploadPayload, CompleteUploadPayload, PartCommandInput, file_scope, parse_upload_id, part_command},
};
use crate::application::{BeginUploadResult, FileEntryView, PartReceiptResponse, UploadSessionResponse};
use crate::error::keys;
use crate::{FileError, FileResult};

const IDEMPOTENCY_HEADER: &str = "idempotency-key";
type CreateUploadSessionRequest = (
    State<FileApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    HeaderMap,
    axum::Json<BeginUploadPayload>,
);
type GetUploadSessionRequest = (
    State<FileApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    axum::extract::Path<String>,
);
type WriteUploadPartRequest = (
    State<FileApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    HeaderMap,
    axum::extract::Path<(String, u32)>,
    Body,
);
type CompleteUploadSessionRequest = (
    State<FileApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    Option<Extension<OperationAuditContext>>,
    axum::extract::Path<String>,
    types::http::RequestJson<CompleteUploadPayload>,
);
type CancelUploadSessionRequest = (
    State<FileApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    Option<Extension<OperationAuditContext>>,
    axum::extract::Path<String>,
);

#[require_perms("file:asset:upload")]
pub async fn create_upload_session(
    (State(state), Extension(user), Extension(scope), headers, axum::Json(payload)): CreateUploadSessionRequest,
) -> Result<Json<BeginUploadResult>, FileApiError> {
    let key = idempotency_key(&headers)?.ok_or(FileError::InvalidInput(keys::IDEMPOTENCY_KEY_REQUIRED))?;
    let command = payload.into_command(&user.id, key)?;
    let response = state.files.begin_upload_session(file_scope(&user, &scope), command).await?;
    Ok(Json(response))
}

#[require_any_perms("file:asset:upload", "file:upload:manage")]
pub async fn get_upload_session(
    (State(state), Extension(user), Extension(scope), axum::extract::Path(id)): GetUploadSessionRequest,
) -> Result<Json<UploadSessionResponse>, FileApiError> {
    let response = state.files.get_upload_session(file_scope(&user, &scope), parse_upload_id(id)?).await?;
    Ok(Json(response))
}

#[require_perms("file:asset:upload")]
pub async fn write_upload_part(
    (State(state), Extension(user), Extension(scope), headers, axum::extract::Path((id, part_number)), body): WriteUploadPartRequest,
) -> Result<Json<PartReceiptResponse>, FileApiError> {
    let digest = headers
        .get("x-content-sha256")
        .and_then(|value| value.to_str().ok())
        .ok_or(FileError::InvalidInput(keys::PART_DIGEST_HEADER_REQUIRED))?;
    let stream = body
        .into_data_stream()
        .map(|chunk| chunk.map_err(|error| FileError::Infrastructure(format!("upload request body failed: {error}"))));
    let command = part_command(PartCommandInput {
        session_id: parse_upload_id(id)?,
        part_number,
        digest: digest.to_owned(),
        body: Box::pin(stream),
    })?;
    let receipt = state.files.write_upload_part(file_scope(&user, &scope), command).await?;
    Ok(Json(PartReceiptResponse {
        part_number: receipt.part_number.value(),
        size_bytes: receipt.size.bytes(),
        sha256: receipt.digest.to_hex(),
    }))
}

#[require_perms("file:asset:upload")]
pub async fn complete_upload_session(
    (State(state), Extension(user), Extension(scope), audit, axum::extract::Path(id), types::http::RequestJson(payload)): CompleteUploadSessionRequest,
) -> Result<Json<FileEntryView>, FileApiError> {
    validate_complete_payload(payload)?;
    let entry = state.files.complete_upload_session(file_scope(&user, &scope), parse_upload_id(id)?).await?;
    state.record_operation(audit).await?;
    Ok(Json(entry))
}

#[require_any_perms("file:asset:upload", "file:upload:manage")]
pub async fn cancel_upload_session(
    (State(state), Extension(user), Extension(scope), audit, axum::extract::Path(id)): CancelUploadSessionRequest,
) -> Result<Json<()>, FileApiError> {
    state.files.cancel_upload_session(file_scope(&user, &scope), parse_upload_id(id)?).await?;
    state.record_operation(audit).await?;
    Ok(Json(()))
}

fn validate_complete_payload(payload: CompleteUploadPayload) -> FileResult<()> {
    if payload.parts.is_empty() {
        return Err(FileError::UploadIncomplete);
    }
    for part in payload.parts {
        crate::domain::PartNumber::new(part.part_number)?;
        if part.size_bytes == 0 {
            return Err(FileError::InvalidPart);
        }
        crate::domain::ContentDigest::from_hex(&part.sha256)?;
    }
    Ok(())
}

fn idempotency_key(headers: &HeaderMap) -> FileResult<Option<String>> {
    headers
        .get(IDEMPOTENCY_HEADER)
        .map(|value| {
            value
                .to_str()
                .map(str::to_owned)
                .map_err(|_| FileError::InvalidInput(keys::IDEMPOTENCY_KEY_INVALID_UTF8))
        })
        .transpose()
}
