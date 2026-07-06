use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use types::http::{ApiErrorKind, ApiErrorResponse, current_locale, localized_error_response};

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
        let body = error_response(&self.0);
        (status_code(&self.0), Json(body)).into_response()
    }
}

fn status_code(error: &AppError) -> StatusCode {
    match error {
        AppError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        AppError::Unauthorized => StatusCode::UNAUTHORIZED,
        AppError::Forbidden(_) => StatusCode::FORBIDDEN,
        AppError::Conflict(_) => StatusCode::CONFLICT,
        AppError::NotFound => StatusCode::NOT_FOUND,
        AppError::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn error_response(error: &AppError) -> ApiErrorResponse {
    let locale = current_locale();
    match error {
        AppError::InvalidInput(message) => localized_error_response(locale, ApiErrorKind::InvalidInput, Some(message)),
        AppError::Unauthorized => localized_error_response(
            locale,
            ApiErrorKind::Unauthorized,
            Some(&LocalizedError::new("errors.user.invalid_credentials")),
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
