mod fields;
mod http_capture;
mod http_log;
mod infrastructure;
mod macros;
mod metrics;
mod runtime_config;
mod runtime_filter;
mod runtime_state;
mod system_log;
mod tracing_init;

#[doc(hidden)]
pub use tracing as __tracing;

pub use fields::{DurationMs, error, error_with_fields_impl, info_with_fields_impl, safe_error_value, safe_field_value, warn_with_fields_impl};
pub use http_log::{HttpLogCaptureState, http_log_middleware};
pub use infrastructure::{InfrastructureDependency, InfrastructureObserver};
pub use metrics::{MetricsConfig, MetricsError, MetricsHandle, db_query_metric, init_metrics, metrics_handler, metrics_middleware};
pub use runtime_config::{
    DEFAULT_HTTP_BODY_CAPTURE_BYTES, HttpLogCaptureConfig, MAX_HTTP_BODY_CAPTURE_BYTES, RuntimeTracingConfig, RuntimeTracingConfigError,
    SlowOperationThresholds, TracingLevel, parse_runtime_tracing_config,
};
pub use runtime_state::RuntimeTracingState;
pub use system_log::{
    SYSTEM_LOG_BATCH_SIZE, SYSTEM_LOG_CHANNEL_CAPACITY, SYSTEM_LOG_EVENT_MAX_BYTES, SYSTEM_LOG_FLUSH_INTERVAL, SYSTEM_LOG_SHUTDOWN_DRAIN_TIMEOUT,
    SystemLogEmitter, SystemLogEvent, SystemLogEventInput, SystemLogIngestionStatus, SystemLogLayer, SystemLogLevel, SystemLogRuntime, SystemLogSink,
    SystemLogWriteFailure, start_system_log_runtime, start_system_log_runtime_with_state,
};
pub use tracing_init::{ReloadableTracing, TracingInitError, init_global_subscriber};

#[cfg(test)]
mod tests;
