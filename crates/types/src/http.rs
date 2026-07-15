use axum::{
    Json,
    extract::{
        FromRequest, FromRequestParts, Query, Request,
        rejection::{JsonRejection, QueryRejection},
    },
    http::{StatusCode, header::ACCEPT_LANGUAGE, request::Parts},
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};
use utoipa::ToSchema;

mod locale;
mod time_range;
mod wire_time;
mod xlsx;

pub use locale::{
    ApiErrorKind, Locale, current_locale, locale_middleware, localized_error_response, translate_error, translate_localized_error, translate_message,
    translate_message_with_params,
};
pub use time_range::{DATE_OR_RFC3339_FORMAT, DateTimeRange, DateTimeRangeError, DateTimeRangeField, parse_date_time_range};
pub use wire_time::{format_utc_rfc3339_millis, serialize_utc_rfc3339_millis};
pub use xlsx::{xlsx_attachment, xlsx_file_attachment};

#[derive(Debug, PartialEq, Eq, Serialize, ToSchema)]
pub struct ApiErrorResponse {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl ApiErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    pub fn with_details(code: impl Into<String>, message: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: Some(details.into()),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RequestJson<T>(pub T);

impl<T, S> FromRequest<S> for RequestJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = RequestJsonRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let locale = Locale::from_header_value(req.headers().get(ACCEPT_LANGUAGE));
        Json::<T>::from_request(req, state)
            .await
            .map(|Json(value)| Self(value))
            .map_err(|error| RequestJsonRejection::from_parts(error, locale))
    }
}

/// Localized query extractor that preserves the shared API error shape.
#[derive(Debug, Clone, Copy, Default)]
pub struct RequestQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for RequestQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = RequestQueryRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let locale = Locale::from_header_value(parts.headers.get(ACCEPT_LANGUAGE));
        Query::<T>::from_request_parts(parts, state)
            .await
            .map(|Query(value)| Self(value))
            .map_err(|error| RequestQueryRejection::new(error, locale))
    }
}

#[derive(Debug)]
pub struct RequestQueryRejection {
    body: ApiErrorResponse,
}

impl RequestQueryRejection {
    fn new(_error: QueryRejection, locale: Locale) -> Self {
        let details = kernel::error::LocalizedError::new("errors.common.invalid_input");
        Self {
            body: localized_error_response(locale, ApiErrorKind::InvalidInput, Some(&details)),
        }
    }
}

impl IntoResponse for RequestQueryRejection {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self.body)).into_response()
    }
}

#[derive(Debug)]
pub struct RequestJsonRejection {
    status: StatusCode,
    body: ApiErrorResponse,
}

impl From<JsonRejection> for RequestJsonRejection {
    fn from(value: JsonRejection) -> Self {
        Self::from_parts(value, current_locale())
    }
}

impl RequestJsonRejection {
    fn from_parts(value: JsonRejection, locale: Locale) -> Self {
        let status = match &value {
            JsonRejection::JsonSyntaxError(_) | JsonRejection::JsonDataError(_) => StatusCode::BAD_REQUEST,
            _ => value.status(),
        };
        let details = value.body_text();
        let body = match value {
            JsonRejection::MissingJsonContentType(_) => localized_error_response(
                locale,
                ApiErrorKind::UnsupportedMediaType,
                Some(&kernel::error::LocalizedError::new("errors.http.expected_json_content_type")),
            )
            .with_raw_details(details),
            JsonRejection::JsonSyntaxError(_) | JsonRejection::JsonDataError(_) => localized_error_response(
                locale,
                ApiErrorKind::InvalidJson,
                Some(&kernel::error::LocalizedError::new("errors.http.invalid_json_payload")),
            )
            .with_raw_details(details),
            JsonRejection::BytesRejection(_) => localized_error_response(
                locale,
                ApiErrorKind::InvalidBody,
                Some(&kernel::error::LocalizedError::new("errors.http.failed_to_read_body")),
            )
            .with_raw_details(details),
            _ => localized_error_response(
                locale,
                ApiErrorKind::InvalidJson,
                Some(&kernel::error::LocalizedError::new("errors.http.invalid_json_payload")),
            )
            .with_raw_details(details),
        };
        Self { status, body }
    }
}

