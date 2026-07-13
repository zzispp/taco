use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::Serialize;

use crate::application::task::{
    HttpFailureCode, OutboundHttpFailure, OutboundHttpHeader, OutboundHttpRequest, OutboundHttpResponse, OutboundHttpResponseHead, TaskExecutionDetailPayload,
};

const HTTP_EXECUTION_DETAIL_KIND: &str = "http_exchange";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct HttpExecutionReport {
    duration_ms: u64,
    request: HttpRequestReport,
    response: Option<HttpResponseReport>,
    failure: Option<HttpFailureReport>,
}

impl HttpExecutionReport {
    pub fn from_response(request: HttpRequestReport, response: OutboundHttpResponse, failure: Option<HttpFailureCode>) -> Self {
        Self {
            duration_ms: duration_ms(response.duration),
            request,
            response: Some(HttpResponseReport::complete(response.head, response.body)),
            failure: failure.map(HttpFailureReport::new),
        }
    }

    pub fn from_failure(request: HttpRequestReport, failure: OutboundHttpFailure) -> Self {
        Self {
            duration_ms: duration_ms(failure.duration),
            request,
            response: failure.response.map(HttpResponseReport::incomplete),
            failure: Some(HttpFailureReport::new(failure.code)),
        }
    }
}

impl TaskExecutionDetailPayload for HttpExecutionReport {
    const KIND: &'static str = HTTP_EXECUTION_DETAIL_KIND;
    const SCHEMA_VERSION: i16 = 1;
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct HttpRequestReport {
    method: String,
    url: String,
    headers: Vec<HttpHeaderReport>,
    body: Option<EncodedBytes>,
}

impl From<&OutboundHttpRequest> for HttpRequestReport {
    fn from(request: &OutboundHttpRequest) -> Self {
        Self {
            method: request.method.clone(),
            url: request.url.clone(),
            headers: request
                .headers
                .iter()
                .map(|(name, value)| HttpHeaderReport::new(name.clone(), value.as_bytes()))
                .collect(),
            body: request.body.as_deref().map(|body| EncodedBytes::new(body.as_bytes())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct HttpResponseReport {
    status: u16,
    final_url: String,
    headers: Vec<HttpHeaderReport>,
    body: Option<EncodedBytes>,
}

impl HttpResponseReport {
    fn complete(head: OutboundHttpResponseHead, body: Vec<u8>) -> Self {
        Self::new(head, Some(EncodedBytes::new(&body)))
    }

    fn incomplete(head: OutboundHttpResponseHead) -> Self {
        Self::new(head, None)
    }

    fn new(head: OutboundHttpResponseHead, body: Option<EncodedBytes>) -> Self {
        Self {
            status: head.status,
            final_url: head.final_url,
            headers: head.headers.into_iter().map(HttpHeaderReport::from).collect(),
            body,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct HttpHeaderReport {
    name: String,
    value: EncodedBytes,
}

impl HttpHeaderReport {
    fn new(name: String, value: &[u8]) -> Self {
        Self {
            name,
            value: EncodedBytes::new(value),
        }
    }
}

impl From<OutboundHttpHeader> for HttpHeaderReport {
    fn from(header: OutboundHttpHeader) -> Self {
        Self::new(header.name, &header.value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct HttpFailureReport {
    code: HttpFailureCode,
}

impl HttpFailureReport {
    const fn new(code: HttpFailureCode) -> Self {
        Self { code }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum ByteEncoding {
    Utf8,
    Base64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct EncodedBytes {
    encoding: ByteEncoding,
    content: String,
    byte_length: u64,
}

impl EncodedBytes {
    fn new(bytes: &[u8]) -> Self {
        let byte_length = u64::try_from(bytes.len()).expect("captured HTTP value length must fit in u64");
        match jsonb_compatible_utf8(bytes) {
            Some(content) => Self {
                encoding: ByteEncoding::Utf8,
                content: content.to_owned(),
                byte_length,
            },
            None => Self {
                encoding: ByteEncoding::Base64,
                content: STANDARD.encode(bytes),
                byte_length,
            },
        }
    }
}

fn jsonb_compatible_utf8(bytes: &[u8]) -> Option<&str> {
    let content = std::str::from_utf8(bytes).ok()?;
    (!content.contains('\0')).then_some(content)
}

fn duration_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).expect("HTTP execution duration must fit in u64 milliseconds")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::EncodedBytes;

    #[test]
    fn captured_bytes_preserve_utf8_binary_and_empty_values() {
        let utf8_text = "\u{54cd}\u{5e94}";
        let utf8 = serde_json::to_value(EncodedBytes::new(utf8_text.as_bytes())).unwrap();
        let binary = serde_json::to_value(EncodedBytes::new(&[0, 159, 146, 150])).unwrap();
        let nul_utf8 = serde_json::to_value(EncodedBytes::new(b"a\0b")).unwrap();
        let empty = serde_json::to_value(EncodedBytes::new(&[])).unwrap();

        assert_eq!(utf8, json!({"encoding": "utf8", "content": utf8_text, "byte_length": 6}));
        assert_eq!(binary, json!({"encoding": "base64", "content": "AJ+Slg==", "byte_length": 4}));
        assert_eq!(nul_utf8, json!({"encoding": "base64", "content": "YQBi", "byte_length": 3}));
        assert_eq!(empty, json!({"encoding": "utf8", "content": "", "byte_length": 0}));
    }
}
