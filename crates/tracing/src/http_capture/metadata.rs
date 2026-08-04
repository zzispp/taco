use axum::http::{HeaderMap, Uri, header::CONTENT_TYPE};
use serde_json::{Map, Value};
use url::{
    Url,
    form_urlencoded::{Serializer, parse},
};

use kernel::redaction::{REDACTED, is_sensitive_key, normalize_key, redact_sensitive_field};

use super::BodyCaptureSnapshot;

const BODY_OMITTED: &str = "[body omitted]";
const URL_OMITTED: &str = "[URL omitted]";
const CONFIG_KEY_FIELD: &str = "config_key";
const CONFIG_VALUE_FIELD: &str = "config_value";

pub(crate) fn content_type(headers: &HeaderMap) -> Option<String> {
    headers.get(CONTENT_TYPE).and_then(|value| value.to_str().ok()).map(str::to_owned)
}

pub(crate) fn body_value(content_type: Option<&str>, snapshot: BodyCaptureSnapshot) -> Value {
    if snapshot.stream_error {
        return unavailable_body("stream_error");
    }
    if snapshot.truncated {
        return unavailable_body("truncated");
    }
    let Some(content_type) = content_type else {
        return unavailable_body("unsupported_content_type");
    };
    let content_type = content_type.to_ascii_lowercase();
    if content_type.contains("json") {
        return json_body(&snapshot.bytes);
    }
    if content_type.starts_with("application/x-www-form-urlencoded") {
        return form_body(&snapshot.bytes);
    }
    unavailable_body("unsupported_content_type")
}

pub(crate) fn query_parameters(uri: &Uri) -> Value {
    let mut values = Map::new();
    for (key, value) in uri.query().into_iter().flat_map(|query| parse(query.as_bytes())) {
        insert_value(&mut values, key.into_owned(), Value::String(value.into_owned()));
    }
    let mut result = Value::Object(values);
    redact_capture_value(&mut result);
    result
}

pub(crate) fn request_headers(headers: &HeaderMap) -> Value {
    let mut values = Map::new();
    for (name, value) in headers {
        insert_value(
            &mut values,
            name.as_str().into(),
            Value::String(String::from_utf8_lossy(value.as_bytes()).into()),
        );
    }
    let mut result = Value::Object(values);
    redact_capture_value(&mut result);
    result
}

fn json_body(bytes: &[u8]) -> Value {
    let Ok(mut value) = serde_json::from_slice::<Value>(bytes) else {
        return unavailable_body("invalid_json");
    };
    redact_capture_value(&mut value);
    captured_body(value)
}

fn form_body(bytes: &[u8]) -> Value {
    let mut values = Map::new();
    for (key, value) in parse(bytes) {
        insert_value(&mut values, key.into_owned(), Value::String(value.into_owned()));
    }
    let mut content = Value::Object(values);
    redact_capture_value(&mut content);
    captured_body(content)
}

fn captured_body(content: Value) -> Value {
    Value::Object(Map::from_iter([("captured".into(), Value::Bool(true)), ("content".into(), content)]))
}

fn unavailable_body(reason: &'static str) -> Value {
    Value::Object(Map::from_iter([
        ("captured".into(), Value::Bool(false)),
        ("reason".into(), Value::String(reason.into())),
    ]))
}

fn insert_value(values: &mut Map<String, Value>, key: String, value: Value) {
    let Some(existing) = values.get_mut(&key) else {
        values.insert(key, value);
        return;
    };
    match existing {
        Value::Array(items) => items.push(value),
        item => {
            let first = std::mem::replace(item, Value::Null);
            *item = Value::Array(vec![first, value]);
        }
    }
}

fn redact_capture_value(value: &mut Value) {
    match value {
        Value::Object(object) => redact_capture_object(object),
        Value::Array(values) => values.iter_mut().for_each(redact_capture_value),
        _ => {}
    }
}

fn redact_capture_object(object: &mut Map<String, Value>) {
    redact_config_value(object);
    for (key, value) in object {
        redact_capture_field(key, value);
    }
}

fn redact_capture_field(key: &str, value: &mut Value) {
    if body_field(key) {
        *value = Value::String(BODY_OMITTED.into());
        return;
    }
    if url_field(key) {
        *value = Value::String(redact_url(value).unwrap_or_else(|| URL_OMITTED.into()));
        return;
    }
    if !redact_sensitive_field(key, value) {
        redact_capture_value(value);
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
        redact_capture_value(value);
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
    redact_capture_value(&mut nested);
    *value = Value::String(nested.to_string());
}

fn body_field(key: &str) -> bool {
    normalize_key(key) == "body"
}

fn url_field(key: &str) -> bool {
    matches!(normalize_key(key).as_str(), "url" | "uri" | "endpoint")
}

fn redact_url(value: &Value) -> Option<String> {
    let mut url = Url::parse(value.as_str()?).ok()?;
    url.set_username("").ok()?;
    url.set_password(None).ok()?;
    url.set_fragment(None);
    let query = url.query().map(redact_query);
    url.set_query(query.as_deref());
    Some(url.into())
}

fn redact_query(value: &str) -> String {
    let mut serializer = Serializer::new(String::new());
    for (key, value) in parse(value.as_bytes()) {
        let value = if is_sensitive_key(&key) { REDACTED.into() } else { value.into_owned() };
        serializer.append_pair(&key, &value);
    }
    serializer.finish()
}
