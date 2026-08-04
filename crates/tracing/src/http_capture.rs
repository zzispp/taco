mod body;
mod metadata;

pub(crate) use body::{BodyCaptureOptions, BodyCaptureSnapshot, SharedBodyCapture, wrap_body};
pub(crate) use metadata::{body_value, content_type, query_parameters, request_headers};

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{BodyCaptureSnapshot, body_value, query_parameters, request_headers};

    #[test]
    fn structured_bodies_and_metadata_are_redacted() {
        let body = body_value(
            Some("application/json"),
            BodyCaptureSnapshot {
                bytes: br#"{"password":"raw","body":"raw-body","endpoint":"https://user:pass@example.com/run?token=raw#fragment","request_id":"request-1"}"#
                    .to_vec(),
                ..Default::default()
            },
        );
        let query = query_parameters(&"/api/test?token=raw&page=1".parse().unwrap());
        let headers = request_headers(&[("authorization".parse().unwrap(), "Bearer raw".parse().unwrap())].into_iter().collect());

        assert_eq!(body["content"]["password"], kernel::redaction::REDACTED);
        assert_eq!(body["content"]["body"], "[body omitted]");
        assert_eq!(body["content"]["endpoint"], "https://example.com/run?token=***");
        assert_eq!(query, json!({"token": kernel::redaction::REDACTED, "page": "1"}));
        assert_eq!(headers["authorization"], kernel::redaction::REDACTED);
    }
}
