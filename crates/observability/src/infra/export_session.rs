use async_trait::async_trait;
use sqlx::{Postgres, Transaction};
use storage::Database;

use crate::{
    application::{ObservabilityError, ObservabilityResult, SystemLogCursorQuery, SystemLogExportSession, SystemLogExportSlice},
    domain::SystemLogFilter,
};

use super::{mapping, query};

pub(super) struct StorageSystemLogExportSession {
    transaction: Transaction<'static, Postgres>,
}

impl StorageSystemLogExportSession {
    pub(super) async fn begin(database: &Database) -> ObservabilityResult<Self> {
        let transaction = database
            .begin_consistent_snapshot()
            .await
            .map_err(|error| ObservabilityError::Infrastructure(error.to_string()))?;
        Ok(Self { transaction })
    }
}

#[async_trait]
impl SystemLogExportSession for StorageSystemLogExportSession {
    async fn page(&mut self, filter: SystemLogFilter, page: SystemLogCursorQuery) -> ObservabilityResult<SystemLogExportSlice> {
        query::page_for_export_on(&mut self.transaction, filter, page).await
    }

    async fn finish(self: Box<Self>) -> ObservabilityResult<()> {
        let Self { transaction } = *self;
        transaction.commit().await.map_err(mapping::sqlx_error)
    }
}
