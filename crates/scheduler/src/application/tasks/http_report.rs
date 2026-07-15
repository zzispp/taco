use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

use crate::application::task::{
    HttpFailureCode, OutboundHttpFailure, OutboundHttpRequest, OutboundHttpResponse, OutboundHttpResponseHead, TaskExecutionDetailPayload,
};

use super::http_sanitization::{HTTP_EXECUTION_DETAIL_KIND, sanitize_http_method, sanitize_http_url};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct HttpExecutionReport {
    duration_ms: u64,
    request: HttpRequestReport,
    response: Option<HttpResponseReport>,
    failure: Option<HttpFailureReport>,
}

impl HttpExecutionReport {
    pub fn from_response(request: HttpRequestReport, response: OutboundHttpResponse, failure: Option<HttpFailureCode>) -> Self {
        let OutboundHttpResponse { head, duration, .. } = response;
        Self {
            duration_ms: duration_ms(duration),
            request,
            response: Some(HttpResponseReport::complete(head)),
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
    headers: Vec<Value>,
    body: Option<Value>,
}

impl From<&OutboundHttpRequest> for HttpRequestReport {
    fn from(request: &OutboundHttpRequest) -> Self {
        Self {
            method: sanitize_http_method(&request.method),
            url: sanitize_http_url(&request.url),
            headers: Vec::new(),
            body: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct HttpResponseReport {
    status: u16,
    final_url: String,
    headers: Vec<Value>,
    body: Option<Value>,
}

impl HttpResponseReport {
    fn complete(head: OutboundHttpResponseHead) -> Self {
        Self::new(head)
    }

    fn incomplete(head: OutboundHttpResponseHead) -> Self {
        Self::new(head)
    }

    fn new(head: OutboundHttpResponseHead) -> Self {
        Self {
            status: head.status,
            final_url: sanitize_http_url(&head.final_url),
            headers: Vec::new(),
            body: None,
        }
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

fn duration_ms(duration: Duration) -> u64 {
    u64::try_from(duration.as_millis()).expect("HTTP execution duration must fit in u64 milliseconds")
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::duration_ms;

    #[test]
    fn duration_conversion_retains_milliseconds() {
        assert_eq!(duration_ms(Duration::from_millis(12)), 12);
    }
}
