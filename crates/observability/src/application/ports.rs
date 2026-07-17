use async_trait::async_trait;
use kernel::pagination::{CursorPage, CursorPageRequest};
use time::OffsetDateTime;

use crate::domain::{NewSystemLog, SystemLogDetail, SystemLogFilter, SystemLogSummary};

use super::{ObservabilityResult, SystemLogCursorQuery, SystemLogCursorSlice, SystemLogExportSlice, SystemLogRetentionReport};

#[async_trait]
/// Persists, queries, and deletes system logs while preserving caller-owned transaction boundaries.
pub trait SystemLogRepository: Send + Sync + 'static {
    async fn insert_batch(&self, events: &[NewSystemLog]) -> ObservabilityResult<()>;
    async fn page(&self, filter: SystemLogFilter, page: SystemLogCursorQuery) -> ObservabilityResult<SystemLogCursorSlice>;
    async fn find(&self, id: &str) -> ObservabilityResult<Option<SystemLogDetail>>;
    async fn delete_ids(&self, ids: &[String]) -> ObservabilityResult<()>;
    async fn count(&self, filter: SystemLogFilter) -> ObservabilityResult<u64>;
    async fn delete_filtered_batch(&self, filter: SystemLogFilter, limit: u64) -> ObservabilityResult<u64>;
    async fn delete_expired_batch(&self, cutoff: OffsetDateTime, limit: u64) -> ObservabilityResult<u64>;
    async fn begin_export(&self) -> ObservabilityResult<Box<dyn SystemLogExportSession>>;
}

#[async_trait]
/// Streams one consistent system-log export and must release its database snapshot in `finish`.
pub trait SystemLogExportSession: Send {
    async fn page(&mut self, filter: SystemLogFilter, page: SystemLogCursorQuery) -> ObservabilityResult<SystemLogExportSlice>;
    async fn finish(self: Box<Self>) -> ObservabilityResult<()>;
}

#[async_trait]
/// Provides the system-log management use cases consumed by authenticated HTTP handlers.
pub trait SystemLogUseCase: Send + Sync + 'static {
    async fn page(&self, filter: SystemLogFilter, page: CursorPageRequest) -> ObservabilityResult<CursorPage<SystemLogSummary>>;
    async fn detail(&self, id: &str) -> ObservabilityResult<SystemLogDetail>;
    async fn delete(&self, id: String) -> ObservabilityResult<()>;
    async fn delete_many(&self, ids: Vec<String>) -> ObservabilityResult<()>;
    async fn count(&self, filter: SystemLogFilter) -> ObservabilityResult<u64>;
    async fn delete_filtered(&self, filter: SystemLogFilter, batch_size: u64) -> ObservabilityResult<SystemLogRetentionReport>;
    async fn begin_export(&self) -> ObservabilityResult<Box<dyn SystemLogExportSession>>;
}

/// Removes expired system logs according to scheduler-provided retention settings.
#[async_trait]
pub trait SystemLogRetentionUseCase: Send + Sync + 'static {
    async fn cleanup_expired(&self, retention_days: u64, batch_size: u64) -> ObservabilityResult<SystemLogRetentionReport>;
}
