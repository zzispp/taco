use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use metrics::counter;

const DROP_METRIC: &str = "system_log_dropped_total";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemLogWriteFailure {
    pub failed_events: u64,
    pub reason: &'static str,
    pub occurred_at: time::OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SystemLogIngestionStatus {
    pub dropped_events: u64,
    pub writer_healthy: bool,
    pub latest_write_failure: Option<SystemLogWriteFailure>,
}

pub(super) struct IngestionState {
    dropped_events: AtomicU64,
    pending_events: AtomicU64,
    writer_healthy: AtomicBool,
    latest_write_failure: Mutex<Option<SystemLogWriteFailure>>,
}

impl Default for IngestionState {
    fn default() -> Self {
        Self {
            dropped_events: AtomicU64::default(),
            pending_events: AtomicU64::default(),
            writer_healthy: AtomicBool::new(true),
            latest_write_failure: Mutex::default(),
        }
    }
}

impl IngestionState {
    pub(super) fn status(&self) -> SystemLogIngestionStatus {
        SystemLogIngestionStatus {
            dropped_events: self.dropped_events.load(Ordering::Relaxed),
            writer_healthy: self.writer_healthy.load(Ordering::Relaxed),
            latest_write_failure: self.latest_write_failure.lock().unwrap().clone(),
        }
    }

    pub(super) fn record_drop(&self, reason: &'static str, count: u64) {
        self.dropped_events.fetch_add(count, Ordering::Relaxed);
        counter!(DROP_METRIC, "reason" => reason).increment(count);
    }

    pub(super) fn record_accepted(&self) {
        self.pending_events.fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_send_failure(&self, reason: &'static str) {
        self.discard_pending(reason, 1);
    }

    pub(super) fn record_persisted(&self, count: u64) {
        self.complete_pending(count);
        self.writer_healthy.store(true, Ordering::Relaxed);
    }

    pub(super) fn record_write_failure(&self, count: u64, error: &str) -> &'static str {
        let reason = classify_write_failure(error);
        self.discard_pending("writer_failed", count);
        self.writer_healthy.store(false, Ordering::Relaxed);
        *self.latest_write_failure.lock().unwrap() = Some(SystemLogWriteFailure {
            failed_events: count,
            reason,
            occurred_at: time::OffsetDateTime::now_utc(),
        });
        reason
    }

    pub(super) fn discard_all_pending(&self, reason: &'static str) {
        let count = self.pending_events.swap(0, Ordering::Relaxed);
        self.record_drop(reason, count);
    }

    fn discard_pending(&self, reason: &'static str, count: u64) {
        self.complete_pending(count);
        self.record_drop(reason, count);
    }

    fn complete_pending(&self, count: u64) {
        let previous = self.pending_events.fetch_sub(count, Ordering::Relaxed);
        assert!(previous >= count, "system log pending event accounting underflow");
    }
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
