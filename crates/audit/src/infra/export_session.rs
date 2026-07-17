use async_trait::async_trait;
use sqlx::{Postgres, Transaction};
use storage::ObservedPgPool;

use crate::{
    application::{AuditCursorQuery, AuditCursorSlice, AuditExportSession, AuditResult, LoginCursorBoundary, OperationCursorBoundary},
    domain::{LoginLog, LoginLogFilter, OperationLogFilter, OperationLogSummary},
};

use super::{mapping, query};

const SNAPSHOT_BEGIN: &str = "BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY";

pub(super) struct StorageAuditExportSession {
    transaction: Transaction<'static, Postgres>,
}

impl StorageAuditExportSession {
    pub(super) async fn begin(pool: ObservedPgPool) -> AuditResult<Self> {
        let transaction = pool.begin_with(SNAPSHOT_BEGIN).await.map_err(mapping::sqlx_error)?;
        Ok(Self { transaction })
    }
}

#[async_trait]
impl AuditExportSession for StorageAuditExportSession {
    async fn page_operations(
        &mut self,
        filter: OperationLogFilter,
        page: AuditCursorQuery<OperationCursorBoundary>,
    ) -> AuditResult<AuditCursorSlice<OperationLogSummary>> {
        query::page_operations_on(&mut self.transaction, filter, page).await
    }

    async fn page_logins(&mut self, filter: LoginLogFilter, page: AuditCursorQuery<LoginCursorBoundary>) -> AuditResult<AuditCursorSlice<LoginLog>> {
        query::page_logins_on(&mut self.transaction, filter, page).await
    }

    async fn finish(self: Box<Self>) -> AuditResult<()> {
        let Self { transaction } = *self;
        transaction.commit().await.map_err(mapping::sqlx_error)
    }
}

#[cfg(test)]
mod tests {
    use super::SNAPSHOT_BEGIN;

    #[test]
    fn export_transaction_uses_a_read_only_repeatable_snapshot() {
        assert_eq!(SNAPSHOT_BEGIN, "BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY");
    }
}