trait ErrorResponseDetails {
    fn with_raw_details(self, raw_details: String) -> Self;
}

impl ErrorResponseDetails for ApiErrorResponse {
    fn with_raw_details(mut self, raw_details: String) -> Self {
        if self.details.is_none() {
            self.details = Some(raw_details);
        }
        self
    }
}

impl IntoResponse for RequestJsonRejection {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::extract::Json;
    use axum::http::HeaderValue;
    use axum::response::IntoResponse;
    use serde_json::json;

    use super::{ApiErrorKind, ApiErrorResponse, Locale, RequestJsonRejection, RequestQueryRejection, localized_error_response};

    #[test]
    fn api_error_response_serializes_without_envelope() {
        let response = ApiErrorResponse::with_details("bad_request", "invalid input", "username is required");
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(
            value,
            json!({
                "code": "bad_request",
                "message": "invalid input",
                "details": "username is required"
            })
        );
    }

    #[test]
    fn request_json_rejection_uses_uniform_invalid_json_shape() {
        let rejection = RequestJsonRejection::from(Json::<serde_json::Value>::from_bytes(br#"{"broken":"#).unwrap_err());

        let response = rejection.into_response();

        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn request_json_missing_required_field_is_bad_request() {
        let rejection = RequestJsonRejection::from(Json::<RequiredField>::from_bytes(br#"{}"#).unwrap_err());

        let response = rejection.into_response();

        assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn request_query_rejection_uses_localized_stable_shape() {
        let error = axum::extract::Query::<QueryFixture>::try_from_uri(&"/?limit=invalid".parse().unwrap()).unwrap_err();
        let rejection = RequestQueryRejection::new(error, Locale::En);

        assert_eq!(
            rejection.body,
            ApiErrorResponse::with_details("invalid_input", "Invalid input", "Invalid input")
        );
        assert_eq!(rejection.into_response().status(), axum::http::StatusCode::BAD_REQUEST);
    }

    #[test]
    fn invalid_cursor_has_stable_code_and_localized_message() {
        let expected = [
            (Locale::ZhCn, "游标无效或已不匹配当前查询"),
            (Locale::En, "The cursor is invalid or no longer matches this query"),
            (Locale::ZhTw, "遊標無效或已不符合目前查詢"),
        ];
        for (locale, message) in expected {
            let response = localized_error_response(locale, ApiErrorKind::InvalidCursor, None);
            assert_eq!(response.code, "invalid_cursor");
            assert_eq!(response.message, message);
            assert_eq!(response.details, None);
        }
    }

    #[derive(Debug, serde::Deserialize)]
    struct QueryFixture {
        #[allow(dead_code)]
        limit: u64,
    }

    #[derive(Debug, serde::Deserialize)]
    struct RequiredField {
        #[allow(dead_code)]
        value: String,
    }

    #[test]
    fn locale_parses_accept_language_candidates() {
        assert_eq!(Locale::from_header("zh-CN,zh;q=0.9,en;q=0.8"), Locale::ZhCn);
        assert_eq!(Locale::from_header("zh-Hans"), Locale::ZhCn);
        assert_eq!(Locale::from_header("zh-TW,zh;q=0.9"), Locale::ZhTw);
        assert_eq!(Locale::from_header("zh-Hant"), Locale::ZhTw);
        assert_eq!(Locale::from_header("en-US,en;q=0.9"), Locale::En);
        assert_eq!(Locale::from_header("fr-FR"), Locale::ZhCn);
    }

    #[test]
    fn locale_parses_header_value() {
        let value = HeaderValue::from_static("zh-HK,zh;q=0.9");

        assert_eq!(Locale::from_header_value(Some(&value)), Locale::ZhTw);
    }
}
