use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use types::http::{ApiErrorKind, ApiErrorResponse, Locale, current_locale, localized_error_response};

use crate::application::SchedulerError;

#[derive(Debug)]
pub struct SchedulerApiError(pub SchedulerError);

impl From<SchedulerError> for SchedulerApiError {
    fn from(value: SchedulerError) -> Self {
        Self(value)
    }
}

impl IntoResponse for SchedulerApiError {
    fn into_response(self) -> Response {
        if matches!(self.0, SchedulerError::Infrastructure(_)) {
            taco_tracing::error_with_fields!("scheduler API infrastructure failure", &self.0, component = "scheduler");
        }
        (status_code(&self.0), Json(error_response(&self.0))).into_response()
    }
}

fn status_code(error: &SchedulerError) -> StatusCode {
    match error {
        SchedulerError::NotFound => StatusCode::NOT_FOUND,
        SchedulerError::Forbidden(_) => StatusCode::FORBIDDEN,
        SchedulerError::Conflict { .. } => StatusCode::CONFLICT,
        SchedulerError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        SchedulerError::InvalidCursor => StatusCode::BAD_REQUEST,
        SchedulerError::Infrastructure(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

fn error_response(error: &SchedulerError) -> ApiErrorResponse {
    localized_response(error, current_locale())
}

fn localized_response(error: &SchedulerError, locale: Locale) -> ApiErrorResponse {
    match error {
        SchedulerError::NotFound => localized_error_response(locale, ApiErrorKind::NotFound, None),
        SchedulerError::Forbidden(details) => localized_error_response(locale, ApiErrorKind::Forbidden, Some(details)),
        SchedulerError::Conflict { code, details } => {
            let mut response = localized_error_response(locale, ApiErrorKind::Conflict, Some(details));
            response.code = (*code).into();
            response
        }
        SchedulerError::InvalidInput(details) => localized_error_response(locale, ApiErrorKind::InvalidInput, Some(details)),
        SchedulerError::InvalidCursor => localized_error_response(locale, ApiErrorKind::InvalidCursor, None),
        SchedulerError::Infrastructure(_) => localized_error_response(
            locale,
            ApiErrorKind::Infrastructure,
            Some(&LocalizedError::new("errors.common.service_unavailable")),
        ),
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use types::http::Locale;

    use crate::application::SchedulerError;

    use super::{localized_response, status_code};

    #[test]
    fn scheduler_conflicts_preserve_their_public_codes() {
        let cases = [
            (
                "scheduler_execution_active",
                "errors.scheduler.execution_active",
                "The job has a pending or running execution",
            ),
            (
                "scheduler_job_changed",
                "errors.scheduler.job_changed",
                "Job was changed by another request; refresh and retry",
            ),
        ];

        for (code, details_key, expected_details) in cases {
            let error = SchedulerError::conflict(code, details_key);
            let response = localized_response(&error, Locale::En);

            assert_eq!(status_code(&error), StatusCode::CONFLICT);
            assert_eq!(response.code, code);
            assert_eq!(response.message, "Resource conflict");
            assert_eq!(response.details.as_deref(), Some(expected_details));
        }
    }

    #[test]
    fn conflict_message_and_details_use_the_request_locale() {
        let error = SchedulerError::conflict("scheduler_execution_active", "errors.scheduler.execution_active");
        let cases = [
            (Locale::ZhCn, "资源冲突", "任务存在待执行或运行中的实例"),
            (Locale::En, "Resource conflict", "The job has a pending or running execution"),
            (Locale::ZhTw, "資源衝突", "任務存在待執行或執行中的實例"),
        ];

        for (locale, expected_message, expected_details) in cases {
            let response = localized_response(&error, locale);
            assert_eq!(response.message, expected_message);
            assert_eq!(response.details.as_deref(), Some(expected_details));
        }
    }

    #[test]
    fn infrastructure_response_does_not_expose_diagnostics() {
        let diagnostic = "postgres password=raw-secret";
        let error = SchedulerError::Infrastructure(diagnostic.into());

        let response = localized_response(&error, Locale::En);
        let serialized = serde_json::to_string(&response).unwrap();

        assert_eq!(status_code(&error), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(response.code, "infrastructure_error");
        assert_eq!(response.message, "Service error");
        assert_eq!(response.details.as_deref(), Some("Service is temporarily unavailable"));
        assert!(!serialized.contains(diagnostic));
    }

    #[test]
    fn invalid_trigger_type_response_is_localized() {
        let error = SchedulerError::InvalidInput(crate::application::localized("errors.scheduler.invalid_trigger_type"));
        let cases = [
            (Locale::ZhCn, "参数错误", "任务日志触发方式无效"),
            (Locale::En, "Invalid input", "Invalid job log trigger type"),
            (Locale::ZhTw, "參數錯誤", "任務日誌觸發方式無效"),
        ];

        assert_eq!(status_code(&error), StatusCode::BAD_REQUEST);
        for (locale, expected_message, expected_details) in cases {
            let response = localized_response(&error, locale);
            assert_eq!(response.code, "invalid_input");
            assert_eq!(response.message, expected_message);
            assert_eq!(response.details.as_deref(), Some(expected_details));
        }
    }

    #[test]
    fn invalid_cursor_uses_the_shared_stable_contract() {
        let error = SchedulerError::InvalidCursor;
        let response = localized_response(&error, Locale::En);

        assert_eq!(status_code(&error), StatusCode::BAD_REQUEST);
        assert_eq!(response.code, "invalid_cursor");
        assert_eq!(response.details, None);
    }
}
