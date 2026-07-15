use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use types::http::{ApiErrorKind, ApiErrorResponse, current_locale, localized_error_response};

use crate::application::AuditError;

#[derive(Debug)]
pub struct AuditApiError(pub AuditError);

impl From<AuditError> for AuditApiError {
    fn from(value: AuditError) -> Self {
        Self(value)
    }
}

impl IntoResponse for AuditApiError {
    fn into_response(self) -> Response {
        if matches!(self.0, AuditError::Infrastructure(_)) {
            hook_tracing::error_with_fields!("audit API infrastructure failure", &self.0, component = "audit");
        }
        (status(&self.0), Json(body(&self.0))).into_response()
    }
}

fn status(error: &AuditError) -> StatusCode {
    match error {
        AuditError::NotFound => StatusCode::NOT_FOUND,
        AuditError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        AuditError::InvalidCursor => StatusCode::BAD_REQUEST,
        AuditError::Infrastructure(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

fn body(error: &AuditError) -> ApiErrorResponse {
    let locale = current_locale();
    match error {
        AuditError::NotFound => localized_error_response(locale, ApiErrorKind::NotFound, None),
        AuditError::InvalidInput(details) => localized_error_response(locale, ApiErrorKind::InvalidInput, Some(details)),
        AuditError::InvalidCursor => localized_error_response(locale, ApiErrorKind::InvalidCursor, None),
        AuditError::Infrastructure(_) => localized_error_response(
            locale,
            ApiErrorKind::Infrastructure,
            Some(&LocalizedError::new("errors.common.service_unavailable")),
        ),
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use crate::application::AuditError;

    use super::{body, status};

    #[test]
    fn invalid_cursor_uses_the_shared_stable_api_contract() {
        assert_eq!(status(&AuditError::InvalidCursor), StatusCode::BAD_REQUEST);
        let response = body(&AuditError::InvalidCursor);
        assert_eq!(response.code, "invalid_cursor");
        assert_eq!(response.details, None);
    }
}
