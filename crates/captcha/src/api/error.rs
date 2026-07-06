use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::http::ApiErrorResponse;

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
    match error {
        CaptchaError::InvalidInput(message) => ApiErrorResponse::with_details("invalid_input", "invalid input", message.clone()),
        CaptchaError::Infrastructure(message) => ApiErrorResponse::with_details("infrastructure_error", "infrastructure error", message.clone()),
    }
}
