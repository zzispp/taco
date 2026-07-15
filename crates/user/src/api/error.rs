use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use types::http::{ApiErrorKind, ApiErrorResponse, Locale, current_locale, localized_error_response, translate_localized_error};

use crate::application::AppError;

#[derive(Debug)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if matches!(&self.0, AppError::Infrastructure(_)) {
            hook_tracing::error_with_fields!("user API infrastructure failure", &self.0, component = "user");
        }
        let body = error_response(&self.0);
        (status_code(&self.0), Json(body)).into_response()
    }
}

fn status_code(error: &AppError) -> StatusCode {
    match error {
        AppError::InvalidCursor | AppError::InvalidInput(_) | AppError::ImportValidation(_) => StatusCode::BAD_REQUEST,
        AppError::Unauthorized => StatusCode::UNAUTHORIZED,
        AppError::AccountDisabled | AppError::AccountLocked { .. } => StatusCode::FORBIDDEN,
        AppError::Forbidden(_) => StatusCode::FORBIDDEN,
        AppError::Conflict(_) => StatusCode::CONFLICT,
        AppError::NotFound => StatusCode::NOT_FOUND,
        AppError::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn error_response(error: &AppError) -> ApiErrorResponse {
    let locale = current_locale();
    match error {
        AppError::InvalidCursor => localized_error_response(locale, ApiErrorKind::InvalidCursor, None),
        AppError::InvalidInput(message) => localized_error_response(locale, ApiErrorKind::InvalidInput, Some(message)),
        AppError::ImportValidation(failures) => import_validation_response(locale, failures),
        AppError::Unauthorized => localized_error_response(
            locale,
            ApiErrorKind::Unauthorized,
            Some(&LocalizedError::new("errors.user.invalid_credentials")),
        ),
        AppError::AccountDisabled => localized_error_response(locale, ApiErrorKind::Forbidden, Some(&LocalizedError::new("errors.user.account_disabled"))),
        AppError::AccountLocked { lock_minutes } => localized_error_response(
            locale,
            ApiErrorKind::Forbidden,
            Some(&LocalizedError::new("errors.user.account_locked").with_param("minutes", lock_minutes.to_string())),
        ),
        AppError::Forbidden(message) => localized_error_response(locale, ApiErrorKind::Forbidden, Some(message)),
        AppError::Conflict(message) => localized_error_response(locale, ApiErrorKind::Conflict, Some(message)),
        AppError::NotFound => localized_error_response(locale, ApiErrorKind::NotFound, None),
        AppError::Infrastructure(_) => localized_error_response(
            locale,
            ApiErrorKind::Infrastructure,
            Some(&LocalizedError::new("errors.common.service_unavailable")),
        ),
    }
}

fn import_validation_response(locale: Locale, failures: &[LocalizedError]) -> ApiErrorResponse {
    let failures = failures
        .iter()
        .map(|failure| translate_localized_error(locale, failure))
        .collect::<Vec<_>>()
        .join("; ");
    let details = LocalizedError::new("errors.user.import_failed").with_param("errors", failures);
    localized_error_response(locale, ApiErrorKind::InvalidInput, Some(&details))
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    use super::{ApiError, StatusCode};
    use crate::application::AppError;

    #[test]
    fn api_error_uses_new_api_http_status() {
        let response = ApiError(AppError::Unauthorized).into_response();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
