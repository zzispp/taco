use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use axum::{
    body::Body,
    http::{HeaderMap, Uri, header::CONTENT_TYPE},
};
use bytes::Bytes;
use http_body::{Body as HttpBody, Frame, SizeHint};
use serde_json::{Map, Value};
use url::{
    Url,
    form_urlencoded::{Serializer, parse},
};

use kernel::redaction::{REDACTED, is_sensitive_key, normalize_key, redact_sensitive_field};

const BODY_OMITTED: &str = "[body omitted]";
const URL_OMITTED: &str = "[URL omitted]";
const CONFIG_KEY_FIELD: &str = "config_key";
const CONFIG_VALUE_FIELD: &str = "config_value";

#[derive(Clone)]
pub(crate) struct SharedBodyCapture(Arc<Mutex<BodyCaptureState>>);

#[derive(Default)]
struct BodyCaptureState {
    bytes: Vec<u8>,
    truncated: bool,
    stream_error: bool,
    complete: bool,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BodyCaptureSnapshot {
    pub bytes: Vec<u8>,
    pub truncated: bool,
    pub stream_error: bool,
}

pub(crate) struct BodyCaptureOptions {
    pub(crate) limit: usize,
    pub(crate) on_complete: Option<Box<dyn FnOnce(BodyCaptureSnapshot) + Send>>,
}

impl SharedBodyCapture {
    pub(crate) fn new() -> Self {
        Self(Arc::new(Mutex::new(BodyCaptureState::default())))
    }

    pub(crate) fn snapshot(&self) -> BodyCaptureSnapshot {
        let Ok(state) = self.0.lock() else {
            return BodyCaptureSnapshot {
                stream_error: true,
                ..Default::default()
            };
        };
        BodyCaptureSnapshot {
            bytes: state.bytes.clone(),
            truncated: state.truncated,
            stream_error: state.stream_error || !state.complete,
        }
    }

    fn record_frame(&self, frame: &Frame<Bytes>, limit: usize) {
        let Some(data) = frame.data_ref() else {
            return;
        };
        let Ok(mut state) = self.0.lock() else {
            return;
        };
        let remaining = limit.saturating_sub(state.bytes.len());
        let bytes = data.as_ref();
        let take = remaining.min(bytes.len());
        state.bytes.extend_from_slice(&bytes[..take]);
        state.truncated |= take < bytes.len();
    }

    fn finish(&self, stream_error: bool) {
        let Ok(mut state) = self.0.lock() else {
            return;
        };
        state.stream_error |= stream_error;
        state.complete = !stream_error;
    }
}

pub(crate) fn wrap_body(body: Body, capture: SharedBodyCapture, options: BodyCaptureOptions) -> Body {
    Body::new(CapturingBody {
        inner: Box::pin(body),
        capture,
        limit: options.limit,
        on_complete: options.on_complete,
        completed: false,
    })
}

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
    let content_type = content_type.unwrap_or_default().to_ascii_lowercase();
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
    Value::Object(Map::from_iter([("captured".into(), Value::Bool(true)), ("content".into(), value)]))
}

fn form_body(bytes: &[u8]) -> Value {
    let mut values = Map::new();
    for (key, value) in parse(bytes) {
        insert_value(&mut values, key.into_owned(), Value::String(value.into_owned()));
    }
    let mut content = Value::Object(values);
    redact_capture_value(&mut content);
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

struct CapturingBody {
    inner: Pin<Box<Body>>,
    capture: SharedBodyCapture,
    limit: usize,
    on_complete: Option<Box<dyn FnOnce(BodyCaptureSnapshot) + Send>>,
    completed: bool,
}

impl CapturingBody {
    fn finish(&mut self, stream_error: bool) {
        if self.completed {
            return;
        }
        self.capture.finish(stream_error);
        self.completed = true;
        if let Some(callback) = self.on_complete.take() {
            callback(self.capture.snapshot());
        }
    }
}

impl Drop for CapturingBody {
    fn drop(&mut self) {
        self.finish(!self.inner.is_end_stream());
    }
}

impl HttpBody for CapturingBody {
    type Data = Bytes;
    type Error = axum::Error;

    fn poll_frame(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.get_mut();
        match this.inner.as_mut().poll_frame(context) {
            Poll::Ready(Some(Ok(frame))) => {
                this.capture.record_frame(&frame, this.limit);
                Poll::Ready(Some(Ok(frame)))
            }
            Poll::Ready(Some(Err(error))) => {
                this.finish(true);
                Poll::Ready(Some(Err(error)))
            }
            Poll::Ready(None) => {
                this.finish(false);
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

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
