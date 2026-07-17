use std::{collections::VecDeque, sync::Mutex};

use async_trait::async_trait;
use time::OffsetDateTime;

use crate::{
    application::{ObservabilityError, ObservabilityResult, SystemLogCursorQuery, SystemLogCursorSlice, SystemLogExportSession, SystemLogRepository},
    domain::{NewSystemLog, SystemLogDetail, SystemLogFilter},
};

use super::delete_all_matching;

#[tokio::test]
async fn manual_cleanup_uses_the_configured_batch_size_until_the_filter_is_empty() {
    let repository = CleanupRepository::new([Ok(2), Ok(1), Ok(0)]);

    let report = delete_all_matching(&repository, SystemLogFilter::default(), 2400).await.unwrap();

    assert_eq!(report.deleted, 3);
    assert_eq!(report.batches, 2);
    assert_eq!(repository.limits(), vec![2400, 2400, 2400]);
}

#[tokio::test]
async fn manual_cleanup_failure_keeps_committed_batch_totals() {
    let repository = CleanupRepository::new([Ok(2), Err(ObservabilityError::Infrastructure("database unavailable".into()))]);

    let error = delete_all_matching(&repository, SystemLogFilter::default(), 1000).await.unwrap_err();

    assert!(matches!(error, ObservabilityError::PartialCleanup { deleted: 2, batches: 1, .. }));
}

struct CleanupRepository {
    batches: Mutex<VecDeque<ObservabilityResult<u64>>>,
    limits: Mutex<Vec<u64>>,
}

impl CleanupRepository {
    fn new(batches: impl IntoIterator<Item = ObservabilityResult<u64>>) -> Self {
        Self {
            batches: Mutex::new(batches.into_iter().collect()),
            limits: Mutex::new(Vec::new()),
        }
    }

    fn limits(&self) -> Vec<u64> {
        self.limits.lock().unwrap().clone()
    }
}

#[async_trait]
impl SystemLogRepository for CleanupRepository {
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

    async fn delete_filtered_batch(&self, _: SystemLogFilter, limit: u64) -> ObservabilityResult<u64> {
        self.limits.lock().unwrap().push(limit);
        self.batches.lock().unwrap().pop_front().unwrap()
    }

    async fn delete_expired_batch(&self, _: OffsetDateTime, _: u64) -> ObservabilityResult<u64> {
        unreachable!()
    }

    async fn begin_export(&self) -> ObservabilityResult<Box<dyn SystemLogExportSession>> {
        unreachable!()
    }
}
