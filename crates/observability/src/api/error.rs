use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use types::http::{ApiErrorKind, ApiErrorResponse, current_locale, localized_error_response};

use crate::application::ObservabilityError;

#[derive(Debug)]
pub struct SystemLogApiError(pub ObservabilityError);

impl From<ObservabilityError> for SystemLogApiError {
    fn from(value: ObservabilityError) -> Self {
        Self(value)
    }
}

impl IntoResponse for SystemLogApiError {
    fn into_response(self) -> Response {
        if matches!(self.0, ObservabilityError::Infrastructure(_)) {
            taco_tracing::error_with_fields!("system log API infrastructure failure", &self.0, component = "observability");
        }
        (status(&self.0), Json(body(&self.0))).into_response()
    }
}

fn status(error: &ObservabilityError) -> StatusCode {
    match error {
        ObservabilityError::NotFound => StatusCode::NOT_FOUND,
        ObservabilityError::Conflict { .. } => StatusCode::CONFLICT,
        ObservabilityError::InvalidInput(_) | ObservabilityError::InvalidCursor => StatusCode::BAD_REQUEST,
        ObservabilityError::PartialCleanup { .. } | ObservabilityError::Infrastructure(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

fn body(error: &ObservabilityError) -> ApiErrorResponse {
    let locale = current_locale();
    match error {
        ObservabilityError::NotFound => localized_error_response(locale, ApiErrorKind::NotFound, None),
        ObservabilityError::Conflict { code, details } => {
            let mut response = localized_error_response(locale, ApiErrorKind::Conflict, Some(details));
            response.code = (*code).into();
            response
        }
        ObservabilityError::InvalidInput(details) => localized_error_response(locale, ApiErrorKind::InvalidInput, Some(details)),
        ObservabilityError::InvalidCursor => localized_error_response(locale, ApiErrorKind::InvalidCursor, None),
        ObservabilityError::PartialCleanup { .. } | ObservabilityError::Infrastructure(_) => localized_error_response(
            locale,
            ApiErrorKind::Infrastructure,
            Some(&LocalizedError::new("errors.common.service_unavailable")),
        ),
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use crate::application::ObservabilityError;

    use super::{body, status};

    #[test]
    fn invalid_cursor_uses_the_shared_stable_api_contract() {
        assert_eq!(status(&ObservabilityError::InvalidCursor), StatusCode::BAD_REQUEST);
        assert_eq!(body(&ObservabilityError::InvalidCursor).code, "invalid_cursor");
    }
}
