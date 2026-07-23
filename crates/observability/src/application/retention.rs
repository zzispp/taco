use time::{Duration, OffsetDateTime};

use super::{ObservabilityError, ObservabilityResult, SystemLogRetentionStore, localized};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SystemLogRetentionReport {
    pub deleted: u64,
    /// Number of committed cleanup transactions.
    pub batches: u64,
}

pub(super) async fn cleanup_expired(
    store: &dyn SystemLogRetentionStore,
    retention_days: u64,
    boundary_batch_size: u64,
) -> ObservabilityResult<SystemLogRetentionReport> {
    cleanup_expired_at(store, OffsetDateTime::now_utc(), retention_days, boundary_batch_size).await
}

async fn cleanup_expired_at(
    store: &dyn SystemLogRetentionStore,
    reference: OffsetDateTime,
    retention_days: u64,
    boundary_batch_size: u64,
) -> ObservabilityResult<SystemLogRetentionReport> {
    if boundary_batch_size == 0 {
        return Err(ObservabilityError::InvalidInput(localized("errors.observability.invalid_cleanup_batch_size")));
    }
    store.cleanup_before(cutoff(reference, retention_days)?, boundary_batch_size).await
}

fn cutoff(reference: OffsetDateTime, retention_days: u64) -> ObservabilityResult<OffsetDateTime> {
    let days = i64::try_from(retention_days).map_err(|error| ObservabilityError::Infrastructure(format!("system log retention days overflow: {error}")))?;
    reference
        .checked_sub(Duration::days(days))
        .ok_or_else(|| ObservabilityError::Infrastructure("system log retention cutoff overflow".into()))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use crate::application::{ObservabilityResult, SystemLogRetentionReport, SystemLogRetentionStore};

    use super::cleanup_expired_at;

    #[tokio::test]
    async fn retention_passes_an_exact_rolling_cutoff_and_boundary_limit() {
        let store = RecordingStore::new(SystemLogRetentionReport { deleted: 8, batches: 3 });
        let reference = timestamp("2026-07-20T18:00:00Z");

        let report = cleanup_expired_at(&store, reference, 7, 1_000).await.unwrap();

        assert_eq!(report, SystemLogRetentionReport { deleted: 8, batches: 3 });
        assert_eq!(store.calls(), vec![(timestamp("2026-07-13T18:00:00Z"), 1_000)]);
    }

    #[tokio::test]
    async fn retention_rejects_a_zero_boundary_batch_size_before_storage() {
        let store = RecordingStore::new(SystemLogRetentionReport::default());

        let result = cleanup_expired_at(&store, timestamp("2026-07-20T18:00:00Z"), 7, 0).await;

        assert!(matches!(result, Err(crate::application::ObservabilityError::InvalidInput(_))));
        assert_eq!(store.calls(), Vec::new());
    }

    struct RecordingStore {
        report: SystemLogRetentionReport,
        calls: Mutex<Vec<(OffsetDateTime, u64)>>,
    }

    impl RecordingStore {
        fn new(report: SystemLogRetentionReport) -> Self {
            Self {
                report,
                calls: Mutex::new(Vec::new()),
            }
        }

        fn calls(&self) -> Vec<(OffsetDateTime, u64)> {
            self.calls.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl SystemLogRetentionStore for RecordingStore {
        async fn cleanup_before(&self, cutoff: OffsetDateTime, boundary_batch_size: u64) -> ObservabilityResult<SystemLogRetentionReport> {
            self.calls.lock().unwrap().push((cutoff, boundary_batch_size));
            Ok(self.report)
        }
    }

    fn timestamp(value: &str) -> OffsetDateTime {
        OffsetDateTime::parse(value, &Rfc3339).unwrap()
    }
}
