use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use kernel::error::LocalizedError;
use types::http::{ApiErrorKind, ApiErrorResponse, Locale, current_locale, localized_error_response};

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
        if matches!(&self.0, RbacError::Infrastructure(_)) {
            taco_tracing::error_with_fields!("RBAC API infrastructure failure", &self.0, component = "rbac");
        }
        let body = error_response(&self.0);
        (status_code(&self.0), Json(body)).into_response()
    }
}

fn status_code(error: &RbacError) -> StatusCode {
    match error {
        RbacError::InvalidCursor => StatusCode::BAD_REQUEST,
        RbacError::Unauthorized => StatusCode::UNAUTHORIZED,
        RbacError::Forbidden => StatusCode::FORBIDDEN,
        RbacError::NotFound => StatusCode::NOT_FOUND,
        RbacError::Conflict(_) => StatusCode::CONFLICT,
        RbacError::InvalidInput(_) => StatusCode::BAD_REQUEST,
        RbacError::Infrastructure(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn error_response(error: &RbacError) -> ApiErrorResponse {
    error_response_for_locale(error, current_locale())
}

fn error_response_for_locale(error: &RbacError, locale: Locale) -> ApiErrorResponse {
    match error {
        RbacError::InvalidCursor => localized_error_response(locale, ApiErrorKind::InvalidCursor, None),
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
    use kernel::error::LocalizedError;
    use types::http::Locale;

    use super::{RbacApiError, StatusCode, error_response_for_locale, status_code};
    use crate::application::RbacError;

    #[test]
    fn rbac_api_error_maps_forbidden_to_http_403() {
        let response = RbacApiError(RbacError::Forbidden).into_response();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn invalid_date_range_is_localized_in_all_supported_languages() {
        let cases = [
            (Locale::ZhCn, "参数错误", "开始时间不能晚于结束时间"),
            (Locale::En, "Invalid input", "Start time must not be later than end time"),
            (Locale::ZhTw, "參數錯誤", "開始時間不能晚於結束時間"),
        ];
        for (locale, message, details) in cases {
            let error = RbacError::InvalidInput(LocalizedError::new("errors.rbac.invalid_date_range"));
            let body = error_response_for_locale(&error, locale);

            assert_eq!(body.code, "invalid_input");
            assert_eq!(body.message, message);
            assert_eq!(body.details.as_deref(), Some(details));
        }
    }

    #[test]
    fn infrastructure_diagnostics_are_not_exposed_in_any_supported_locale() {
        let diagnostic = "redis endpoint=private";
        let error = RbacError::Infrastructure(diagnostic.into());
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
