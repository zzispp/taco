use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::http::ApiErrorResponse;

use crate::application::SystemError;
use rbac::application::RbacError;

#[derive(Debug)]
pub struct SystemApiError(pub SystemError);

impl From<SystemError> for SystemApiError {
    fn from(value: SystemError) -> Self {
        Self(value)
    }
}

impl From<RbacError> for SystemApiError {
    fn from(value: RbacError) -> Self {
        Self(match value {
            RbacError::Unauthorized => SystemError::Infrastructure("unauthorized".into()),
            RbacError::Forbidden => SystemError::Forbidden("forbidden".into()),
            RbacError::NotFound => SystemError::NotFound,
            RbacError::Conflict(message) => SystemError::Conflict(message),
            RbacError::InvalidInput(message) => SystemError::InvalidInput(message),
            RbacError::Infrastructure(message) => SystemError::Infrastructure(message),
        })
    }
}

impl IntoResponse for SystemApiError {
    fn into_response(self) -> Response {
        (status_code(&self.0), Json(error_response(&self.0))).into_response()
    }
}

fn status_code(error: &SystemError) -> StatusCode {
    match error {
        SystemError::NotFound => StatusCode::NOT_FOUND,
        SystemError::Forbidden(_) => StatusCode::FORBIDDEN,
        SystemError::Conflict(_) => StatusCode::CONFLICT,
        SystemError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        SystemError::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn error_response(error: &SystemError) -> ApiErrorResponse {
    match error {
        SystemError::NotFound => ApiErrorResponse::new("not_found", "resource not found"),
        SystemError::Forbidden(message) => ApiErrorResponse::new("forbidden", message.clone()),
        SystemError::Conflict(message) => ApiErrorResponse::with_details("conflict", "resource conflict", message.clone()),
        SystemError::InvalidInput(message) => ApiErrorResponse::with_details("invalid_input", "invalid input", message.clone()),
        SystemError::Infrastructure(message) => ApiErrorResponse::with_details("infrastructure_error", "infrastructure error", message.clone()),
    }
}
