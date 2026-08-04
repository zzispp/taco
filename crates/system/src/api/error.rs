use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use types::http::{ApiErrorKind, ApiErrorResponse, Locale, current_locale, localized_error_response};

use crate::application::SystemError;
use rbac::application::RbacError;

const RBAC_UNAUTHORIZED_INFRASTRUCTURE_ERROR: &str = "infra.rbac.unexpected_unauthorized";

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
            RbacError::Unauthorized => SystemError::Infrastructure(RBAC_UNAUTHORIZED_INFRASTRUCTURE_ERROR.into()),
            RbacError::Forbidden => SystemError::Forbidden(LocalizedError::new("errors.common.forbidden")),
            RbacError::NotFound => SystemError::NotFound,
            RbacError::Conflict(message) => SystemError::Conflict(message),
            RbacError::InvalidInput(message) => SystemError::InvalidInput(message),
            RbacError::InvalidCursor => SystemError::InvalidCursor,
            RbacError::Infrastructure(message) => SystemError::Infrastructure(message),
        })
    }
}

impl IntoResponse for SystemApiError {
    fn into_response(self) -> Response {
        if matches!(&self.0, SystemError::Infrastructure(_)) {
            taco_tracing::error_with_fields!("system API infrastructure failure", &self.0, component = "system");
        }
        (status_code(&self.0), Json(error_response(&self.0))).into_response()
    }
}

fn status_code(error: &SystemError) -> StatusCode {
    match error {
        SystemError::NotFound => StatusCode::NOT_FOUND,
        SystemError::Forbidden(_) => StatusCode::FORBIDDEN,
        SystemError::Conflict(_) => StatusCode::CONFLICT,
        SystemError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        SystemError::InvalidCursor => StatusCode::BAD_REQUEST,
        SystemError::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn error_response(error: &SystemError) -> ApiErrorResponse {
    error_response_for_locale(error, current_locale())
}

fn error_response_for_locale(error: &SystemError, locale: Locale) -> ApiErrorResponse {
    match error {
        SystemError::NotFound => localized_error_response(locale, ApiErrorKind::NotFound, None),
        SystemError::Forbidden(message) => localized_error_response(locale, ApiErrorKind::Forbidden, Some(message)),
        SystemError::Conflict(message) => localized_error_response(locale, ApiErrorKind::Conflict, Some(message)),
        SystemError::InvalidInput(message) => localized_error_response(locale, ApiErrorKind::InvalidInput, Some(message)),
        SystemError::InvalidCursor => localized_error_response(locale, ApiErrorKind::InvalidCursor, None),
        SystemError::Infrastructure(_) => localized_error_response(
            locale,
            ApiErrorKind::Infrastructure,
            Some(&LocalizedError::new("errors.common.service_unavailable")),
        ),
    }
}

#[cfg(test)]
mod tests {
    use kernel::error::LocalizedError;
    use types::http::Locale;

    use super::{StatusCode, error_response_for_locale, status_code};
    use crate::application::SystemError;

    #[test]
    fn context_validation_error_is_localized_in_all_supported_languages() {
        let error = SystemError::InvalidInput(LocalizedError::new("errors.system.export_config_unconfigured"));
        let cases = [
            (Locale::ZhCn, "参数错误", "导出参数未配置"),
            (Locale::En, "Invalid input", "Export config is not configured"),
            (Locale::ZhTw, "參數錯誤", "匯出參數未設定"),
        ];

        assert_eq!(status_code(&error), StatusCode::BAD_REQUEST);
        for (locale, message, details) in cases {
            let response = error_response_for_locale(&error, locale);

            assert_eq!(response.code, "invalid_input");
            assert_eq!(response.message, message);
            assert_eq!(response.details.as_deref(), Some(details));
        }
    }

    #[test]
    fn infrastructure_diagnostics_are_not_exposed_in_any_supported_locale() {
        let diagnostic = "PostgreSQL connection rejected";
        let error = SystemError::Infrastructure(diagnostic.into());
        let cases = [
            (Locale::ZhCn, "服务异常", "服务暂不可用"),
            (Locale::En, "Service error", "Service is temporarily unavailable"),
            (Locale::ZhTw, "服務異常", "服務暫不可用"),
        ];

        assert_eq!(status_code(&error), StatusCode::INTERNAL_SERVER_ERROR);
        for (locale, message, details) in cases {
            let response = error_response_for_locale(&error, locale);
            let serialized = serde_json::to_string(&response).unwrap();

            assert_eq!(response.code, "infrastructure_error");
            assert_eq!(response.message, message);
            assert_eq!(response.details.as_deref(), Some(details));
            assert!(!serialized.contains(diagnostic));
        }
    }
}
