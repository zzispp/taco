use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use async_trait::async_trait;
use metrics::set_default_local_recorder;
use metrics_exporter_prometheus::PrometheusBuilder;
use serde_json::Map;
use tokio::sync::Notify;

use super::{
    SYSTEM_LOG_BATCH_SIZE, SYSTEM_LOG_CHANNEL_CAPACITY, SYSTEM_LOG_FLUSH_INTERVAL, SystemLogDeliveryGuarantee, SystemLogEvent, SystemLogEventInput,
    SystemLogLevel, SystemLogSink, start_system_log_runtime,
};

const ONE_MILLISECOND: Duration = Duration::from_millis(1);

#[tokio::test(start_paused = true)]
async fn first_event_after_idle_receives_a_full_flush_window() {
    let sink = Arc::new(BatchCollectingSink::default());
    let runtime = start_system_log_runtime(sink.clone(), SystemLogLevel::Trace);
    tokio::time::advance(SYSTEM_LOG_FLUSH_INTERVAL * 3).await;

    runtime.emitter().emit(event("one"));
    tokio::task::yield_now().await;
    assert_eq!(sink.batches(), Vec::<Vec<String>>::new());

    tokio::time::advance(SYSTEM_LOG_FLUSH_INTERVAL - ONE_MILLISECOND).await;
    tokio::task::yield_now().await;
    assert_eq!(sink.batches(), Vec::<Vec<String>>::new());

    tokio::time::advance(ONE_MILLISECOND).await;
    tokio::task::yield_now().await;
    assert_eq!(sink.batches(), vec![vec!["one".to_owned()]]);

    runtime.shutdown().await;
}

#[tokio::test(start_paused = true)]
async fn later_events_do_not_extend_the_first_event_deadline() {
    let sink = Arc::new(BatchCollectingSink::default());
    let runtime = start_system_log_runtime(sink.clone(), SystemLogLevel::Trace);
    runtime.emitter().emit(event("one"));
    tokio::task::yield_now().await;

    tokio::time::advance(SYSTEM_LOG_FLUSH_INTERVAL - Duration::from_millis(10)).await;
    runtime.emitter().emit(event("two"));
    tokio::task::yield_now().await;
    assert_eq!(sink.batches(), Vec::<Vec<String>>::new());

    tokio::time::advance(Duration::from_millis(10)).await;
    tokio::task::yield_now().await;
    assert_eq!(sink.batches(), vec![vec!["one".to_owned(), "two".to_owned()]]);

    runtime.shutdown().await;
}

#[tokio::test(start_paused = true)]
async fn runtime_status_accounts_for_queue_pending_persisted_and_writer_lifecycle() {
    let runtime = start_system_log_runtime(Arc::new(CollectingSink), SystemLogLevel::Trace);
    runtime.emitter().emit(event("one"));

    let queued = runtime.status();
    assert_eq!(queued.queue_depth, 1);
    assert_eq!(queued.queue_capacity, SYSTEM_LOG_CHANNEL_CAPACITY);
    assert_eq!(queued.pending_events, 1);
    assert_eq!(queued.persisted_events, 0);
    assert_eq!(queued.dropped_events, 0);
    assert!(queued.writer_running);
    assert!(queued.writer_healthy);
    assert_eq!(queued.delivery_guarantee, SystemLogDeliveryGuarantee::BestEffort);

    tokio::task::yield_now().await;
    let buffered = runtime.status();
    assert_eq!(buffered.queue_depth, 0);
    assert_eq!(buffered.pending_events, 1);
    assert_eq!(buffered.persisted_events, 0);

    tokio::time::advance(SYSTEM_LOG_FLUSH_INTERVAL).await;
    tokio::task::yield_now().await;
    let persisted = runtime.status();
    assert_eq!(persisted.queue_depth, 0);
    assert_eq!(persisted.pending_events, 0);
    assert_eq!(persisted.persisted_events, 1);
    assert_eq!(persisted.dropped_events, 0);

    runtime.shutdown().await;
    let stopped = runtime.status();
    assert!(!stopped.writer_running);
    assert_eq!(stopped.pending_events, 0);
}

