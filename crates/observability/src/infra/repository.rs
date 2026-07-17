use async_trait::async_trait;
use storage::Database;
use time::OffsetDateTime;

use crate::{
    application::{ObservabilityResult, SystemLogCursorQuery, SystemLogCursorSlice, SystemLogExportSession, SystemLogRepository},
    domain::{NewSystemLog, SystemLogDetail, SystemLogFilter},
};

use super::{command, export_session::StorageSystemLogExportSession, query};

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

    async fn delete_ids(&self, ids: &[String]) -> ObservabilityResult<()> {
        command::delete_ids(self.database.raw_pool(), ids).await
    }

    async fn count(&self, filter: SystemLogFilter) -> ObservabilityResult<u64> {
        query::count(self.database.raw_pool(), filter).await
    }

    async fn delete_filtered_batch(&self, filter: SystemLogFilter, limit: u64) -> ObservabilityResult<u64> {
        command::delete_filtered_batch(self.database.raw_pool(), filter, limit).await
    }

    async fn delete_expired_batch(&self, cutoff: OffsetDateTime, limit: u64) -> ObservabilityResult<u64> {
        command::delete_expired_batch(self.database.raw_pool(), cutoff, limit).await
    }

    async fn begin_export(&self) -> ObservabilityResult<Box<dyn SystemLogExportSession>> {
        Ok(Box::new(StorageSystemLogExportSession::begin(&self.database).await?))
    }
}
