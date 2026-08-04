use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use types::http::{ApiErrorKind, ApiErrorResponse, Locale, current_locale, localized_error_response};

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
            taco_tracing::error_with_fields!("audit API infrastructure failure", &self.0, component = "audit");
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
    body_for_locale(error, current_locale())
}

fn body_for_locale(error: &AuditError, locale: Locale) -> ApiErrorResponse {
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
    use kernel::error::LocalizedError;
    use types::http::Locale;

    use super::{body, body_for_locale, status};

    #[test]
    fn invalid_cursor_uses_the_shared_stable_api_contract() {
        assert_eq!(status(&AuditError::InvalidCursor), StatusCode::BAD_REQUEST);
        let response = body(&AuditError::InvalidCursor);
        assert_eq!(response.code, "invalid_cursor");
        assert_eq!(response.details, None);
    }

    #[test]
    fn context_validation_error_is_localized_in_all_supported_languages() {
        let error = AuditError::InvalidInput(LocalizedError::new("errors.audit.invalid_status"));
        let cases = [
            (Locale::ZhCn, "参数错误", "日志状态无效"),
            (Locale::En, "Invalid input", "Invalid log status"),
            (Locale::ZhTw, "參數錯誤", "日誌狀態無效"),
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
    fn infrastructure_diagnostics_are_not_exposed_in_any_supported_locale() {
        let diagnostic = "database user=internal";
        let error = AuditError::Infrastructure(diagnostic.into());
        let cases = [
            (Locale::ZhCn, "服务异常", "服务暂不可用"),
            (Locale::En, "Service error", "Service is temporarily unavailable"),
            (Locale::ZhTw, "服務異常", "服務暫不可用"),
        ];

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
