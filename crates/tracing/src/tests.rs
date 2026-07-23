use std::{
    io::{self, Write},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use tracing_subscriber::{Layer, Registry, fmt::MakeWriter, layer::Context, layer::SubscriberExt};

use super::runtime_filter::RuntimeLevelFilter;
use super::{
    DurationMs, HttpLogCaptureConfig, MetricsConfig, RuntimeTracingConfig, RuntimeTracingState, SlowOperationThresholds, SystemLogEvent, SystemLogLayer,
    SystemLogSink, TracingLevel, init_metrics, parse_runtime_tracing_config, start_system_log_runtime_with_state,
};

#[test]
fn duration_ms_formats_milliseconds() {
    assert_eq!(DurationMs(Duration::from_millis(42)).to_string(), "42ms");
}

#[test]
fn runtime_tracing_config_rejects_unknown_log_levels() {
    let result = parse_runtime_tracing_config(
        r#"{"log_level":"invalid","http":{"access_enabled":true,"capture_request_body":false,"capture_response_body":false,"capture_query_parameters":true,"capture_request_headers":false,"max_body_capture_bytes":0},"slow_operation_ms":{"postgres":500,"redis":100,"outbound_http":1000}}"#,
    );
    assert!(result.is_err());
}

#[test]
fn metrics_can_be_disabled() {
    let handle = init_metrics(MetricsConfig { enabled: false }).unwrap();

    assert!(handle.is_none());
}

#[test]
fn structured_macros_redact_values_before_the_stdout_layer() {
    let output = CapturedOutput::default();
    let subscriber = Registry::default().with(tracing_subscriber::fmt::layer().without_time().with_ansi(false).with_writer(output.clone()));
    let guard = tracing::subscriber::set_default(subscriber);

    crate::info_with_fields!(
        "request completed",
        password = "top-secret-value",
        endpoint = "https://login-user:credential-pass@example.com/run?token=query-token-value#url-fragment"
    );
    drop(guard);

    let rendered = output.text();
    assert!(rendered.contains("password=***"));
    assert!(rendered.contains("endpoint=https://example.com/run?token=***"));
    for secret in ["top-secret-value", "login-user", "credential-pass", "query-token-value", "url-fragment"] {
        assert!(!rendered.contains(secret));
    }
}

#[tokio::test]
async fn error_macro_redacts_raw_error_before_stdout_and_persistence() {
    let output = CapturedOutput::default();
    let sink = Arc::new(CollectingSink::default());
    let runtime = start_system_log_runtime_with_state(sink.clone(), RuntimeTracingState::new(runtime_config(TracingLevel::Trace)));
    let subscriber = Registry::default()
        .with(tracing_subscriber::fmt::layer().without_time().with_ansi(false).with_writer(output.clone()))
        .with(SystemLogLayer::new(runtime.emitter()));
    let error = std::io::Error::other(
        "request failed https://login-user:credential-pass@example.com/run?token=query-token-value#url-fragment password=top-secret-value",
    );

    tracing::subscriber::with_default(subscriber, || {
        crate::error_with_fields!("outbound request failed", &error, component = "test");
    });
    wait_for_system_log(&sink).await;

    let persisted = sink.events()[0].fields.to_string();
    let stdout = output.text();
    for secret in ["login-user", "credential-pass", "query-token-value", "url-fragment", "top-secret-value"] {
        assert!(!stdout.contains(secret), "stdout leaked {secret}");
        assert!(!persisted.contains(secret), "persistence leaked {secret}");
    }
    assert!(stdout.contains("***"));
    assert!(persisted.contains("***"));
}

#[tokio::test]
async fn runtime_reload_updates_stdout_filter_and_system_log_threshold_together() {
    let state = RuntimeTracingState::new(runtime_config(TracingLevel::Info));
    let sink = Arc::new(CollectingSink::default());
    let runtime = start_system_log_runtime_with_state(sink.clone(), state.clone());
    let stdout_events = Arc::new(AtomicUsize::new(0));
    let subscriber = Registry::default()
        .with(RuntimeLevelFilter::new(state.clone()))
        .with(EventCounter(stdout_events.clone()))
        .with(SystemLogLayer::new(runtime.emitter()));

    tracing::subscriber::with_default(subscriber, || {
        tracing::debug!(target: "test::runtime", __taco_system_log = true, message = "before reload");
        state.reload(runtime_config(TracingLevel::Debug));
        crate::debug_with_fields!("after reload", request_id = "request-1");
    });
    wait_for_system_log(&sink).await;

    assert_eq!(stdout_events.load(Ordering::Relaxed), 1);
    assert_eq!(sink.events().len(), 1);
    assert_eq!(sink.events()[0].message, "after reload");
    assert_eq!(sink.events()[0].target, "taco_tracing::tests");
}

#[test]
fn http_capture_uses_the_shared_runtime_config_snapshot() {
    let state = RuntimeTracingState::new(runtime_config(TracingLevel::Info));
    let capture = super::HttpLogCaptureState::from_runtime_state(state.clone());
    assert!(capture.config_for_test().access_enabled);

    let mut updated = runtime_config(TracingLevel::Debug);
    updated.http.access_enabled = false;
    state.reload(updated);

    assert!(!capture.config_for_test().access_enabled);
}

#[tokio::test]
async fn infrastructure_observer_logs_only_failed_or_slow_operations_with_safe_fields() {
    let observer = super::InfrastructureObserver::new(RuntimeTracingState::new(runtime_config(TracingLevel::Info)));
    let sink = Arc::new(CollectingSink::default());
    let runtime = start_system_log_runtime_with_state(sink.clone(), RuntimeTracingState::new(runtime_config(TracingLevel::Trace)));
    let subscriber = Registry::default().with(SystemLogLayer::new(runtime.emitter()));

    tracing::subscriber::with_default(subscriber, || {
        observer.record(super::InfrastructureDependency::OutboundHttp, "safe_operation", Duration::ZERO, true);
        observer.record(super::InfrastructureDependency::Redis, "slow_cache_read", Duration::from_millis(100), true);
        observer.record(super::InfrastructureDependency::Postgres, "failed_query", Duration::from_millis(1), false);
    });
    wait_for_system_logs(&sink, 2).await;

    let events = sink.events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].fields["dependency"], "redis");
    assert_eq!(events[0].fields["operation"], "slow_cache_read");
    assert_eq!(events[1].fields["dependency"], "postgres");
    assert_eq!(events[1].fields["operation"], "failed_query");
    assert!(events.iter().all(|event| !event.fields.to_string().contains("safe_operation")));
}

