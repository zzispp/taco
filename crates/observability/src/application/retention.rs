use time::{Duration, OffsetDateTime};

use super::{ObservabilityError, ObservabilityResult, SystemLogRepository};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SystemLogRetentionReport {
    pub deleted: u64,
    pub batches: u64,
}

pub(super) async fn cleanup_expired(
    repository: &dyn SystemLogRepository,
    retention_days: u64,
    batch_size: u64,
) -> ObservabilityResult<SystemLogRetentionReport> {
    let cutoff = cutoff(retention_days)?;
    if batch_size == 0 {
        return Err(ObservabilityError::Infrastructure("system log cleanup batch size must be positive".into()));
    }
    let mut report = SystemLogRetentionReport { deleted: 0, batches: 0 };
    loop {
        let deleted = match repository.delete_expired_batch(cutoff, batch_size).await {
            Ok(deleted) => deleted,
            Err(error) => return Err(partial_cleanup_error(report, error)),
        };
        if deleted == 0 {
            return Ok(report);
        }
        report.deleted = report
            .deleted
            .checked_add(deleted)
            .ok_or_else(|| ObservabilityError::Infrastructure("system log cleanup deleted count overflow".into()))?;
        report.batches = report
            .batches
            .checked_add(1)
            .ok_or_else(|| ObservabilityError::Infrastructure("system log cleanup batch count overflow".into()))?;
    }
}

fn partial_cleanup_error(report: SystemLogRetentionReport, error: ObservabilityError) -> ObservabilityError {
    if report.deleted == 0 {
        return error;
    }
    ObservabilityError::partial_cleanup(report, error.to_string())
}

fn cutoff(retention_days: u64) -> ObservabilityResult<OffsetDateTime> {
    let days = i64::try_from(retention_days).map_err(|error| ObservabilityError::Infrastructure(format!("system log retention days overflow: {error}")))?;
    OffsetDateTime::now_utc()
        .checked_sub(Duration::days(days))
        .ok_or_else(|| ObservabilityError::Infrastructure("system log retention cutoff overflow".into()))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use time::OffsetDateTime;

    use crate::{
        application::{ObservabilityError, ObservabilityResult, SystemLogCursorQuery, SystemLogCursorSlice, SystemLogExportSession, SystemLogRepository},
        domain::{NewSystemLog, SystemLogDetail, SystemLogFilter},
    };

    use super::cleanup_expired;

    #[tokio::test]
    async fn retention_cleanup_continues_until_every_expired_batch_is_deleted() {
        let repository = RetentionRepository::new([Ok(3), Ok(2), Ok(0)]);

        let report = cleanup_expired(&repository, 7, 1_000).await.unwrap();

        assert_eq!(report.deleted, 5);
        assert_eq!(report.batches, 2);
        assert_eq!(repository.limits(), vec![1_000, 1_000, 1_000]);
    }

    #[tokio::test]
    async fn retention_failure_keeps_committed_batch_totals() {
        let repository = RetentionRepository::new([Ok(3), Err(ObservabilityError::Infrastructure("database unavailable".into()))]);

        let error = cleanup_expired(&repository, 7, 1_000).await.unwrap_err();

        assert!(matches!(error, ObservabilityError::PartialCleanup { deleted: 3, batches: 1, .. }));
    }

    struct RetentionRepository {
        results: Mutex<Vec<ObservabilityResult<u64>>>,
        limits: Mutex<Vec<u64>>,
    }

    impl RetentionRepository {
        fn new(results: impl IntoIterator<Item = ObservabilityResult<u64>>) -> Self {
            Self {
                results: Mutex::new(results.into_iter().collect()),
                limits: Mutex::new(Vec::new()),
            }
        }

        fn limits(&self) -> Vec<u64> {
            self.limits.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl SystemLogRepository for RetentionRepository {
        async fn insert_batch(&self, _: &[NewSystemLog]) -> ObservabilityResult<()> {
            unreachable!()
        }
        async fn page(&self, _: SystemLogFilter, _: SystemLogCursorQuery) -> ObservabilityResult<SystemLogCursorSlice> {
            unreachable!()
        }
        async fn find(&self, _: &str) -> ObservabilityResult<Option<SystemLogDetail>> {
            unreachable!()
        }
        async fn delete_ids(&self, _: &[String]) -> ObservabilityResult<()> {
            unreachable!()
        }
        async fn count(&self, _: SystemLogFilter) -> ObservabilityResult<u64> {
            unreachable!()
        }
        async fn delete_filtered_batch(&self, _: SystemLogFilter, _: u64) -> ObservabilityResult<u64> {
            unreachable!()
        }

        async fn delete_expired_batch(&self, _: OffsetDateTime, limit: u64) -> ObservabilityResult<u64> {
            self.limits.lock().unwrap().push(limit);
            self.results.lock().unwrap().remove(0)
        }

        async fn begin_export(&self) -> ObservabilityResult<Box<dyn SystemLogExportSession>> {
            unreachable!()
        }
    }
}
