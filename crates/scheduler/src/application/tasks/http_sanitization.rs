use serde_json::{Value, json};

use super::http_request::HTTP_REQUEST_TASK_KEY;

pub(crate) const HTTP_EXECUTION_DETAIL_KIND: &str = "http_exchange";
const URL_OMITTED: &str = "[URL omitted]";
const METHOD_OMITTED: &str = "[method omitted]";
const HTTP_METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];
const HTTP_FAILURE_CODES: &[&str] = &["request_build", "timeout", "connect", "request", "response_body", "http_status"];

/// HTTP task configurations can contain credentials and arbitrary file data.
/// Execution logs retain only a safe routing summary; headers, bodies, and
/// query parameters are deliberately never copied into an execution detail.
pub(crate) fn sanitize_http_url(raw: &str) -> String {
    let Ok(mut url) = reqwest::Url::parse(raw) else {
        return URL_OMITTED.into();
    };
    if !matches!(url.scheme(), "http" | "https") {
        return URL_OMITTED.into();
    }
    if url.set_username("").is_err() || url.set_password(None).is_err() {
        return URL_OMITTED.into();
    }
    url.set_query(None);
    url.set_fragment(None);
    url.to_string()
}

pub(crate) fn sanitize_http_method(raw: &str) -> String {
    HTTP_METHODS
        .iter()
        .find(|candidate| candidate.eq_ignore_ascii_case(raw))
        .map(|method| (*method).into())
        .unwrap_or_else(|| METHOD_OMITTED.into())
}

pub(crate) fn sanitize_http_task_params(params: Value) -> Value {
    let object = params.as_object();
    json!({
        "method": sanitized_method_value(object.and_then(|value| value.get("method"))),
        "url": sanitized_url_value(object.and_then(|value| value.get("url"))),
    })
}

pub(crate) fn sanitize_execution_task_params(task_key: &str, params: Value) -> Value {
    if task_key == HTTP_REQUEST_TASK_KEY {
        return sanitize_http_task_params(params);
    }
    params
}

pub(crate) fn sanitize_http_execution_payload(payload: Value) -> Value {
    let object = payload.as_object();
    json!({
        "duration_ms": object.and_then(|value| value.get("duration_ms")).and_then(Value::as_u64).unwrap_or_default(),
        "request": sanitized_request(object.and_then(|value| value.get("request"))),
        "response": sanitized_response(object.and_then(|value| value.get("response"))),
        "failure": sanitized_failure(object.and_then(|value| value.get("failure"))),
    })
}

pub(crate) fn sanitize_http_invoke_target(task_key: &str, invoke_target: &str) -> String {
    if task_key == HTTP_REQUEST_TASK_KEY {
        return redacted_http_invoke_target();
    }
    invoke_target.into()
}

pub(crate) fn redacted_http_invoke_target() -> String {
    format!("{HTTP_REQUEST_TASK_KEY}(...)")
}

fn sanitized_method_value(value: Option<&Value>) -> String {
    value.and_then(Value::as_str).map(sanitize_http_method).unwrap_or_else(|| METHOD_OMITTED.into())
}

fn sanitized_request(value: Option<&Value>) -> Value {
    let object = value.and_then(Value::as_object);
    json!({
        "method": sanitized_method_value(object.and_then(|value| value.get("method"))),
        "url": sanitized_url_value(object.and_then(|value| value.get("url"))),
        "headers": [],
        "body": Value::Null,
    })
}

fn sanitized_response(value: Option<&Value>) -> Value {
    let Some(object) = value.and_then(Value::as_object) else {
        return Value::Null;
    };
    json!({
        "status": object.get("status").and_then(Value::as_u64).filter(|status| *status <= u64::from(u16::MAX)).unwrap_or_default(),
        "final_url": sanitized_url_value(object.get("final_url")),
        "headers": [],
        "body": Value::Null,
    })
}

fn sanitized_failure(value: Option<&Value>) -> Value {
    let Some(code) = value
        .and_then(Value::as_object)
        .and_then(|object| object.get("code"))
        .and_then(Value::as_str)
        .filter(|code| HTTP_FAILURE_CODES.contains(code))
    else {
        return Value::Null;
    };
    json!({ "code": code })
}

fn sanitized_url_value(value: Option<&Value>) -> String {
    value.and_then(Value::as_str).map(sanitize_http_url).unwrap_or_else(|| URL_OMITTED.into())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{
        HTTP_REQUEST_TASK_KEY, redacted_http_invoke_target, sanitize_execution_task_params, sanitize_http_execution_payload, sanitize_http_invoke_target,
        sanitize_http_task_params, sanitize_http_url,
    };

    const SECRET_MARKERS: &[&str] = &[
        "url-user",
        "url-password",
        "query-token",
        "request-password",
        "request-token",
        "request-captcha",
        "request-file-content",
        "response-token",
        "response-file-content",
    ];

    #[test]
    fn sanitized_url_removes_userinfo_query_and_fragment() {
        let value = sanitize_http_url("https://url-user:url-password@example.test/path?token=query-token&mode=full#fragment");

        assert_eq!(value, "https://example.test/path");
    }

    #[test]
    fn execution_projections_exclude_all_http_secret_sources() {
        let params = json!({
            "method": "POST",
            "url": "https://url-user:url-password@example.test/request?token=query-token",
            "headers": {"Authorization": "Bearer request-token"},
            "body": "request-password request-captcha request-file-content",
        });
        let payload = json!({
            "duration_ms": 12,
            "request": params.clone(),
            "response": {
                "status": 503,
                "final_url": "https://url-user:url-password@example.test/response?token=response-token",
                "headers": [{"name": "Set-Cookie", "value": "response-token"}],
                "body": "response-file-content",
            },
            "failure": {"code": "http_status"},
        });

        let params = sanitize_http_task_params(params);
        let payload = sanitize_http_execution_payload(payload);
        let rendered = format!("{params}{payload}");

        assert_eq!(params, json!({"method": "POST", "url": "https://example.test/request"}));
        assert_eq!(payload["request"]["headers"], json!([]));
        assert_eq!(payload["request"]["body"], serde_json::Value::Null);
        assert_eq!(payload["response"]["headers"], json!([]));
        assert_eq!(payload["response"]["body"], serde_json::Value::Null);
        assert_eq!(payload["failure"], json!({"code": "http_status"}));
        for marker in SECRET_MARKERS {
            assert!(!rendered.contains(marker), "sanitized execution projection leaked {marker}");
        }
    }

    #[test]
    fn http_execution_target_is_replaced_in_legacy_log_summaries() {
        let target = sanitize_http_invoke_target(
            HTTP_REQUEST_TASK_KEY,
            "httpClient.request(POST, https://url-user:url-password@example.test?token=query-token)",
        );

        assert_eq!(target, "httpClient.request(...)");
        assert_eq!(redacted_http_invoke_target(), target);
    }

    #[test]
    fn terminal_http_task_params_keep_only_safe_execution_metadata() {
        let params = sanitize_execution_task_params(
            HTTP_REQUEST_TASK_KEY,
            json!({
                "method": "POST",
                "url": "https://url-user:url-password@example.test/run?token=query-token",
                "headers": {"Authorization": "request-token"},
                "body": "request-file-content",
            }),
        );

        assert_eq!(params, json!({"method": "POST", "url": "https://example.test/run"}));
    }
}
