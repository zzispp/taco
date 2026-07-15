use async_trait::async_trait;
use sqlx::{AssertSqlSafe, PgConnection, PgPool, Postgres, QueryBuilder, query_as, query_scalar};

use crate::{
    application::{ExecutionLogDetail, ExecutionLogSummary, SchedulerCursorQuery, SchedulerCursorSlice, SchedulerQueryStore, SchedulerResult},
    domain::{ExecutionState, Job, JobListFilter, JobLogListFilter},
};

mod cursor;

use cursor::{WindowSpec, empty_slice, push_limit, push_window, resolve_job_snapshot, resolve_log_snapshot, slice};

use super::{
    StorageSchedulerRepository,
    export_session::StorageSchedulerExportSession,
    mapping::{map_execution_log, map_execution_log_detail, map_job, map_sqlx_error},
    records::{ExecutionLogDetailRecord, ExecutionLogSummaryRecord, JobRecord},
    sql::{EXECUTION_LOG_DETAIL_COLUMNS, EXECUTION_LOG_SUMMARY_COLUMNS, JOB_COLUMNS},
};

#[async_trait]
impl SchedulerQueryStore for StorageSchedulerRepository {
    async fn find_job(&self, id: &str) -> SchedulerResult<Job> {
        find_job(self.pool(), id).await
    }

    async fn page_jobs(&self, filter: JobListFilter, page: SchedulerCursorQuery) -> SchedulerResult<SchedulerCursorSlice<Job>> {
        let mut connection = self.pool().acquire().await.map_err(map_sqlx_error)?;
        page_jobs_on(&mut connection, filter, page).await
    }

    async fn begin_export(&self) -> SchedulerResult<Box<dyn crate::application::SchedulerQueryExportSession>> {
        Ok(Box::new(StorageSchedulerExportSession::begin(self.pool()).await?))
    }

    async fn task_key_exists(&self, task_key: &str) -> SchedulerResult<bool> {
        query_scalar("SELECT EXISTS(SELECT 1 FROM sys_job WHERE task_key=$1)")
            .bind(task_key)
            .fetch_one(self.pool())
            .await
            .map_err(map_sqlx_error)
    }

    async fn find_execution_log(&self, id: &str) -> SchedulerResult<ExecutionLogSummary> {
        let sql = format!(
            "SELECT {EXECUTION_LOG_SUMMARY_COLUMNS} FROM sys_job_execution WHERE execution_id=$1 AND state='{}'",
            ExecutionState::Terminal.code()
        );
        let record = query_as::<_, ExecutionLogSummaryRecord>(AssertSqlSafe(sql))
            .bind(id)
            .fetch_one(self.pool())
            .await
            .map_err(map_sqlx_error)?;
        map_execution_log(record)
    }

    async fn find_execution_log_detail(&self, id: &str) -> SchedulerResult<ExecutionLogDetail> {
        let sql = format!(
            "SELECT {EXECUTION_LOG_DETAIL_COLUMNS} FROM sys_job_execution WHERE execution_id=$1 AND state='{}'",
            ExecutionState::Terminal.code()
        );
        let record = query_as::<_, ExecutionLogDetailRecord>(AssertSqlSafe(sql))
            .bind(id)
            .fetch_one(self.pool())
            .await
            .map_err(map_sqlx_error)?;
        map_execution_log_detail(record)
    }

    async fn page_execution_logs(&self, filter: JobLogListFilter, page: SchedulerCursorQuery) -> SchedulerResult<SchedulerCursorSlice<ExecutionLogSummary>> {
        let mut connection = self.pool().acquire().await.map_err(map_sqlx_error)?;
        page_execution_logs_on(&mut connection, filter, page).await
    }
}

