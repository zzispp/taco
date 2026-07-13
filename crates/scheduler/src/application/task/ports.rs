use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::TaskExecutionFailure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutboundHttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutboundHttpHeader {
    pub name: String,
    pub value: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutboundHttpResponseHead {
    pub status: u16,
    pub headers: Vec<OutboundHttpHeader>,
    pub final_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutboundHttpResponse {
    pub head: OutboundHttpResponseHead,
    pub body: Vec<u8>,
    pub duration: Duration,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HttpFailureCode {
    RequestBuild,
    Timeout,
    Connect,
    Request,
    ResponseBody,
    HttpStatus,
}

impl HttpFailureCode {
    pub const fn code(self) -> &'static str {
        match self {
            Self::RequestBuild => "request_build",
            Self::Timeout => "timeout",
            Self::Connect => "connect",
            Self::Request => "request",
            Self::ResponseBody => "response_body",
            Self::HttpStatus => "http_status",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutboundHttpFailure {
    pub code: HttpFailureCode,
    pub duration: Duration,
    pub response: Option<OutboundHttpResponseHead>,
}

#[async_trait]
pub trait HttpTaskClient: Send + Sync + 'static {
    async fn send(&self, request: OutboundHttpRequest) -> Result<OutboundHttpResponse, OutboundHttpFailure>;
}

#[async_trait]
pub trait SystemCacheRefreshPort: Send + Sync + 'static {
    async fn refresh_config_cache(&self) -> Result<(), TaskExecutionFailure>;
    async fn refresh_dict_cache(&self) -> Result<(), TaskExecutionFailure>;
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::HttpFailureCode;

    #[test]
    fn http_failure_codes_have_stable_wire_values() {
        let cases = [
            (HttpFailureCode::RequestBuild, "request_build"),
            (HttpFailureCode::Timeout, "timeout"),
            (HttpFailureCode::Connect, "connect"),
            (HttpFailureCode::Request, "request"),
            (HttpFailureCode::ResponseBody, "response_body"),
            (HttpFailureCode::HttpStatus, "http_status"),
        ];

        for (code, expected) in cases {
            assert_eq!(code.code(), expected);
            assert_eq!(serde_json::to_value(code).unwrap(), json!(expected));
        }
    }
}
