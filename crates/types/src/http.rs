use axum::{
    Json,
    extract::{FromRequest, Request, rejection::JsonRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Serialize, de::DeserializeOwned};
use utoipa::ToSchema;

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
        Json::<T>::from_request(req, state)
            .await
            .map(|Json(value)| Self(value))
            .map_err(RequestJsonRejection::from)
    }
}

#[derive(Debug)]
pub struct RequestJsonRejection {
    status: StatusCode,
    body: ApiErrorResponse,
}

impl From<JsonRejection> for RequestJsonRejection {
    fn from(value: JsonRejection) -> Self {
        let status = value.status();
        let details = value.body_text();
        let body = match value {
            JsonRejection::MissingJsonContentType(_) => {
                ApiErrorResponse::with_details("unsupported_media_type", "expected application/json content type", details)
            }
            JsonRejection::JsonSyntaxError(_) | JsonRejection::JsonDataError(_) => {
                ApiErrorResponse::with_details("invalid_json", "invalid JSON payload", details)
            }
            JsonRejection::BytesRejection(_) => ApiErrorResponse::with_details("invalid_body", "failed to read request body", details),
            _ => ApiErrorResponse::with_details("invalid_json", "invalid JSON payload", details),
        };
        Self { status, body }
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
    use axum::response::IntoResponse;
    use serde_json::json;

    use super::{ApiErrorResponse, RequestJsonRejection};

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
}