fn runtime_config(log_level: TracingLevel) -> RuntimeTracingConfig {
    RuntimeTracingConfig {
        log_level,
        http: HttpLogCaptureConfig {
            access_enabled: true,
            capture_request_body: false,
            capture_response_body: false,
            capture_query_parameters: false,
            capture_request_headers: false,
            max_body_capture_bytes: 0,
        },
        slow_operation_ms: SlowOperationThresholds {
            postgres: 500,
            redis: 100,
            outbound_http: 1_000,
        },
    }
}

async fn wait_for_system_log(sink: &CollectingSink) {
    wait_for_system_logs(sink, 1).await;
}

async fn wait_for_system_logs(sink: &CollectingSink, expected: usize) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while sink.events().len() < expected {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("system log writer did not persist the event");
}

#[derive(Clone, Default)]
struct CapturedOutput(Arc<Mutex<Vec<u8>>>);

impl CapturedOutput {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

impl<'a> MakeWriter<'a> for CapturedOutput {
    type Writer = CapturedWriter;

    fn make_writer(&'a self) -> Self::Writer {
        CapturedWriter(self.0.clone())
    }
}

struct CapturedWriter(Arc<Mutex<Vec<u8>>>);

impl Write for CapturedWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct EventCounter(Arc<AtomicUsize>);

impl<S> Layer<S> for EventCounter
where
    S: tracing::Subscriber,
{
    fn on_event(&self, _: &tracing::Event<'_>, _: Context<'_, S>) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Default)]
struct CollectingSink(Mutex<Vec<SystemLogEvent>>);

impl CollectingSink {
    fn events(&self) -> Vec<SystemLogEvent> {
        self.0.lock().unwrap().clone()
    }
}

#[async_trait::async_trait]
impl SystemLogSink for CollectingSink {
    async fn insert_batch(&self, events: Vec<SystemLogEvent>) -> Result<(), String> {
        self.0.lock().unwrap().extend(events);
        Ok(())
    }
}
