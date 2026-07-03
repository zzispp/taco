use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::http::ApiErrorResponse;

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
    match error {
        RbacError::Unauthorized => ApiErrorResponse::new("unauthorized", "unauthorized"),
        RbacError::Forbidden => ApiErrorResponse::new("forbidden", "forbidden"),
        RbacError::NotFound => ApiErrorResponse::new("not_found", "resource not found"),
        RbacError::Conflict(message) => ApiErrorResponse::with_details("conflict", "resource conflict", message.clone()),
        RbacError::InvalidInput(message) => ApiErrorResponse::with_details("invalid_input", "invalid input", message.clone()),
        RbacError::Infrastructure(message) => ApiErrorResponse::with_details("infrastructure_error", "infrastructure error", message.clone()),
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
