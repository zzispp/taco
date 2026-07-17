use std::{error::Error, fmt};

use serde::Deserialize;

pub const DEFAULT_HTTP_BODY_CAPTURE_BYTES: u64 = 16 * 1024;
pub const MAX_HTTP_BODY_CAPTURE_BYTES: u64 = 64 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TracingLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl TracingLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }

    pub const fn priority(self) -> u8 {
        match self {
            Self::Trace => 0,
            Self::Debug => 1,
            Self::Info => 2,
            Self::Warn => 3,
            Self::Error => 4,
        }
    }

    pub fn allows(self, level: &tracing::Level) -> bool {
        self.priority() <= tracing_level_priority(level)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeTracingConfig {
    pub log_level: TracingLevel,
    pub http: HttpLogCaptureConfig,
    pub slow_operation_ms: SlowOperationThresholds,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct HttpLogCaptureConfig {
    pub access_enabled: bool,
    pub capture_request_body: bool,
    pub capture_response_body: bool,
    pub capture_query_parameters: bool,
    pub capture_request_headers: bool,
    pub max_body_capture_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SlowOperationThresholds {
    pub postgres: u64,
    pub redis: u64,
    pub outbound_http: u64,
}

fn tracing_level_priority(level: &tracing::Level) -> u8 {
    match *level {
        tracing::Level::TRACE => 0,
        tracing::Level::DEBUG => 1,
        tracing::Level::INFO => 2,
        tracing::Level::WARN => 3,
        tracing::Level::ERROR => 4,
    }
}

#[derive(Debug)]
pub enum RuntimeTracingConfigError {
    InvalidJson(serde_json::Error),
    BodyCaptureLimitExceeded,
    NonPositiveSlowOperationThreshold,
}

impl fmt::Display for RuntimeTracingConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(_) => formatter.write_str("runtime tracing configuration must be valid JSON"),
            Self::BodyCaptureLimitExceeded => write!(formatter, "HTTP body capture limit must not exceed {MAX_HTTP_BODY_CAPTURE_BYTES} bytes"),
            Self::NonPositiveSlowOperationThreshold => formatter.write_str("slow operation thresholds must be positive"),
        }
    }
}

impl Error for RuntimeTracingConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidJson(error) => Some(error),
            Self::BodyCaptureLimitExceeded | Self::NonPositiveSlowOperationThreshold => None,
        }
    }
}

pub fn parse_runtime_tracing_config(value: &str) -> Result<RuntimeTracingConfig, RuntimeTracingConfigError> {
    let config = serde_json::from_str(value).map_err(RuntimeTracingConfigError::InvalidJson)?;
    validate_runtime_tracing_config(config)
}

fn validate_runtime_tracing_config(config: RuntimeTracingConfig) -> Result<RuntimeTracingConfig, RuntimeTracingConfigError> {
    if config.http.max_body_capture_bytes > MAX_HTTP_BODY_CAPTURE_BYTES {
        return Err(RuntimeTracingConfigError::BodyCaptureLimitExceeded);
    }
    let thresholds = &config.slow_operation_ms;
    if [thresholds.postgres, thresholds.redis, thresholds.outbound_http]
        .into_iter()
        .any(|value| value == 0)
    {
        return Err(RuntimeTracingConfigError::NonPositiveSlowOperationThreshold);
    }
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_HTTP_BODY_CAPTURE_BYTES, MAX_HTTP_BODY_CAPTURE_BYTES, RuntimeTracingConfigError, TracingLevel, parse_runtime_tracing_config};

    #[test]
    fn parses_the_accepted_runtime_configuration_shape() {
        let config = parse_runtime_tracing_config(&format!(
            r#"{{"log_level":"info","http":{{"access_enabled":true,"capture_request_body":false,"capture_response_body":false,"capture_query_parameters":true,"capture_request_headers":false,"max_body_capture_bytes":{DEFAULT_HTTP_BODY_CAPTURE_BYTES}}},"slow_operation_ms":{{"postgres":500,"redis":100,"outbound_http":1000}}}}"#
        ))
        .unwrap();

        assert_eq!(config.log_level, TracingLevel::Info);
        assert_eq!(config.http.max_body_capture_bytes, DEFAULT_HTTP_BODY_CAPTURE_BYTES);
        assert_eq!(config.slow_operation_ms.redis, 100);
    }

    #[test]
    fn rejects_invalid_runtime_configuration_values() {
        let over_limit = format!(
            r#"{{"log_level":"info","http":{{"access_enabled":true,"capture_request_body":false,"capture_response_body":false,"capture_query_parameters":true,"capture_request_headers":false,"max_body_capture_bytes":{}}},"slow_operation_ms":{{"postgres":500,"redis":100,"outbound_http":1000}}}}"#,
            MAX_HTTP_BODY_CAPTURE_BYTES + 1
        );
        let zero_threshold = r#"{"log_level":"info","http":{"access_enabled":true,"capture_request_body":false,"capture_response_body":false,"capture_query_parameters":true,"capture_request_headers":false,"max_body_capture_bytes":0},"slow_operation_ms":{"postgres":0,"redis":100,"outbound_http":1000}}"#;

        assert!(matches!(
            parse_runtime_tracing_config(&over_limit),
            Err(RuntimeTracingConfigError::BodyCaptureLimitExceeded)
        ));
        assert!(matches!(
            parse_runtime_tracing_config(zero_threshold),
            Err(RuntimeTracingConfigError::NonPositiveSlowOperationThreshold)
        ));
    }
}
