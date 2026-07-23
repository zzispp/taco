use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use async_trait::async_trait;
use metrics::set_default_local_recorder;
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::sync::Notify;

use super::{SYSTEM_LOG_BATCH_SIZE, SystemLogEvent, SystemLogEventInput, SystemLogLevel, SystemLogSink, start_system_log_runtime};

const QUEUED_AFTER_IN_FLIGHT: usize = 3;

#[tokio::test]
async fn writer_panic_reconciles_in_flight_and_queued_events() {
    let recorder = PrometheusBuilder::new().build_recorder();
    let handle = recorder.handle();
    let _guard = set_default_local_recorder(&recorder);
    let sink = Arc::new(BlockingPanickingSink::default());
    let runtime = start_system_log_runtime(sink.clone(), SystemLogLevel::Trace);
    for index in 0..SYSTEM_LOG_BATCH_SIZE {
        runtime.emitter().emit(event(&format!("in-flight-{index}")));
    }
    sink.started.notified().await;
    for index in 0..QUEUED_AFTER_IN_FLIGHT {
        runtime.emitter().emit(event(&format!("queued-{index}")));
    }

    sink.release.notify_one();
    wait_for_writer_stop(&runtime).await;

    let status = runtime.status();
    assert!(!status.writer_running);
    assert!(!status.writer_healthy);
    assert_eq!(status.queue_depth, 0);
    assert_eq!(status.pending_events, 0);
    assert_eq!(status.persisted_events, 0);
    let expected_dropped = u64::try_from(SYSTEM_LOG_BATCH_SIZE + QUEUED_AFTER_IN_FLIGHT).unwrap();
    assert_eq!(status.dropped_events, expected_dropped);
    assert_eq!(
        status.latest_write_failure.as_ref().map(|failure| failure.failed_events),
        Some(expected_dropped)
    );
    assert_eq!(status.latest_write_failure.as_ref().map(|failure| failure.reason), Some("writer_terminated"));
    assert_metric(&handle.render(), "system_log_dropped_total{reason=\"writer_terminated\"}", expected_dropped);
    runtime.shutdown().await;
    let after_shutdown = runtime.status();
    assert_eq!(
        after_shutdown.latest_write_failure.as_ref().map(|failure| failure.failed_events),
        Some(expected_dropped)
    );
    assert_eq!(
        after_shutdown.latest_write_failure.as_ref().map(|failure| failure.reason),
        Some("writer_terminated")
    );
}

#[tokio::test]
async fn shutdown_gate_rejects_events_before_the_writer_finishes() {
    let recorder = PrometheusBuilder::new().build_recorder();
    let handle = recorder.handle();
    let _guard = set_default_local_recorder(&recorder);
    let sink = Arc::new(BlockingSuccessfulSink::default());
    let runtime = Arc::new(start_system_log_runtime(sink.clone(), SystemLogLevel::Trace));
    for index in 0..SYSTEM_LOG_BATCH_SIZE {
        runtime.emitter().emit(event(&format!("in-flight-{index}")));
    }
    sink.started.notified().await;

    let shutdown_runtime = runtime.clone();
    let shutdown = tokio::spawn(async move { shutdown_runtime.shutdown().await });
    tokio::task::yield_now().await;
    runtime.emitter().emit(event("after-shutdown-request"));

    let stopping = runtime.status();
    assert_eq!(stopping.pending_events, SYSTEM_LOG_BATCH_SIZE as u64);
    assert_eq!(stopping.queue_depth, 0);
    assert_eq!(stopping.dropped_events, 1);
    assert_metric(&handle.render(), "system_log_dropped_total{reason=\"runtime_stopping\"}", 1);

    sink.release.notify_one();
    shutdown.await.unwrap();
    let stopped = runtime.status();
    assert_eq!(stopped.pending_events, 0);
    assert_eq!(stopped.persisted_events, SYSTEM_LOG_BATCH_SIZE as u64);
    assert_eq!(stopped.dropped_events, 1);
    assert!(stopped.writer_healthy);
}

async fn wait_for_writer_stop(runtime: &super::SystemLogRuntime) {
    tokio::time::timeout(std::time::Duration::from_secs(1), async {
        while runtime.status().writer_running {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("writer did not stop after sink panic");
}

fn event(message: &str) -> SystemLogEvent {
    SystemLogEvent::new(SystemLogEventInput {
        occurred_at: time::OffsetDateTime::now_utc(),
        level: SystemLogLevel::Info,
        target: "test::writer_exit".into(),
        message: message.into(),
        fields: serde_json::Map::new(),
    })
}

fn assert_metric(rendered: &str, prefix: &str, expected: u64) {
    let actual = rendered
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap_or_else(|| panic!("metric `{prefix}` missing from:\n{rendered}"));
    assert_eq!(actual, format!("{prefix} {expected}"));
}

#[derive(Default)]
struct BlockingPanickingSink {
    first: AtomicBool,
    started: Notify,
    release: Notify,
}

#[async_trait]
impl SystemLogSink for BlockingPanickingSink {
    async fn insert_batch(&self, _: Vec<SystemLogEvent>) -> Result<(), String> {
        if !self.first.swap(true, Ordering::Relaxed) {
            self.started.notify_one();
            self.release.notified().await;
            panic!("intentional sink panic");
        }
        Ok(())
    }
}

#[derive(Default)]
struct BlockingSuccessfulSink {
    started: Notify,
    release: Notify,
}

#[async_trait]
impl SystemLogSink for BlockingSuccessfulSink {
    async fn insert_batch(&self, _: Vec<SystemLogEvent>) -> Result<(), String> {
        self.started.notify_one();
        self.release.notified().await;
        Ok(())
    }
}
