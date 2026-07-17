use axum::http::Uri;
use kernel::redaction::{REDACTED, is_sensitive_key, normalize_key, redact_sensitive_field};
use serde_json::{Map, Value};
use url::{
    Url,
    form_urlencoded::{Serializer, parse},
};

pub(crate) const SNAPSHOT_MAX_CHARS: usize = 2_000;
pub(crate) const SNAPSHOT_CAPTURE_MAX_BYTES: usize = SNAPSHOT_MAX_CHARS * MAX_UTF8_CODE_POINT_BYTES;
const MAX_UTF8_CODE_POINT_BYTES: usize = 4;
const URL_MAX_CHARS: usize = 255;
const REQUEST_ID_MAX_CHARS: usize = 64;
const BODY_OMITTED: &str = "[body omitted]";
const URL_OMITTED: &str = "[URL omitted]";
const MULTIPART_OMITTED: &str = "[multipart omitted]";
const INVALID_JSON: &str = "[invalid JSON omitted]";
const JSON_SERIALIZATION_OMITTED: &str = "[JSON serialization omitted]";
const CONFIG_KEY_FIELD: &str = "config_key";
const CONFIG_VALUE_FIELD: &str = "config_value";
pub(crate) const BODY_STREAM_ERROR: &str = "[body stream error omitted]";
const BODY_TRUNCATED: &str = "[body truncated at 2000 characters]";

pub struct CapturedBody<'a> {
    pub content_type: Option<&'a str>,
    pub bytes: &'a [u8],
    pub truncated: bool,
    pub stream_error: bool,
}

pub fn request_snapshot(content_type: Option<&str>, bytes: &[u8]) -> String {
    request_snapshot_from_capture(CapturedBody {
        content_type,
        bytes,
        truncated: false,
        stream_error: false,
    })
}

pub fn request_snapshot_from_capture(input: CapturedBody<'_>) -> String {
    if let Some(marker) = capture_marker(&input) {
        return marker.into();
    }
    request_snapshot_unbounded(input.content_type, input.bytes)
}

fn request_snapshot_unbounded(content_type: Option<&str>, bytes: &[u8]) -> String {
    let content_type = content_type.unwrap_or_default().to_ascii_lowercase();
    if content_type.starts_with("multipart/") {
        return MULTIPART_OMITTED.into();
    }
    if content_type.contains("json") {
        return json_snapshot(bytes);
    }
    if content_type.starts_with("application/x-www-form-urlencoded") {
        return truncate(&redact_form(bytes), SNAPSHOT_MAX_CHARS);
    }

    // Only structured formats can be redacted deterministically. Opaque and
    // untyped bodies may contain file content or credentials, so never persist them.
    BODY_OMITTED.into()
}

pub fn sanitized_url(uri: &Uri) -> String {
    let Some(query) = uri.query() else {
        return truncate(uri.path(), URL_MAX_CHARS);
    };
    let query = redact_pairs(query);
    truncate(&format!("{}?{query}", uri.path()), URL_MAX_CHARS)
}

pub(crate) fn normalized_request_id(value: &str) -> String {
    truncate(value, REQUEST_ID_MAX_CHARS)
}

fn json_snapshot(bytes: &[u8]) -> String {
    let Ok(mut value) = serde_json::from_slice::<Value>(bytes) else {
        return INVALID_JSON.into();
    };
    redact_json(&mut value);
    truncate(&serialize_json(&value), SNAPSHOT_MAX_CHARS)
}

fn redact_json(value: &mut Value) {
    match value {
        Value::Object(object) => redact_object(object),
        Value::Array(values) => values.iter_mut().for_each(redact_json),
        _ => {}
    }
}

fn redact_object(object: &mut Map<String, Value>) {
    redact_config_value(object);
    for (key, value) in object {
        redact_field(key, value);
    }
}

fn redact_config_value(object: &mut Map<String, Value>) {
    let sensitive_key = object.get(CONFIG_KEY_FIELD).and_then(Value::as_str).is_some_and(is_sensitive_key);
    let Some(value) = object.get_mut(CONFIG_VALUE_FIELD) else {
        return;
    };
    if sensitive_key {
        *value = Value::String(REDACTED.into());
        return;
    }
    redact_embedded_config(value);
}

fn redact_embedded_config(value: &mut Value) {
    let Value::String(raw) = value else {
        redact_json(value);
        return;
    };
    let trimmed = raw.trim_start();
    if !trimmed.starts_with('{') && !trimmed.starts_with('[') {
        return;
    }
    let Ok(mut nested) = serde_json::from_str::<Value>(raw) else {
        *value = Value::String(REDACTED.into());
        return;
    };
    redact_json(&mut nested);
    *value = Value::String(nested.to_string());
}

fn redact_field(key: &str, value: &mut Value) {
    if body_field(key) {
        *value = Value::String(BODY_OMITTED.into());
        return;
    }
    if url_field(key) {
        *value = Value::String(redact_url(value).unwrap_or_else(|| URL_OMITTED.into()));
        return;
    }
    if redact_sensitive_field(key, value) {
        return;
    }
    redact_json(value);
}

fn serialize_json(value: &Value) -> String {
    match serde_json::to_string(value) {
        Ok(serialized) => serialized,
        Err(error) => {
            taco_tracing::error_with_fields!(
                "operation audit JSON snapshot serialization failed",
                &error,
                event_type = "operation",
                reason = "serialization_failed"
            );
            JSON_SERIALIZATION_OMITTED.into()
        }
    }
}

fn redact_form(bytes: &[u8]) -> String {
    redact_pairs(&String::from_utf8_lossy(bytes))
}

fn redact_pairs(value: &str) -> String {
    let mut serializer = Serializer::new(String::new());
    for (key, value) in parse(value.as_bytes()) {
        serializer.append_pair(&key, &redact_pair_value(&key, value.as_ref()));
    }
    serializer.finish()
}

fn redact_pair_value(key: &str, value: &str) -> String {
    if body_field(key) {
        return BODY_OMITTED.into();
    }
    if url_field(key) {
        return redact_url_text(value).unwrap_or_else(|| URL_OMITTED.into());
    }
    if is_sensitive_key(key) {
        return REDACTED.into();
    }
    value.into()
}

fn redact_url(value: &Value) -> Option<String> {
    redact_url_text(value.as_str()?)
}

fn redact_url_text(raw: &str) -> Option<String> {
    let mut url = Url::parse(raw).ok()?;
    url.set_username("").ok()?;
    url.set_password(None).ok()?;
    url.set_fragment(None);
    if let Some(query) = url.query() {
        url.set_query(Some(&redact_pairs(query)));
    }
    Some(url.into())
}

fn capture_marker(input: &CapturedBody<'_>) -> Option<&'static str> {
    if input.stream_error {
        return Some(BODY_STREAM_ERROR);
    }
    (input.truncated && requires_complete_body(input.content_type)).then_some(BODY_TRUNCATED)
}

fn requires_complete_body(content_type: Option<&str>) -> bool {
    let content_type = content_type.unwrap_or_default().to_ascii_lowercase();
    content_type.contains("json") || content_type.starts_with("multipart/")
}

fn body_field(key: &str) -> bool {
    normalize_key(key) == "body"
}

fn url_field(key: &str) -> bool {
    matches!(normalize_key(key).as_str(), "url" | "uri" | "endpoint")
}

pub fn truncate(value: &str, max_chars: usize) -> String {
    let Some((boundary, _)) = value.char_indices().nth(max_chars) else {
        return value.into();
    };
    value[..boundary].into()
}

#[cfg(test)]
#[path = "sanitize_tests.rs"]
mod tests;
