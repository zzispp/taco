use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use kernel::{
    excel::TemporaryXlsxFile,
    pagination::{CursorPage, CursorPageRequest},
};
use time::OffsetDateTime;

use crate::domain::{NewSystemLog, SystemLogDetail, SystemLogFilter, SystemLogSummary};

use super::{
    ObservabilityResult, SystemLogCursorQuery, SystemLogCursorSlice, SystemLogExportLayout, SystemLogExportRequest, SystemLogExportSlice,
    SystemLogRetentionReport,
};

#[async_trait]
/// Persists, queries, and deletes system logs while preserving caller-owned transaction boundaries.
pub trait SystemLogRepository: Send + Sync + 'static {
    async fn insert_batch(&self, events: &[NewSystemLog]) -> ObservabilityResult<()>;
    async fn page(&self, filter: SystemLogFilter, page: SystemLogCursorQuery) -> ObservabilityResult<SystemLogCursorSlice>;
    async fn find(&self, id: &str) -> ObservabilityResult<Option<SystemLogDetail>>;
    async fn delete_ids_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> ObservabilityResult<()>;
    async fn count(&self, filter: SystemLogFilter) -> ObservabilityResult<u64>;
    async fn delete_filtered_batch(&self, filter: SystemLogFilter, limit: u64) -> ObservabilityResult<u64>;
    async fn begin_export(&self) -> ObservabilityResult<Box<dyn SystemLogExportSession>>;
}

#[async_trait]
/// Reclaims expired UTC-day partitions and deletes only the cutoff-day boundary by row.
pub trait SystemLogRetentionStore: Send + Sync + 'static {
    async fn cleanup_before(&self, cutoff: OffsetDateTime, boundary_batch_size: u64) -> ObservabilityResult<SystemLogRetentionReport>;
}

#[async_trait]
/// Streams one consistent system-log export and must release its database snapshot in `finish` or `abort`.
pub trait SystemLogExportSession: Send {
    async fn page(&mut self, filter: SystemLogFilter, page: SystemLogCursorQuery) -> ObservabilityResult<SystemLogExportSlice>;
    async fn finish(self: Box<Self>) -> ObservabilityResult<()>;
    async fn abort(self: Box<Self>) -> ObservabilityResult<()>;
}

#[async_trait]
/// Appends export rows and finalizes one workbook artifact.
pub trait SystemLogExportWriter: Send {
    async fn append(&mut self, item: SystemLogDetail) -> ObservabilityResult<()>;
    async fn finish(self: Box<Self>) -> ObservabilityResult<TemporaryXlsxFile>;
}

pub struct SystemLogExportWriterRequest {
    pub capacity: usize,
    pub layout: SystemLogExportLayout,
}

/// Creates the concrete export writer without leaking its implementation into application orchestration.
pub trait SystemLogExportWriterFactory: Send + Sync + 'static {
    fn start(&self, request: SystemLogExportWriterRequest) -> ObservabilityResult<Box<dyn SystemLogExportWriter>>;
}

#[async_trait]
/// Provides the system-log management use cases consumed by authenticated HTTP handlers.
pub trait SystemLogUseCase: Send + Sync + 'static {
    async fn page(&self, filter: SystemLogFilter, page: CursorPageRequest) -> ObservabilityResult<CursorPage<SystemLogSummary>>;
    async fn detail(&self, id: &str) -> ObservabilityResult<SystemLogDetail>;
    async fn delete_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> ObservabilityResult<()>;
    async fn count(&self, filter: SystemLogFilter) -> ObservabilityResult<u64>;
    async fn delete_filtered(&self, filter: SystemLogFilter, batch_size: u64) -> ObservabilityResult<SystemLogRetentionReport>;
}

#[async_trait]
/// Produces a complete system-log workbook while owning the snapshot and writer lifecycle.
pub trait SystemLogExportUseCase: Send + Sync + 'static {
    async fn export_xlsx(&self, request: SystemLogExportRequest) -> ObservabilityResult<TemporaryXlsxFile>;
}

/// Removes expired system logs according to scheduler-provided retention settings.
#[async_trait]
pub trait SystemLogRetentionUseCase: Send + Sync + 'static {
    async fn cleanup_expired(&self, retention_days: u64, boundary_batch_size: u64) -> ObservabilityResult<SystemLogRetentionReport>;
}
