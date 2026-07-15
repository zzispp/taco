use std::collections::BTreeMap;

use axum::http::Uri;
use url::form_urlencoded::parse;

use super::{
    BODY_OMITTED, BODY_TRUNCATED, CapturedBody, MULTIPART_OMITTED, normalized_request_id, request_snapshot, request_snapshot_from_capture, sanitized_url,
    truncate,
};

#[test]
fn nested_json_secrets_are_redacted_recursively() {
    let input = br#"{"password":"one","nested":[{"access_token":"two"}],"code":"public-code","captchaCode":"three"}"#;
    let snapshot = request_snapshot(Some("application/json"), input);

    assert_eq!(
        snapshot,
        r#"{"captchaCode":"***","code":"public-code","nested":[{"access_token":"***"}],"password":"***"}"#
    );
    for secret in ["one", "two", "three"] {
        assert!(!snapshot.contains(secret));
    }
}

#[test]
fn sensitive_runtime_config_values_are_redacted_by_config_key() {
    let input = br#"{"config_key":"sys.account.captchaConfig","config_value":"provider-secret"}"#;
    let snapshot = request_snapshot(Some("application/json"), input);

    assert_eq!(snapshot, r#"{"config_key":"sys.account.captchaConfig","config_value":"***"}"#);
    assert!(!snapshot.contains("initial-secret"));
}

#[test]
fn captcha_runtime_config_value_is_fully_redacted() {
    let input = br#"{"config_key":"sys.account.captchaConfig","config_value":"{\"secret_key\":\"raw\"}"}"#;
    let snapshot = request_snapshot(Some("application/json"), input);

    assert_eq!(snapshot, r#"{"config_key":"sys.account.captchaConfig","config_value":"***"}"#);
    assert!(!snapshot.contains("raw"));
}

#[test]
fn json_encoded_runtime_config_values_are_recursively_redacted() {
    let input = br#"{"config_key":"sys.provider.settings","config_value":"{\"secret_key\":\"raw\",\"theme\":\"auto\"}"}"#;
    let snapshot = request_snapshot(Some("application/json"), input);
    let outer: serde_json::Value = serde_json::from_str(&snapshot).unwrap();
    let inner: serde_json::Value = serde_json::from_str(outer["config_value"].as_str().unwrap()).unwrap();

    assert_eq!(inner, serde_json::json!({"secret_key": "***", "theme": "auto"}));
    assert!(!snapshot.contains("raw"));
}

#[test]
fn credential_bearing_http_headers_are_redacted() {
    let input = br#"{"headers":{"Authorization":"Bearer one","Cookie":"session=two","X-API-Key":"three","X-Trace":"visible"}}"#;
    let snapshot = request_snapshot(Some("application/json"), input);

    assert_eq!(
        snapshot,
        r#"{"headers":{"Authorization":"***","Cookie":"***","X-API-Key":"***","X-Trace":"visible"}}"#
    );
    for secret in ["one", "two", "three"] {
        assert!(!snapshot.contains(secret));
    }
}

#[test]
fn nested_body_and_url_fields_are_structurally_sanitized_for_requests() {
    let input = br#"{"request":{"body":{"password":"body-secret"},"endpoint":"https://user:pass@example.com/run?token=query-secret&mode=full#fragment"},"response":{"body":"response-secret","uri":"not a URL"}}"#;
    let request = request_snapshot(Some("application/json"), input);
    let expected = serde_json::json!({
        "request": {
            "body": "[body omitted]",
            "endpoint": "https://example.com/run?token=***&mode=full",
        },
        "response": {
            "body": "[body omitted]",
            "uri": "[URL omitted]",
        },
    });

    assert_eq!(serde_json::from_str::<serde_json::Value>(&request).unwrap(), expected);
    for secret in ["body-secret", "response-secret", "user", "pass", "query-secret", "fragment"] {
        assert!(!request.contains(secret));
    }
}

#[test]
fn invalid_json_and_multipart_never_store_raw_secret_bytes() {
    assert_eq!(request_snapshot(Some("application/json"), br#"{"password":"raw""#), "[invalid JSON omitted]");
    assert_eq!(
        request_snapshot(Some("multipart/form-data; boundary=x"), b"raw-file-password"),
        MULTIPART_OMITTED
    );
}

#[test]
fn opaque_and_untyped_request_bodies_never_store_file_or_credential_bytes() {
    let raw = b"file-content\x00Bearer bearer-secret refresh-token captcha-value";

    for content_type in [None, Some("application/octet-stream"), Some("text/plain"), Some("application/pdf")] {
        let snapshot = request_snapshot(content_type, raw);

        assert_eq!(snapshot, BODY_OMITTED);
        for secret in ["file-content", "bearer-secret", "refresh-token", "captcha-value"] {
            assert!(!snapshot.contains(secret));
        }
    }
}

#[test]
fn url_query_is_redacted_and_limited() {
    let uri: Uri = "/api/test?token=raw&name=alice".parse().unwrap();
    assert_eq!(sanitized_url(&uri), "/api/test?token=***&name=alice");
}

#[test]
fn percent_encoded_sensitive_keys_are_decoded_before_redaction() {
    let uri: Uri = "/api/test?to%6ben=raw&name=alice".parse().unwrap();
    assert_eq!(sanitized_url(&uri), "/api/test?token=***&name=alice");
    assert_eq!(request_snapshot(Some("application/x-www-form-urlencoded"), b"to%6ben=raw"), "token=***");
}

#[test]
fn form_and_url_query_pairs_use_the_same_redaction_rules() {
    let input = b"body=body-secret&endpoint=https%3A%2F%2Fuser%3Apass%40example.com%2Frun%3Ftoken%3Dquery-secret%23fragment&name=alice";
    let request = request_snapshot(Some("application/x-www-form-urlencoded"), input);
    let url: Uri = "/api/test?body=body-secret&endpoint=https%3A%2F%2Fuser%3Apass%40example.com%2Frun%3Ftoken%3Dquery-secret%23fragment"
        .parse()
        .unwrap();

    assert_eq!(form_pairs(&request).get("body").map(String::as_str), Some("[body omitted]"));
    assert_eq!(
        form_pairs(&request).get("endpoint").map(String::as_str),
        Some("https://example.com/run?token=***")
    );
    assert!(!request.contains("body-secret"));
    assert!(!request.contains("user"));
    assert!(!request.contains("pass"));
    assert!(!request.contains("query-secret"));
    assert!(!request.contains("fragment"));
    let sanitized = sanitized_url(&url);
    let sanitized_query = sanitized.split_once('?').unwrap().1;
    assert_eq!(form_pairs(sanitized_query).get("body").map(String::as_str), Some("[body omitted]"));
    assert_eq!(
        form_pairs(sanitized_query).get("endpoint").map(String::as_str),
        Some("https://example.com/run?token=***")
    );
    assert!(!sanitized.contains("body-secret"));
    assert!(!sanitized.contains("user"));
    assert!(!sanitized.contains("pass"));
    assert!(!sanitized.contains("query-secret"));
    assert!(!sanitized.contains("fragment"));
}

fn form_pairs(value: &str) -> BTreeMap<String, String> {
    parse(value.as_bytes()).map(|(key, value)| (key.into_owned(), value.into_owned())).collect()
}

#[test]
fn bounded_capture_uses_an_explicit_marker_without_partial_json() {
    let snapshot = request_snapshot_from_capture(CapturedBody {
        content_type: Some("application/json"),
        bytes: br#"{"password":"raw"}"#,
        truncated: true,
        stream_error: false,
    });
    assert_eq!(snapshot, BODY_TRUNCATED);
    assert!(!snapshot.contains("raw"));
}

#[test]
fn bounded_opaque_and_form_captures_never_keep_raw_secret_bytes() {
    let plain = "文".repeat(2_001);
    let plain = request_snapshot_from_capture(CapturedBody {
        content_type: Some("text/plain"),
        bytes: plain.as_bytes(),
        truncated: true,
        stream_error: false,
    });
    let form = request_snapshot_from_capture(CapturedBody {
        content_type: Some("application/x-www-form-urlencoded"),
        bytes: b"token=raw&name=alice",
        truncated: true,
        stream_error: false,
    });

    assert_eq!(plain, BODY_OMITTED);
    assert_eq!(form, "token=***&name=alice");
}

#[test]
fn truncation_never_splits_utf8_code_points() {
    let value = "中".repeat(1_000);
    let truncated = truncate(&value, 2_000);
    assert_eq!(truncated.chars().count(), 1_000);
    assert_eq!(truncated, value);
}

#[test]
fn url_and_request_id_limits_count_unicode_characters() {
    let uri: Uri = format!("/{}", "中".repeat(300)).parse().unwrap();
    let url = sanitized_url(&uri);
    assert_eq!(url.chars().count(), 255);
    assert_eq!(url, format!("/{}", "中".repeat(254)));

    let request_id = normalized_request_id(&"界".repeat(65));
    assert_eq!(request_id.chars().count(), 64);
    assert_eq!(request_id, "界".repeat(64));
}
