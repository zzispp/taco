use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use types::http::{ApiErrorKind, ApiErrorResponse, current_locale, localized_error_response};

use crate::application::CaptchaError;

#[derive(Debug)]
pub struct CaptchaApiError(pub CaptchaError);

impl From<CaptchaError> for CaptchaApiError {
    fn from(value: CaptchaError) -> Self {
        Self(value)
    }
}

impl IntoResponse for CaptchaApiError {
    fn into_response(self) -> Response {
        let body = error_response(&self.0);
        (status_code(&self.0), Json(body)).into_response()
    }
}

fn status_code(error: &CaptchaError) -> StatusCode {
    match error {
        CaptchaError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        CaptchaError::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn error_response(error: &CaptchaError) -> ApiErrorResponse {
    let locale = current_locale();
    match error {
        CaptchaError::InvalidInput(message) => localized_error_response(locale, ApiErrorKind::InvalidInput, Some(message)),
        CaptchaError::Infrastructure(_) => localized_error_response(
            locale,
            ApiErrorKind::Infrastructure,
            Some(&LocalizedError::new("errors.common.service_unavailable")),
        ),
    }
}
