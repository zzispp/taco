use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use serde_json::Map;
use tokio::sync::Notify;
use tracing_subscriber::{
    Layer, Registry,
    layer::{Context, SubscriberExt},
};

use super::{SystemLogEvent, SystemLogEventInput, SystemLogLevel, SystemLogSink, start_system_log_runtime};
use crate::SystemLogLayer;

#[tokio::test]
async fn failed_batch_counts_every_dropped_event_and_retains_failure_state() {
    let runtime = start_system_log_runtime(Arc::new(FailingSink), SystemLogLevel::Trace);
    runtime.emitter().emit(event("one"));
    runtime.emitter().emit(event("two"));

    runtime.shutdown().await;

    let status = runtime.status();
    assert_eq!(status.dropped_events, 2);
    assert_eq!(status.latest_write_failure.as_ref().map(|failure| failure.failed_events), Some(2));
}

#[tokio::test]
async fn shutdown_drains_queued_events_before_returning() {
    let sink = Arc::new(CollectingSink::default());
    let runtime = start_system_log_runtime(sink.clone(), SystemLogLevel::Trace);
    runtime.emitter().emit(event("one"));
    runtime.emitter().emit(event("two"));

    runtime.shutdown().await;

    assert_eq!(sink.events().len(), 2);
    assert_eq!(runtime.status().dropped_events, 0);
}

#[tokio::test]
async fn emitting_after_writer_shutdown_is_dropped_without_pending_work() {
    let runtime = start_system_log_runtime(Arc::new(CollectingSink::default()), SystemLogLevel::Trace);
    runtime.shutdown().await;

    runtime.emitter().emit(event("after-shutdown"));

    let status = runtime.status();
    assert_eq!(status.dropped_events, 1);
    assert_eq!(status.queue_depth, 0);
    assert_eq!(status.pending_events, 0);
}

#[tokio::test]
async fn writer_failure_does_not_enqueue_a_persistence_diagnostic() {
    let sink = Arc::new(CountingFailingSink::default());
    let deferred_layer = DeferredSystemLogLayer::default();
    let subscriber = Registry::default().with(deferred_layer.clone());
    let runtime = tracing::subscriber::with_default(subscriber, || {
        let runtime = start_system_log_runtime(sink.clone(), SystemLogLevel::Trace);
        deferred_layer.install(SystemLogLayer::new(runtime.emitter()));
        crate::info_with_fields!("persist one event", operation = "test");
        runtime
    });
    tokio::time::sleep(Duration::from_millis(350)).await;
    runtime.shutdown().await;

    assert_eq!(sink.calls(), 1);
    assert_eq!(runtime.status().dropped_events, 1);
}

#[tokio::test]
async fn shutdown_has_a_total_deadline_and_counts_the_in_flight_batch() {
    let sink = Arc::new(PendingSink::default());
    let runtime = start_system_log_runtime(sink.clone(), SystemLogLevel::Trace);
    runtime.emitter().emit(event("one"));
    tokio::time::timeout(Duration::from_secs(1), sink.started.notified())
        .await
        .expect("writer never started the pending batch");

    tokio::time::timeout(Duration::from_secs(6), runtime.shutdown())
        .await
        .expect("shutdown exceeded its total deadline");

    let status = runtime.status();
    assert_eq!(status.dropped_events, 1);
    assert!(!status.writer_running);
    assert!(!status.writer_healthy);
    assert_eq!(status.latest_write_failure.as_ref().map(|failure| failure.failed_events), Some(1));
    assert_eq!(status.latest_write_failure.as_ref().map(|failure| failure.reason), Some("shutdown_timeout"));
}

fn event(message: &str) -> SystemLogEvent {
    SystemLogEvent::new(SystemLogEventInput {
        occurred_at: time::OffsetDateTime::now_utc(),
        level: SystemLogLevel::Info,
        target: "test::ingestion".into(),
        message: message.into(),
        fields: Map::new(),
    })
}

struct FailingSink;

#[async_trait]
impl SystemLogSink for FailingSink {
    async fn insert_batch(&self, _: Vec<SystemLogEvent>) -> Result<(), String> {
        Err("unavailable".into())
    }
}

#[derive(Default)]
struct CountingFailingSink(Mutex<u64>);

impl CountingFailingSink {
    fn calls(&self) -> u64 {
        *self.0.lock().unwrap()
    }
}

#[async_trait]
impl SystemLogSink for CountingFailingSink {
    async fn insert_batch(&self, _: Vec<SystemLogEvent>) -> Result<(), String> {
        *self.0.lock().unwrap() += 1;
        Err("unavailable".into())
    }
}

#[derive(Default)]
struct PendingSink {
    started: Notify,
}

#[async_trait]
impl SystemLogSink for PendingSink {
    async fn insert_batch(&self, _: Vec<SystemLogEvent>) -> Result<(), String> {
        self.started.notify_waiters();
        std::future::pending().await
    }
}

#[derive(Clone, Default)]
struct DeferredSystemLogLayer(Arc<Mutex<Option<SystemLogLayer>>>);

impl DeferredSystemLogLayer {
    fn install(&self, layer: SystemLogLayer) {
        *self.0.lock().unwrap() = Some(layer);
    }
}

impl<S> Layer<S> for DeferredSystemLogLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, context: Context<'_, S>) {
        if let Some(layer) = self.0.lock().unwrap().clone() {
            layer.on_event(event, context);
        }
    }
}

#[derive(Default)]
struct CollectingSink(Mutex<Vec<SystemLogEvent>>);

impl CollectingSink {
    fn events(&self) -> Vec<SystemLogEvent> {
        self.0.lock().unwrap().clone()
    }
}

#[async_trait]
impl SystemLogSink for CollectingSink {
    async fn insert_batch(&self, events: Vec<SystemLogEvent>) -> Result<(), String> {
        self.0.lock().unwrap().extend(events);
        Ok(())
    }
}