#[tokio::test(start_paused = true)]
async fn queue_full_drop_preserves_exact_pending_accounting() {
    let sink = Arc::new(FirstBatchBlockingSink::default());
    let runtime = start_system_log_runtime(sink.clone(), SystemLogLevel::Trace);
    for index in 0..SYSTEM_LOG_BATCH_SIZE {
        runtime.emitter().emit(event(&format!("in-flight-{index}")));
    }
    sink.started.notified().await;

    for index in 0..SYSTEM_LOG_CHANNEL_CAPACITY {
        runtime.emitter().emit(event(&format!("queued-{index}")));
    }
    runtime.emitter().emit(event("dropped"));

    let saturated = runtime.status();
    assert_eq!(saturated.queue_depth, SYSTEM_LOG_CHANNEL_CAPACITY);
    assert_eq!(
        saturated.pending_events,
        u64::try_from(SYSTEM_LOG_BATCH_SIZE + SYSTEM_LOG_CHANNEL_CAPACITY).unwrap()
    );
    assert_eq!(saturated.persisted_events, 0);
    assert_eq!(saturated.dropped_events, 1);

    sink.release.notify_one();
    runtime.shutdown().await;
    let drained = runtime.status();
    assert_eq!(drained.queue_depth, 0);
    assert_eq!(drained.pending_events, 0);
    assert_eq!(
        drained.persisted_events,
        u64::try_from(SYSTEM_LOG_BATCH_SIZE + SYSTEM_LOG_CHANNEL_CAPACITY).unwrap()
    );
    assert_eq!(drained.dropped_events, 1);
}

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn ingestion_metrics_use_stable_names_and_labels() {
    let recorder = PrometheusBuilder::new().build_recorder();
    let handle = recorder.handle();
    let _guard = set_default_local_recorder(&recorder);
    let sink = Arc::new(SuccessThenFailureSink::default());
    let runtime = start_system_log_runtime(sink.clone(), SystemLogLevel::Trace);
    runtime.emitter().emit(event("persisted"));
    tokio::task::yield_now().await;
    tokio::time::advance(SYSTEM_LOG_FLUSH_INTERVAL).await;
    tokio::task::yield_now().await;
    runtime.emitter().emit(event("one"));
    tokio::task::yield_now().await;
    tokio::time::advance(SYSTEM_LOG_FLUSH_INTERVAL).await;
    tokio::task::yield_now().await;
    runtime.shutdown().await;

    let metrics = handle.render();
    assert_eq!(sink.calls(), 2);
    assert_metric(&metrics, "system_log_queue_depth", "system_log_queue_depth 0");
    assert_metric(&metrics, "system_log_pending_events", "system_log_pending_events 0");
    assert_metric(&metrics, "system_log_persisted_total", "system_log_persisted_total 1");
    assert_metric(
        &metrics,
        "system_log_dropped_total{reason=\"writer_failed\"}",
        "system_log_dropped_total{reason=\"writer_failed\"} 1",
    );
    assert_metric(&metrics, "system_log_writer_running", "system_log_writer_running 0");
    assert_metric(&metrics, "system_log_writer_healthy", "system_log_writer_healthy 0");
    assert_metric(
        &metrics,
        "system_log_sink_write_duration_seconds_count{outcome=\"failure\"}",
        "system_log_sink_write_duration_seconds_count{outcome=\"failure\"} 1",
    );
    assert_metric(
        &metrics,
        "system_log_sink_write_duration_seconds_count{outcome=\"success\"}",
        "system_log_sink_write_duration_seconds_count{outcome=\"success\"} 1",
    );
}

fn event(message: &str) -> SystemLogEvent {
    SystemLogEvent::new(SystemLogEventInput {
        occurred_at: time::OffsetDateTime::now_utc(),
        level: SystemLogLevel::Info,
        target: "test::runtime".into(),
        message: message.into(),
        fields: Map::new(),
    })
}

fn assert_metric(rendered: &str, prefix: &str, expected: &str) {
    let actual = rendered
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap_or_else(|| panic!("metric `{prefix}` missing from:\n{rendered}"));
    assert_eq!(actual, expected);
}

struct CollectingSink;

#[async_trait]
impl SystemLogSink for CollectingSink {
    async fn insert_batch(&self, _: Vec<SystemLogEvent>) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Default)]
struct SuccessThenFailureSink(AtomicU64);

impl SuccessThenFailureSink {
    fn calls(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl SystemLogSink for SuccessThenFailureSink {
    async fn insert_batch(&self, _: Vec<SystemLogEvent>) -> Result<(), String> {
        if self.0.fetch_add(1, Ordering::Relaxed) == 0 {
            return Ok(());
        }
        Err("unavailable".into())
    }
}

#[derive(Default)]
struct FirstBatchBlockingSink {
    block_first: AtomicBool,
    started: Notify,
    release: Notify,
}

#[async_trait]
impl SystemLogSink for FirstBatchBlockingSink {
    async fn insert_batch(&self, _: Vec<SystemLogEvent>) -> Result<(), String> {
        if !self.block_first.swap(true, Ordering::Relaxed) {
            self.started.notify_one();
            self.release.notified().await;
        }
        Ok(())
    }
}

#[derive(Default)]
struct BatchCollectingSink(Mutex<Vec<Vec<String>>>);

impl BatchCollectingSink {
    fn batches(&self) -> Vec<Vec<String>> {
        self.0.lock().unwrap().clone()
    }
}

#[async_trait]
impl SystemLogSink for BatchCollectingSink {
    async fn insert_batch(&self, events: Vec<SystemLogEvent>) -> Result<(), String> {
        self.0.lock().unwrap().push(events.into_iter().map(|event| event.message).collect());
        Ok(())
    }
}
