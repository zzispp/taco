use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::TaskExecutionFailure;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FileTrashCleanupResult {
    pub purged_entries: u64,
    pub blocked_roots: u64,
    pub deleted_objects: u64,
    pub failed_objects: u64,
    pub retried_provider_cleanups: u64,
    pub failed_provider_cleanups: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FileUploadSessionCleanupResult {
    pub expired_sessions: u64,
    pub reconciled_sessions: u64,
    pub retried_provider_cleanups: u64,
    pub failed_provider_cleanups: u64,
}

#[async_trait]
pub trait FileCleanupPort: Send + Sync + 'static {
    async fn purge_trash(&self, retention_days: u64, batch_size: u64) -> Result<FileTrashCleanupResult, TaskExecutionFailure>;
    async fn cleanup_upload_sessions(&self, batch_size: u64) -> Result<FileUploadSessionCleanupResult, TaskExecutionFailure>;
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SystemLogCleanupResult {
    pub deleted: u64,
    pub batches: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemLogCleanupLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl SystemLogCleanupLevel {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "trace" => Some(Self::Trace),
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemLogCleanupFilter {
    pub keyword: Option<String>,
    pub levels: Vec<SystemLogCleanupLevel>,
    pub target: Option<String>,
    pub begin_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
}

/// Executes complete system-log cleanup cycles for scheduler invocations.
#[async_trait]
pub trait SystemLogCleanupPort: Send + Sync + 'static {
    async fn cleanup_expired(&self, retention_days: u64, batch_size: u64) -> Result<SystemLogCleanupResult, TaskExecutionFailure>;
    async fn cleanup_filtered(&self, filter: SystemLogCleanupFilter, batch_size: u64) -> Result<SystemLogCleanupResult, TaskExecutionFailure>;
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
