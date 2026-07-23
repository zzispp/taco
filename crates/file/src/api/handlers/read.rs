use audit_contract::OperationAuditContext;
use axum::{
    Extension, Json,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use futures_util::StreamExt;
use kernel::pagination::CursorPage;
use rbac::{api::CurrentUser, domain::DataScopeFilter};
use rbac_macros::require_perms;
use types::http::RequestQuery;

use crate::FileError;
use crate::api::{
    FileApiError, FileApiState,
    dto::{FileListParams, FileSpaceParams, file_scope, parse_directory_id, parse_file_id, parse_read_range, parse_space_id},
};
use crate::application::{DirectoryTrailEntry, FileContent, FileEntryView, FileOverviewView, FileReadRequest, FileSpaceView, ProviderSummary};

type ApiResult<T> = Result<Json<T>, FileApiError>;
type ScopedState = (State<FileApiState>, Extension<CurrentUser>, Extension<DataScopeFilter>);

#[require_perms("file:asset:list")]
pub async fn list_files(
    (State(state), Extension(user), Extension(scope)): ScopedState,
    RequestQuery(params): RequestQuery<FileListParams>,
) -> ApiResult<CursorPage<FileEntryView>> {
    let (query, page) = params.into_query()?;
    Ok(Json(state.files.list_entries(file_scope(&user, &scope), query, page).await?))
}

#[require_perms("file:asset:list")]
pub async fn file_directory_trail(
    (State(state), Extension(user), Extension(scope)): ScopedState,
    Path(id): Path<String>,
) -> ApiResult<Vec<DirectoryTrailEntry>> {
    Ok(Json(state.files.directory_trail(file_scope(&user, &scope), parse_directory_id(id)?).await?))
}

#[require_perms("file:asset:query")]
pub async fn file_overview(
    (State(state), Extension(user), Extension(scope)): ScopedState,
    RequestQuery(params): RequestQuery<OverviewParams>,
) -> ApiResult<FileOverviewView> {
    let space_id = params.space_id.map(parse_space_id).transpose()?;
    Ok(Json(state.files.overview(file_scope(&user, &scope), space_id).await?))
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OverviewParams {
    pub space_id: Option<String>,
}

#[require_perms("file:space:list")]
pub async fn list_file_spaces(
    (State(state), Extension(user), Extension(scope)): ScopedState,
    RequestQuery(params): RequestQuery<FileSpaceParams>,
) -> ApiResult<CursorPage<FileSpaceView>> {
    let (query, page) = params.into_query()?;
    Ok(Json(state.files.list_spaces(file_scope(&user, &scope), query, page).await?))
}

#[require_perms("file:asset:query")]
pub async fn get_file((State(state), Extension(user), Extension(scope)): ScopedState, Path(id): Path<String>) -> ApiResult<FileEntryView> {
    Ok(Json(state.files.get_entry(file_scope(&user, &scope), parse_file_id(id)?).await?))
}

#[require_perms("file:provider:query")]
pub async fn list_file_providers(State(state): State<FileApiState>) -> ApiResult<Vec<ProviderSummary>> {
    Ok(Json(state.files.provider_summaries().await?))
}

#[require_perms("file:asset:download")]
pub async fn download_file(
    (State(state), Extension(user), Extension(scope)): ScopedState,
    audit: Option<Extension<OperationAuditContext>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, FileApiError> {
    let request = FileReadRequest {
        id: parse_file_id(id)?,
        range: parse_read_range(&headers)?,
    };
    let content = state.files.content(file_scope(&user, &scope), request).await?;
    let response = content_response(content, ContentDisposition::Attachment)?;
    state.record_operation(audit).await?;
    Ok(response)
}

#[require_perms("file:asset:query")]
pub async fn preview_file(scoped: ScopedState, Path(id): Path<String>, headers: HeaderMap) -> Result<Response, FileApiError> {
    let (State(state), Extension(user), Extension(scope)) = scoped;
    let content = state
        .files
        .preview(
            file_scope(&user, &scope),
            FileReadRequest {
                id: parse_file_id(id)?,
                range: parse_read_range(&headers)?,
            },
        )
        .await?;
    content_response(content, ContentDisposition::Inline)
}

#[require_perms("file:asset:query")]
pub async fn thumbnail_file(scoped: ScopedState, Path(id): Path<String>, headers: HeaderMap) -> Result<Response, FileApiError> {
    if headers.get(header::RANGE).is_some() {
        return Err(FileError::RangeNotSatisfiable.into());
    }
    let (State(state), Extension(user), Extension(scope)) = scoped;
    let content = state.files.thumbnail(file_scope(&user, &scope), parse_file_id(id)?).await?;
    content_response(content, ContentDisposition::Inline)
}

#[derive(Clone, Copy)]
enum ContentDisposition {
    Attachment,
    Inline,
}

fn content_response(content: FileContent, disposition: ContentDisposition) -> Result<Response, FileApiError> {
    let metadata = content.metadata;
    let stream = content.body.map(|item| item.map_err(|error| std::io::Error::other(error.to_string())));
    let mut response = Body::from_stream(stream).into_response();
    *response.status_mut() = if metadata.range.is_some() {
        StatusCode::PARTIAL_CONTENT
    } else {
        StatusCode::OK
    };
    let headers = response.headers_mut();
    if metadata.accept_ranges {
        headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    }
    headers.insert(header::CONTENT_LENGTH, header_value(metadata.response_size())?);
    headers.insert(header::CONTENT_TYPE, header_value(&metadata.content_type)?);
    headers.insert(header::CONTENT_DISPOSITION, disposition_header(&metadata.name, disposition)?);
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    if metadata.truncated {
        headers.insert(header::HeaderName::from_static("x-preview-truncated"), HeaderValue::from_static("true"));
    }
    if let Some(range) = metadata.range {
        let value = format!("bytes {}-{}/{}", range.start(), range.end_exclusive() - 1, metadata.size.bytes());
        headers.insert(header::CONTENT_RANGE, header_value(&value)?);
    }
    Ok(response)
}

fn header_value(value: impl ToString) -> Result<HeaderValue, FileApiError> {
    HeaderValue::from_str(&value.to_string()).map_err(|_| crate::FileError::InvalidInput(crate::error::keys::RESPONSE_HEADER_INVALID).into())
}

fn disposition_header(name: &str, disposition: ContentDisposition) -> Result<HeaderValue, FileApiError> {
    let safe = name
        .chars()
        .map(|value| {
            if value.is_ascii_alphanumeric() || ".-_ ".contains(value) {
                value
            } else {
                '_'
            }
        })
        .collect::<String>();
    let value = match disposition {
        ContentDisposition::Attachment => "attachment",
        ContentDisposition::Inline => "inline",
    };
    header_value(format!("{value}; filename=\"{safe}\""))
}

#[cfg(test)]
mod tests {
    use axum::http::{StatusCode, header};
    use bytes::Bytes;

    use crate::application::{FileContent, FileContentMetadata, as_object_stream};
    use crate::domain::ByteSize;

    use super::{ContentDisposition, content_response};

    #[test]
    fn download_response_forces_attachment_for_previewable_content() {
        let response = content_response(file_content("image/png"), ContentDisposition::Attachment).unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_DISPOSITION], "attachment; filename=\"photo.png\"");
        assert_eq!(response.headers()[header::CONTENT_TYPE], "image/png");
        assert_eq!(response.headers()[header::CONTENT_LENGTH], "4");
        assert_eq!(response.headers()[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    }

    #[test]
    fn preview_response_is_inline_and_disables_content_sniffing() {
        let response = content_response(file_content("text/plain"), ContentDisposition::Inline).unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_DISPOSITION], "inline; filename=\"photo.png\"");
        assert_eq!(response.headers()[header::CONTENT_TYPE], "text/plain");
        assert_eq!(response.headers()[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    }

    #[test]
    fn truncated_preview_reports_the_actual_bounded_length() {
        let mut content = file_content("text/plain");
        content.metadata.size = ByteSize::from_bytes(crate::application::TEXT_PREVIEW_MAX_BYTES + 1);
        content.metadata.truncated = true;
        let response = content_response(content, ContentDisposition::Inline).unwrap();

        assert_eq!(response.headers()["x-preview-truncated"], "true");
        assert_eq!(
            response.headers()[header::CONTENT_LENGTH],
            crate::application::TEXT_PREVIEW_MAX_BYTES.to_string()
        );
    }

    fn file_content(content_type: &str) -> FileContent {
        FileContent {
            metadata: FileContentMetadata {
                name: "photo.png".into(),
                content_type: content_type.into(),
                size: ByteSize::from_bytes(4),
                range: None,
                truncated: false,
                accept_ranges: true,
            },
            body: as_object_stream(Bytes::from_static(b"data")),
        }
    }
}