pub(super) async fn page_jobs_on(
    connection: &mut PgConnection,
    filter: JobListFilter,
    page: SchedulerCursorQuery,
) -> SchedulerResult<SchedulerCursorSlice<Job>> {
    let snapshot = resolve_job_snapshot(connection, page.snapshot.clone()).await?;
    let Some(snapshot) = snapshot else { return Ok(empty_slice()) };
    let mut builder = QueryBuilder::<Postgres>::new(format!("SELECT {JOB_COLUMNS} FROM sys_job WHERE TRUE"));
    push_job_filters(&mut builder, &filter);
    push_window(
        &mut builder,
        WindowSpec {
            id_column: "job_id",
            snapshot: &snapshot,
            page: &page,
        },
    )?;
    push_limit(&mut builder, page.limit)?;
    let records = builder
        .build_query_as::<JobRecord>()
        .fetch_all(&mut *connection)
        .await
        .map_err(map_sqlx_error)?;
    let items = records.into_iter().map(map_job).collect::<SchedulerResult<Vec<_>>>()?;
    slice(items, snapshot, page)
}

pub(super) async fn page_execution_logs_on(
    connection: &mut PgConnection,
    filter: JobLogListFilter,
    page: SchedulerCursorQuery,
) -> SchedulerResult<SchedulerCursorSlice<ExecutionLogSummary>> {
    let state = ExecutionState::Terminal.code();
    let snapshot = resolve_log_snapshot(connection, page.snapshot.clone()).await?;
    let Some(snapshot) = snapshot else { return Ok(empty_slice()) };
    let mut builder = QueryBuilder::<Postgres>::new(format!("SELECT {EXECUTION_LOG_SUMMARY_COLUMNS} FROM sys_job_execution WHERE state="));
    builder.push_bind(state);
    push_execution_filters(&mut builder, &filter);
    push_window(
        &mut builder,
        WindowSpec {
            id_column: "execution_id",
            snapshot: &snapshot,
            page: &page,
        },
    )?;
    push_limit(&mut builder, page.limit)?;
    let records = builder
        .build_query_as::<ExecutionLogSummaryRecord>()
        .fetch_all(&mut *connection)
        .await
        .map_err(map_sqlx_error)?;
    let items = records.into_iter().map(map_execution_log).collect::<SchedulerResult<Vec<_>>>()?;
    slice(items, snapshot, page)
}

pub(super) async fn find_job(pool: &PgPool, id: &str) -> SchedulerResult<Job> {
    let sql = format!("SELECT {JOB_COLUMNS} FROM sys_job WHERE job_id=$1");
    let record = query_as::<_, JobRecord>(AssertSqlSafe(sql))
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(map_sqlx_error)?;
    map_job(record)
}

fn push_job_filters(builder: &mut QueryBuilder<Postgres>, filter: &JobListFilter) {
    push_like(builder, "job_name", filter.name.as_deref());
    push_equal(builder, "job_group", filter.group.as_deref());
    push_equal(builder, "status", filter.status.map(|value| value.code()));
    push_time_range(builder, filter.begin_time, filter.end_time);
}

fn push_execution_filters(builder: &mut QueryBuilder<Postgres>, filter: &JobLogListFilter) {
    push_like(builder, "job_name", filter.name.as_deref());
    push_equal(builder, "job_group", filter.group.as_deref());
    push_equal(builder, "outcome", filter.outcome.map(|value| value.code()));
    push_equal(builder, "trigger_type", filter.trigger.map(|value| value.code()));
    push_time_range(builder, filter.begin_time, filter.end_time);
}

fn push_like(builder: &mut QueryBuilder<Postgres>, column: &'static str, value: Option<&str>) {
    let Some(value) = value else { return };
    builder.push(" AND ").push(column).push(" ILIKE ").push_bind(format!("%{value}%"));
}

fn push_equal(builder: &mut QueryBuilder<Postgres>, column: &'static str, value: Option<&str>) {
    let Some(value) = value else { return };
    builder.push(" AND ").push(column).push("=").push_bind(value.to_owned());
}

fn push_time_range(builder: &mut QueryBuilder<Postgres>, begin: Option<chrono::DateTime<chrono::Utc>>, end: Option<chrono::DateTime<chrono::Utc>>) {
    if let Some(begin) = begin {
        builder.push(" AND create_time>=").push_bind(begin);
    }
    if let Some(end) = end {
        builder.push(" AND create_time<=").push_bind(end);
    }
}

#[cfg(test)]
#[path = "query_tests.rs"]
mod tests;
