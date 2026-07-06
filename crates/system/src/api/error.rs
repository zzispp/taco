use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use types::http::{ApiErrorKind, ApiErrorResponse, current_locale, localized_error_response};

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
            RbacError::Forbidden => SystemError::Forbidden(LocalizedError::new("errors.common.forbidden")),
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
    let locale = current_locale();
    match error {
        SystemError::NotFound => localized_error_response(locale, ApiErrorKind::NotFound, None),
        SystemError::Forbidden(message) => localized_error_response(locale, ApiErrorKind::Forbidden, Some(message)),
        SystemError::Conflict(message) => localized_error_response(locale, ApiErrorKind::Conflict, Some(message)),
        SystemError::InvalidInput(message) => localized_error_response(locale, ApiErrorKind::InvalidInput, Some(message)),
        SystemError::Infrastructure(_) => localized_error_response(
            locale,
            ApiErrorKind::Infrastructure,
            Some(&LocalizedError::new("errors.common.service_unavailable")),
        ),
    }
}
