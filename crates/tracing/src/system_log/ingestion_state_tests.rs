use std::{
    sync::{Arc, Barrier, mpsc},
    time::Duration,
};

use metrics::set_default_local_recorder;
use metrics_exporter_prometheus::PrometheusBuilder;

use super::ingestion_state::IngestionState;

const TEST_QUEUE_CAPACITY: usize = 64;
const PRODUCER_COUNT: usize = 8;

#[test]
fn enqueue_accounting_is_atomic_with_forced_discard() {
    let state = Arc::new(IngestionState::new(TEST_QUEUE_CAPACITY));
    let (enqueue_started_tx, enqueue_started_rx) = mpsc::channel();
    let (release_enqueue_tx, release_enqueue_rx) = mpsc::channel();
    let (discard_started_tx, discard_started_rx) = mpsc::channel();
    let (discard_done_tx, discard_done_rx) = mpsc::channel();

    std::thread::scope(|scope| {
        let producer_state = state.clone();
        scope.spawn(move || {
            producer_state.record_enqueue(|| {
                enqueue_started_tx.send(()).unwrap();
                release_enqueue_rx.recv().unwrap();
                Ok(())
            });
        });
        enqueue_started_rx.recv().unwrap();

        let discard_state = state.clone();
        scope.spawn(move || {
            discard_started_tx.send(()).unwrap();
            discard_state.discard_all_pending("shutdown_timeout");
            discard_done_tx.send(()).unwrap();
        });
        discard_started_rx.recv().unwrap();
        assert_eq!(discard_done_rx.recv_timeout(Duration::from_millis(20)), Err(mpsc::RecvTimeoutError::Timeout));

        release_enqueue_tx.send(()).unwrap();
        discard_done_rx.recv().unwrap();
    });

    let status = state.status();
    assert_eq!(status.queue_depth, 0);
    assert_eq!(status.pending_events, 0);
    assert_eq!(status.dropped_events, 1);
}

#[test]
fn concurrent_enqueue_metrics_match_the_serialized_state() {
    let recorder = PrometheusBuilder::new().build_recorder();
    let handle = recorder.handle();
    let state = Arc::new(IngestionState::new(TEST_QUEUE_CAPACITY));
    let start = Arc::new(Barrier::new(PRODUCER_COUNT));

    std::thread::scope(|scope| {
        for _ in 0..PRODUCER_COUNT {
            let state = state.clone();
            let start = start.clone();
            let recorder = &recorder;
            scope.spawn(move || {
                let _guard = set_default_local_recorder(recorder);
                start.wait();
                state.record_enqueue(|| Ok(()));
            });
        }
    });

    let status = state.status();
    let metrics = handle.render();
    assert_eq!(status.queue_depth, PRODUCER_COUNT);
    assert_eq!(status.pending_events, PRODUCER_COUNT as u64);
    assert_metric(&metrics, "system_log_queue_depth", PRODUCER_COUNT);
    assert_metric(&metrics, "system_log_pending_events", PRODUCER_COUNT);
}

fn assert_metric(rendered: &str, name: &str, expected: usize) {
    let actual = rendered
        .lines()
        .find(|line| line.starts_with(name))
        .unwrap_or_else(|| panic!("metric `{name}` missing from:\n{rendered}"));
    assert_eq!(actual, format!("{name} {expected}"));
}
