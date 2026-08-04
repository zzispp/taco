use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use types::http::{ApiErrorKind, ApiErrorResponse, Locale, current_locale, localized_error_response};

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
        if matches!(&self.0, ObservabilityError::PartialCleanup { .. } | ObservabilityError::Infrastructure(_)) {
            taco_tracing::error_with_fields!("system log API failure", &self.0, component = "observability");
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
    body_for_locale(error, current_locale())
}

fn body_for_locale(error: &ObservabilityError, locale: Locale) -> ApiErrorResponse {
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
    use kernel::error::LocalizedError;
    use types::http::Locale;

    use super::{body, body_for_locale, status};

    #[test]
    fn invalid_cursor_uses_the_shared_stable_api_contract() {
        assert_eq!(status(&ObservabilityError::InvalidCursor), StatusCode::BAD_REQUEST);
        assert_eq!(body(&ObservabilityError::InvalidCursor).code, "invalid_cursor");
    }

    #[test]
    fn context_validation_error_is_localized_in_all_supported_languages() {
        let error = ObservabilityError::InvalidInput(LocalizedError::new("errors.observability.invalid_level"));
        let cases = [
            (Locale::ZhCn, "参数错误", "日志级别无效"),
            (Locale::En, "Invalid input", "Invalid log level"),
            (Locale::ZhTw, "參數錯誤", "日誌級別無效"),
        ];

        assert_eq!(status(&error), StatusCode::BAD_REQUEST);
        for (locale, message, details) in cases {
            let response = body_for_locale(&error, locale);

            assert_eq!(response.code, "invalid_input");
            assert_eq!(response.message, message);
            assert_eq!(response.details.as_deref(), Some(details));
        }
    }

    #[test]
    fn cleanup_and_infrastructure_diagnostics_are_not_exposed_in_any_supported_locale() {
        let diagnostic = "partition drop failed: internal relation name";
        let errors = [
            ObservabilityError::PartialCleanup {
                rows_deleted: 7,
                dropped_partitions: 2,
                batches: 3,
                failure: diagnostic.into(),
            },
            ObservabilityError::Infrastructure(diagnostic.into()),
        ];
        let cases = [
            (Locale::ZhCn, "服务异常", "服务暂不可用"),
            (Locale::En, "Service error", "Service is temporarily unavailable"),
            (Locale::ZhTw, "服務異常", "服務暫不可用"),
        ];

        for error in errors {
            assert_eq!(status(&error), StatusCode::SERVICE_UNAVAILABLE);
            for (locale, message, details) in cases {
                let response = body_for_locale(&error, locale);
                let serialized = serde_json::to_string(&response).unwrap();

                assert_eq!(response.code, "infrastructure_error");
                assert_eq!(response.message, message);
                assert_eq!(response.details.as_deref(), Some(details));
                assert!(!serialized.contains(diagnostic));
            }
        }
    }
}
