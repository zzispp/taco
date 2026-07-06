use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use types::http::{ApiErrorKind, ApiErrorResponse, current_locale, localized_error_response};

use crate::application::RbacError;

#[derive(Debug)]
pub struct RbacApiError(pub RbacError);

impl From<RbacError> for RbacApiError {
    fn from(value: RbacError) -> Self {
        Self(value)
    }
}

impl IntoResponse for RbacApiError {
    fn into_response(self) -> Response {
        let body = error_response(&self.0);
        (status_code(&self.0), Json(body)).into_response()
    }
}

fn status_code(error: &RbacError) -> StatusCode {
    match error {
        RbacError::Unauthorized => StatusCode::UNAUTHORIZED,
        RbacError::Forbidden => StatusCode::FORBIDDEN,
        RbacError::NotFound => StatusCode::NOT_FOUND,
        RbacError::Conflict(_) => StatusCode::CONFLICT,
        RbacError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        RbacError::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn error_response(error: &RbacError) -> ApiErrorResponse {
    let locale = current_locale();
    match error {
        RbacError::Unauthorized => localized_error_response(locale, ApiErrorKind::Unauthorized, None),
        RbacError::Forbidden => localized_error_response(locale, ApiErrorKind::Forbidden, None),
        RbacError::NotFound => localized_error_response(locale, ApiErrorKind::NotFound, None),
        RbacError::Conflict(message) => localized_error_response(locale, ApiErrorKind::Conflict, Some(message)),
        RbacError::InvalidInput(message) => localized_error_response(locale, ApiErrorKind::InvalidInput, Some(message)),
        RbacError::Infrastructure(_) => localized_error_response(
            locale,
            ApiErrorKind::Infrastructure,
            Some(&LocalizedError::new("errors.common.service_unavailable")),
        ),
    }
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    use super::{RbacApiError, StatusCode};
    use crate::application::RbacError;

    #[test]
    fn rbac_api_error_maps_forbidden_to_http_403() {
        let response = RbacApiError(RbacError::Forbidden).into_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
