use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use types::http::{ApiErrorKind, ApiErrorResponse, Locale, current_locale, localized_error_response};

use crate::FileError;

#[derive(Debug)]
pub struct FileApiError(pub FileError);

impl From<FileError> for FileApiError {
    fn from(value: FileError) -> Self {
        Self(value)
    }
}

impl IntoResponse for FileApiError {
    fn into_response(self) -> Response {
        let status = status_code(&self.0);
        let body = response_body(&self.0);
        (status, Json(body)).into_response()
    }
}

fn status_code(error: &FileError) -> StatusCode {
    match error {
        FileError::NotFound | FileError::UploadNotFound => StatusCode::NOT_FOUND,
        FileError::UploadResultUnavailable => StatusCode::GONE,
        FileError::Forbidden => StatusCode::FORBIDDEN,
        FileError::NameConflict | FileError::UploadPartConflict | FileError::UploadIntentTerminal | FileError::UploadCompletionInProgress => {
            StatusCode::CONFLICT
        }
        FileError::QuotaExceeded { .. } | FileError::CapacityExceeded { .. } => StatusCode::CONFLICT,
        FileError::RangeNotSatisfiable => StatusCode::RANGE_NOT_SATISFIABLE,
        FileError::ProviderIo { .. } | FileError::ProviderUnavailable { .. } | FileError::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
        FileError::InvalidInput(_)
        | FileError::InvalidUploadTransition { .. }
        | FileError::UploadIncomplete
        | FileError::InvalidPart
        | FileError::DigestMismatch
        | FileError::SizeMismatch => StatusCode::BAD_REQUEST,
    }
}

fn response_body(error: &FileError) -> ApiErrorResponse {
    response_body_for_locale(error, current_locale())
}

fn response_body_for_locale(error: &FileError, locale: Locale) -> ApiErrorResponse {
    let mut response = localized_error_response(locale, error_kind(error), Some(&error_detail(error)));
    if let Some(code) = stable_file_error_code(error) {
        response.code = code.into();
    }
    response
}

fn stable_file_error_code(error: &FileError) -> Option<&'static str> {
    match error {
        FileError::UploadIntentTerminal => Some("upload_intent_terminal"),
        FileError::UploadResultUnavailable => Some("upload_result_unavailable"),
        FileError::UploadCompletionInProgress => Some("upload_completion_in_progress"),
        _ => None,
    }
}

fn error_kind(error: &FileError) -> ApiErrorKind {
    match error {
        FileError::NotFound | FileError::UploadNotFound | FileError::UploadResultUnavailable => ApiErrorKind::NotFound,
        FileError::Forbidden => ApiErrorKind::Forbidden,
        FileError::NameConflict
        | FileError::UploadPartConflict
        | FileError::UploadIntentTerminal
        | FileError::UploadCompletionInProgress
        | FileError::QuotaExceeded { .. }
        | FileError::CapacityExceeded { .. } => ApiErrorKind::Conflict,
        FileError::ProviderIo { .. } | FileError::ProviderUnavailable { .. } | FileError::Infrastructure(_) => ApiErrorKind::Infrastructure,
        _ => ApiErrorKind::InvalidInput,
    }
}

fn error_detail(error: &FileError) -> LocalizedError {
    let key = match error {
        FileError::InvalidInput(key) => return LocalizedError::new(key),
        FileError::NameConflict => "errors.file.name_conflict",
        FileError::NotFound => "errors.file.not_found",
        FileError::Forbidden => "errors.common.forbidden",
        FileError::UploadNotFound => "errors.file.upload_not_found",
        FileError::UploadIntentTerminal => "errors.file.upload_intent_terminal",
        FileError::UploadResultUnavailable => "errors.file.upload_result_unavailable",
        FileError::UploadCompletionInProgress => "errors.file.upload_completion_in_progress",
        FileError::InvalidUploadTransition { .. } => "errors.file.invalid_upload_transition",
        FileError::UploadIncomplete => "errors.file.upload_incomplete",
        FileError::InvalidPart => "errors.file.invalid_part",
        FileError::UploadPartConflict => "errors.file.upload_part_conflict",
        FileError::DigestMismatch => "errors.file.digest_mismatch",
        FileError::SizeMismatch => "errors.file.size_mismatch",
        FileError::RangeNotSatisfiable => "errors.file.range_not_satisfiable",
        FileError::CapacityExceeded { .. } => "errors.file.capacity_exceeded",
        FileError::QuotaExceeded { .. } => "errors.file.quota_exceeded",
        FileError::ProviderUnavailable { .. } => "errors.file.provider_unavailable",
        FileError::ProviderIo { .. } => "errors.file.provider_io",
        FileError::Infrastructure(_) => "errors.file.infrastructure",
    };
    LocalizedError::new(key)
}

#[cfg(test)]
mod tests {
    use super::{FileApiError, StatusCode, error_detail, response_body_for_locale};
    use crate::{FileError, error::keys};
    use axum::response::IntoResponse;
    use types::http::Locale;

    #[test]
    fn invalid_input_preserves_the_specific_localization_key() {
        let error = FileError::InvalidInput(keys::ENTRY_NAME_INVALID);

        assert_eq!(error_detail(&error).key(), keys::ENTRY_NAME_INVALID);
        assert_eq!(response_body_for_locale(&error, Locale::ZhCn).details.as_deref(), Some("文件名为空或过长"));
        assert_eq!(
            response_body_for_locale(&error, Locale::En).details.as_deref(),
            Some("The file name is blank or too long")
        );
        assert_eq!(response_body_for_locale(&error, Locale::ZhTw).details.as_deref(), Some("檔案名稱為空白或過長"));
    }

    #[test]
    fn invalid_input_keeps_bad_request_status() {
        let response = FileApiError(FileError::InvalidInput(keys::ENTRY_NAME_INVALID)).into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
