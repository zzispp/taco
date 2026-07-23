use std::sync::{Mutex, MutexGuard};

use metrics::{counter, gauge};

const DROP_METRIC: &str = "system_log_dropped_total";
const PENDING_METRIC: &str = "system_log_pending_events";
const PERSISTED_METRIC: &str = "system_log_persisted_total";
const QUEUE_DEPTH_METRIC: &str = "system_log_queue_depth";
const RUNTIME_STOPPING_DROP_REASON: &str = "runtime_stopping";
const SHUTDOWN_TIMEOUT_DROP_REASON: &str = "shutdown_timeout";
const WRITER_HEALTH_METRIC: &str = "system_log_writer_healthy";
const WRITER_RUNNING_METRIC: &str = "system_log_writer_running";
const WRITER_TERMINATED_DROP_REASON: &str = "writer_terminated";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SystemLogDeliveryGuarantee {
    BestEffort,
}

impl SystemLogDeliveryGuarantee {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BestEffort => "best_effort",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemLogWriteFailure {
    pub failed_events: u64,
    pub reason: &'static str,
    pub occurred_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemLogIngestionStatus {
    pub delivery_guarantee: SystemLogDeliveryGuarantee,
    pub queue_depth: usize,
    pub queue_capacity: usize,
    pub pending_events: u64,
    pub persisted_events: u64,
    pub dropped_events: u64,
    pub writer_running: bool,
    pub writer_healthy: bool,
    pub latest_write_failure: Option<SystemLogWriteFailure>,
}

pub(super) struct IngestionState {
    queue_capacity: usize,
    snapshot: Mutex<IngestionSnapshot>,
}

struct IngestionSnapshot {
    accepting: bool,
    queue_depth: u64,
    pending_events: u64,
    persisted_events: u64,
    dropped_events: u64,
    writer_running: bool,
    writer_healthy: bool,
    abnormal_writer_drop_reason: &'static str,
    latest_write_failure: Option<SystemLogWriteFailure>,
}

impl IngestionState {
    pub(super) fn new(queue_capacity: usize) -> Self {
        let state = Self {
            queue_capacity,
            snapshot: Mutex::new(IngestionSnapshot::new()),
        };
        publish_initial_metrics();
        state
    }

    pub(super) fn status(&self) -> SystemLogIngestionStatus {
        let snapshot = self.lock_snapshot();
        SystemLogIngestionStatus {
            delivery_guarantee: SystemLogDeliveryGuarantee::BestEffort,
            queue_depth: usize::try_from(snapshot.queue_depth).expect("system log queue depth must fit in usize"),
            queue_capacity: self.queue_capacity,
            pending_events: snapshot.pending_events,
            persisted_events: snapshot.persisted_events,
            dropped_events: snapshot.dropped_events,
            writer_running: snapshot.writer_running,
            writer_healthy: snapshot.writer_healthy,
            latest_write_failure: snapshot.latest_write_failure.clone(),
        }
    }

    pub(super) fn record_writer_started(&self) {
        let mut snapshot = self.lock_snapshot();
        snapshot.writer_running = true;
        gauge!(WRITER_RUNNING_METRIC).set(1.0);
    }

    pub(super) fn record_writer_stopped(&self, completed: bool) {
        let mut snapshot = self.lock_snapshot();
        if !snapshot.writer_running {
            return;
        }
        snapshot.accepting = false;
        if !completed {
            let reason = snapshot.abnormal_writer_drop_reason;
            let failed_events = snapshot.pending_events;
            discard_all_pending(&mut snapshot, reason);
            set_writer_healthy(&mut snapshot, false);
            snapshot.latest_write_failure = Some(SystemLogWriteFailure {
                failed_events,
                reason,
                occurred_at: time::OffsetDateTime::now_utc(),
            });
        }
        snapshot.writer_running = false;
        gauge!(WRITER_RUNNING_METRIC).set(0.0);
    }

    pub(super) fn stop_accepting(&self) {
        self.lock_snapshot().accepting = false;
    }

    pub(super) fn mark_shutdown_timeout(&self) {
        self.lock_snapshot().abnormal_writer_drop_reason = SHUTDOWN_TIMEOUT_DROP_REASON;
    }

    pub(super) fn record_drop(&self, reason: &'static str, count: u64) {
        record_drop(&mut self.lock_snapshot(), reason, count);
    }

    pub(super) fn record_enqueue<F>(&self, enqueue: F)
    where
        F: FnOnce() -> Result<(), &'static str>,
    {
        let mut snapshot = self.lock_snapshot();
        if !snapshot.accepting {
            record_drop(&mut snapshot, RUNTIME_STOPPING_DROP_REASON, 1);
            return;
        }
        match enqueue() {
            Ok(()) => record_accepted(&mut snapshot, self.queue_capacity),
            Err(reason) => record_drop(&mut snapshot, reason, 1),
        }
    }

    pub(super) fn record_dequeued(&self) {
        let mut snapshot = self.lock_snapshot();
        snapshot.queue_depth = checked_sub(snapshot.queue_depth, 1, "system log queue depth accounting underflow");
        gauge!(QUEUE_DEPTH_METRIC).set(snapshot.queue_depth as f64);
    }

    pub(super) fn record_persisted(&self, count: u64) {
        let mut snapshot = self.lock_snapshot();
        complete_pending(&mut snapshot, count);
        snapshot.persisted_events = checked_add(snapshot.persisted_events, count, "system log persisted event accounting overflow");
        counter!(PERSISTED_METRIC).increment(count);
        set_writer_healthy(&mut snapshot, true);
    }

