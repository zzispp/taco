use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use storage::Database;

use crate::{
    application::{
        ObservabilityResult, SystemLogCursorQuery, SystemLogCursorSlice, SystemLogExportSession, SystemLogRepository, SystemLogRetentionReport,
        SystemLogRetentionStore,
    },
    domain::{NewSystemLog, SystemLogDetail, SystemLogFilter},
};

use super::{command, export_session::StorageSystemLogExportSession, query, retention_store};

#[derive(Clone)]
pub struct StorageSystemLogRepository {
    database: Database,
}

impl StorageSystemLogRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl SystemLogRepository for StorageSystemLogRepository {
    async fn insert_batch(&self, events: &[NewSystemLog]) -> ObservabilityResult<()> {
        command::insert_batch(self.database.raw_pool(), events).await
    }

    async fn page(&self, filter: SystemLogFilter, page: SystemLogCursorQuery) -> ObservabilityResult<SystemLogCursorSlice> {
        query::page(self.database.raw_pool(), filter, page).await
    }

    async fn find(&self, id: &str) -> ObservabilityResult<Option<SystemLogDetail>> {
        query::find(self.database.raw_pool(), id).await
    }

    async fn delete_ids_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> ObservabilityResult<()> {
        command::delete_ids_with_audit(self.database.raw_pool(), ids, audit).await
    }

    async fn count(&self, filter: SystemLogFilter) -> ObservabilityResult<u64> {
        query::count(self.database.raw_pool(), filter).await
    }

    async fn delete_filtered_batch(&self, filter: SystemLogFilter, limit: u64) -> ObservabilityResult<u64> {
        command::delete_filtered_batch(self.database.raw_pool(), filter, limit).await
    }

    async fn begin_export(&self) -> ObservabilityResult<Box<dyn SystemLogExportSession>> {
        Ok(Box::new(StorageSystemLogExportSession::begin(&self.database).await?))
    }
}

#[async_trait]
impl SystemLogRetentionStore for StorageSystemLogRepository {
    async fn cleanup_before(&self, cutoff: time::OffsetDateTime, boundary_batch_size: u64) -> ObservabilityResult<SystemLogRetentionReport> {
        retention_store::cleanup_before(self.database.raw_pool(), cutoff, boundary_batch_size).await
    }
}
