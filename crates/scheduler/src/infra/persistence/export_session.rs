use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};

use crate::{
    application::{ExecutionLogSummary, SchedulerCursorQuery, SchedulerCursorSlice, SchedulerQueryExportSession, SchedulerResult},
    domain::{Job, JobListFilter, JobLogListFilter},
};

use super::{mapping::map_sqlx_error, query};

const SNAPSHOT_BEGIN: &str = "BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY";

pub(super) struct StorageSchedulerExportSession {
    transaction: Transaction<'static, Postgres>,
}

impl StorageSchedulerExportSession {
    pub(super) async fn begin(pool: &PgPool) -> SchedulerResult<Self> {
        let transaction = pool.begin_with(SNAPSHOT_BEGIN).await.map_err(map_sqlx_error)?;
        Ok(Self { transaction })
    }
}

#[async_trait]
impl SchedulerQueryExportSession for StorageSchedulerExportSession {
    async fn page_jobs(&mut self, filter: JobListFilter, page: SchedulerCursorQuery) -> SchedulerResult<SchedulerCursorSlice<Job>> {
        query::page_jobs_on(&mut self.transaction, filter, page).await
    }

    async fn page_execution_logs(
        &mut self,
        filter: JobLogListFilter,
        page: SchedulerCursorQuery,
    ) -> SchedulerResult<SchedulerCursorSlice<ExecutionLogSummary>> {
        query::page_execution_logs_on(&mut self.transaction, filter, page).await
    }

    async fn finish(self: Box<Self>) -> SchedulerResult<()> {
        let Self { transaction } = *self;
        transaction.commit().await.map_err(map_sqlx_error)
    }
}

#[cfg(test)]
mod tests {
    use super::SNAPSHOT_BEGIN;

    #[test]
    fn export_transaction_is_read_only_repeatable_read() {
        assert_eq!(SNAPSHOT_BEGIN, "BEGIN ISOLATION LEVEL REPEATABLE READ READ ONLY");
    }
}