    pub(super) fn record_write_failure(&self, count: u64, error: &str) -> &'static str {
        let reason = classify_write_failure(error);
        let mut snapshot = self.lock_snapshot();
        discard_pending(&mut snapshot, "writer_failed", count);
        set_writer_healthy(&mut snapshot, false);
        snapshot.latest_write_failure = Some(SystemLogWriteFailure {
            failed_events: count,
            reason,
            occurred_at: time::OffsetDateTime::now_utc(),
        });
        reason
    }

    #[cfg(test)]
    pub(super) fn discard_all_pending(&self, reason: &'static str) {
        let mut snapshot = self.lock_snapshot();
        snapshot.accepting = false;
        discard_all_pending(&mut snapshot, reason);
    }

    fn lock_snapshot(&self) -> MutexGuard<'_, IngestionSnapshot> {
        self.snapshot.lock().expect("system log ingestion state lock poisoned")
    }
}

impl IngestionSnapshot {
    const fn new() -> Self {
        Self {
            accepting: true,
            queue_depth: 0,
            pending_events: 0,
            persisted_events: 0,
            dropped_events: 0,
            writer_running: false,
            writer_healthy: true,
            abnormal_writer_drop_reason: WRITER_TERMINATED_DROP_REASON,
            latest_write_failure: None,
        }
    }
}

fn record_accepted(snapshot: &mut IngestionSnapshot, queue_capacity: usize) {
    snapshot.pending_events = checked_add(snapshot.pending_events, 1, "system log pending event accounting overflow");
    snapshot.queue_depth = checked_add(snapshot.queue_depth, 1, "system log queue depth accounting overflow");
    assert!(
        snapshot.queue_depth <= queue_capacity as u64,
        "system log queue depth exceeded channel capacity"
    );
    gauge!(PENDING_METRIC).set(snapshot.pending_events as f64);
    gauge!(QUEUE_DEPTH_METRIC).set(snapshot.queue_depth as f64);
}

fn discard_all_pending(snapshot: &mut IngestionSnapshot, reason: &'static str) {
    // Pending covers channel, buffer, and in-flight sink events.
    let count = snapshot.pending_events;
    snapshot.pending_events = 0;
    snapshot.queue_depth = 0;
    gauge!(PENDING_METRIC).set(0.0);
    gauge!(QUEUE_DEPTH_METRIC).set(0.0);
    if count > 0 {
        record_drop(snapshot, reason, count);
    }
}

fn discard_pending(snapshot: &mut IngestionSnapshot, reason: &'static str, count: u64) {
    complete_pending(snapshot, count);
    record_drop(snapshot, reason, count);
}

fn complete_pending(snapshot: &mut IngestionSnapshot, count: u64) {
    snapshot.pending_events = checked_sub(snapshot.pending_events, count, "system log pending event accounting underflow");
    gauge!(PENDING_METRIC).set(snapshot.pending_events as f64);
}

fn record_drop(snapshot: &mut IngestionSnapshot, reason: &'static str, count: u64) {
    snapshot.dropped_events = checked_add(snapshot.dropped_events, count, "system log dropped event accounting overflow");
    counter!(DROP_METRIC, "reason" => reason).increment(count);
}

fn set_writer_healthy(snapshot: &mut IngestionSnapshot, healthy: bool) {
    snapshot.writer_healthy = healthy;
    gauge!(WRITER_HEALTH_METRIC).set(if healthy { 1.0 } else { 0.0 });
}

fn publish_initial_metrics() {
    counter!(PERSISTED_METRIC).absolute(0);
    gauge!(QUEUE_DEPTH_METRIC).set(0.0);
    gauge!(PENDING_METRIC).set(0.0);
    gauge!(WRITER_RUNNING_METRIC).set(0.0);
    gauge!(WRITER_HEALTH_METRIC).set(1.0);
}

fn checked_add(value: u64, count: u64, message: &'static str) -> u64 {
    value.checked_add(count).unwrap_or_else(|| panic!("{message}"))
}

fn checked_sub(value: u64, count: u64, message: &'static str) -> u64 {
    value.checked_sub(count).unwrap_or_else(|| panic!("{message}"))
}

fn classify_write_failure(error: &str) -> &'static str {
    let error = error.to_ascii_lowercase();
    if contains_any(&error, &["permission denied", "not authorized", "authentication"]) {
        return "authorization";
    }
    if contains_any(&error, &["partition", "no partition", "check constraint"]) {
        return "partition";
    }
    if contains_any(&error, &["constraint", "violates", "duplicate key"]) {
        return "constraint";
    }
    if contains_any(&error, &["connection", "network", "timed out", "broken pipe", "unavailable"]) {
        return "connection";
    }
    "unknown"
}

fn contains_any(value: &str, markers: &[&str]) -> bool {
    markers.iter().any(|marker| value.contains(marker))
}

#[cfg(test)]
mod tests {
    use super::classify_write_failure;

    #[test]
    fn write_failure_reason_is_safe_and_actionable() {
        assert_eq!(classify_write_failure("password authentication failed"), "authorization");
        assert_eq!(classify_write_failure("no partition of relation found"), "partition");
        assert_eq!(classify_write_failure("duplicate key violates constraint"), "constraint");
        assert_eq!(classify_write_failure("connection timed out"), "connection");
        assert_eq!(classify_write_failure("unexpected driver state"), "unknown");
    }
}
