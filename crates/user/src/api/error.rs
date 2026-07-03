use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use types::http::ApiErrorResponse;

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
        AppError::Conflict(_) => StatusCode::CONFLICT,
        AppError::NotFound => StatusCode::NOT_FOUND,
        AppError::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn error_response(error: &AppError) -> ApiErrorResponse {
    match error {
        AppError::InvalidInput(message) => ApiErrorResponse::with_details("invalid_input", "invalid input", message.clone()),
        AppError::Unauthorized => ApiErrorResponse::new("unauthorized", "username or password is incorrect"),
        AppError::Conflict(message) => ApiErrorResponse::with_details("conflict", "resource conflict", message.clone()),
        AppError::NotFound => ApiErrorResponse::new("not_found", "user not found"),
        AppError::Infrastructure(message) => ApiErrorResponse::with_details("infrastructure_error", "infrastructure error", message.clone()),
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
