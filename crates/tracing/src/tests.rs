use super::{DurationMs, MetricsConfig, TracingConfig, init_global_subscriber, init_metrics};
use std::time::Duration;

#[test]
fn duration_ms_formats_milliseconds() {
    assert_eq!(DurationMs(Duration::from_millis(42)).to_string(), "42ms");
}

#[test]
fn tracing_config_rejects_invalid_filter_syntax() {
    let config = TracingConfig {
        log_level: "[".into(),
        file_logging_enabled: false,
        file_directory: String::new(),
        file_prefix: String::new(),
    };

    let result = init_global_subscriber(config);

    assert!(result.is_err());
}

#[test]
fn metrics_can_be_disabled() {
    let handle = init_metrics(MetricsConfig { enabled: false }).unwrap();

    assert!(handle.is_none());
}
