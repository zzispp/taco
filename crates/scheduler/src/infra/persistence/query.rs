use async_trait::async_trait;
use kernel::pagination::{Page, PageSliceRequest};
use sqlx::{AssertSqlSafe, PgPool, query_as, query_scalar};

use crate::{
    application::{ExecutionLogDetail, ExecutionLogSummary, SchedulerQueryStore, SchedulerResult},
    domain::{ExecutionState, Job, JobListFilter, JobLogListFilter},
};

use super::{
    StorageSchedulerRepository,
    mapping::{map_execution_log, map_execution_log_detail, map_job, map_sqlx_error},
    records::{ExecutionLogDetailRecord, ExecutionLogSummaryRecord, JobRecord},
    sql::{EXECUTION_LOG_DETAIL_COLUMNS, EXECUTION_LOG_SUMMARY_COLUMNS, JOB_COLUMNS},
};

#[async_trait]
impl SchedulerQueryStore for StorageSchedulerRepository {
    async fn find_job(&self, id: &str) -> SchedulerResult<Job> {
        find_job(self.pool(), id).await
    }

    async fn page_jobs(&self, filter: JobListFilter, page: PageSliceRequest) -> SchedulerResult<Page<Job>> {
        let total = job_count(self.pool(), &filter).await?;
        let sql = format!(
            "SELECT {JOB_COLUMNS} FROM sys_job WHERE {} ORDER BY create_time DESC LIMIT $6 OFFSET $7",
            job_predicate()
        );
        let records = bind_job_filter(query_as::<_, JobRecord>(AssertSqlSafe(sql)), &filter)
            .bind(to_i64(page.limit)?)
            .bind(to_i64(page.offset)?)
            .fetch_all(self.pool())
            .await
            .map_err(map_sqlx_error)?;
        Ok(Page {
            items: records.into_iter().map(map_job).collect::<SchedulerResult<_>>()?,
            total: to_u64(total)?,
            page: page.page,
            page_size: page.page_size,
        })
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

    async fn page_execution_logs(&self, filter: JobLogListFilter, page: PageSliceRequest) -> SchedulerResult<Page<ExecutionLogSummary>> {
        let total = execution_count(self.pool(), &filter).await?;
        let sql = format!(
            "SELECT {EXECUTION_LOG_SUMMARY_COLUMNS} FROM sys_job_execution WHERE {} ORDER BY create_time DESC LIMIT $7 OFFSET $8",
            execution_predicate()
        );
        let records = bind_execution_filter(query_as::<_, ExecutionLogSummaryRecord>(AssertSqlSafe(sql)), &filter)
            .bind(to_i64(page.limit)?)
            .bind(to_i64(page.offset)?)
            .fetch_all(self.pool())
            .await
            .map_err(map_sqlx_error)?;
        Ok(Page {
            items: records.into_iter().map(map_execution_log).collect::<SchedulerResult<_>>()?,
            total: to_u64(total)?,
            page: page.page,
            page_size: page.page_size,
        })
    }
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

async fn job_count(pool: &PgPool, filter: &JobListFilter) -> SchedulerResult<i64> {
    let sql = format!("SELECT COUNT(*) FROM sys_job WHERE {}", job_predicate());
    let row = bind_job_filter(query_as::<_, (i64,)>(AssertSqlSafe(sql)), filter)
        .fetch_one(pool)
        .await
        .map_err(map_sqlx_error)?;
    Ok(row.0)
}

async fn execution_count(pool: &PgPool, filter: &JobLogListFilter) -> SchedulerResult<i64> {
    let sql = format!("SELECT COUNT(*) FROM sys_job_execution WHERE {}", execution_predicate());
    let row = bind_execution_filter(query_as::<_, (i64,)>(AssertSqlSafe(sql)), filter)
        .fetch_one(pool)
        .await
        .map_err(map_sqlx_error)?;
    Ok(row.0)
}

fn job_predicate() -> &'static str {
    "($1::text IS NULL OR job_name ILIKE '%' || $1 || '%') AND ($2::text IS NULL OR job_group=$2) AND ($3::text IS NULL OR status=$3) AND ($4::timestamptz IS NULL OR create_time >= $4) AND ($5::timestamptz IS NULL OR create_time <= $5)"
}

fn execution_predicate() -> String {
    format!(
        "state='{}' AND ($1::text IS NULL OR job_name ILIKE '%' || $1 || '%') AND \
         ($2::text IS NULL OR job_group=$2) AND ($3::text IS NULL OR outcome=$3) AND \
         ($4::text IS NULL OR trigger_type=$4) AND ($5::timestamptz IS NULL OR create_time >= $5) AND \
         ($6::timestamptz IS NULL OR create_time <= $6)",
        ExecutionState::Terminal.code()
    )
}

fn bind_job_filter<'q, O>(
    query: sqlx::query::QueryAs<'q, sqlx::Postgres, O, sqlx::postgres::PgArguments>,
    filter: &'q JobListFilter,
) -> sqlx::query::QueryAs<'q, sqlx::Postgres, O, sqlx::postgres::PgArguments>
where
    O: Send + Unpin,
{
    query
        .bind(filter.name.as_deref())
        .bind(filter.group.as_deref())
        .bind(filter.status.map(|status| status.code()))
        .bind(filter.begin_time)
        .bind(filter.end_time)
}

fn bind_execution_filter<'q, O>(
    query: sqlx::query::QueryAs<'q, sqlx::Postgres, O, sqlx::postgres::PgArguments>,
    filter: &'q JobLogListFilter,
) -> sqlx::query::QueryAs<'q, sqlx::Postgres, O, sqlx::postgres::PgArguments>
where
    O: Send + Unpin,
{
    query
        .bind(filter.name.as_deref())
        .bind(filter.group.as_deref())
        .bind(filter.outcome.map(|outcome| outcome.code()))
        .bind(execution_trigger_code(filter))
        .bind(filter.begin_time)
        .bind(filter.end_time)
}

fn execution_trigger_code(filter: &JobLogListFilter) -> Option<&'static str> {
    filter.trigger.map(|trigger| trigger.code())
}

fn to_i64(value: u64) -> SchedulerResult<i64> {
    i64::try_from(value).map_err(|error| crate::application::SchedulerError::Infrastructure(format!("pagination conversion failed: {error}")))
}

fn to_u64(value: i64) -> SchedulerResult<u64> {
    u64::try_from(value).map_err(|error| crate::application::SchedulerError::Infrastructure(format!("count conversion failed: {error}")))
}

#[cfg(test)]
mod tests {
    use crate::domain::{JobLogListFilter, TriggerType};

    use super::{execution_predicate, execution_trigger_code};

    #[test]
    fn execution_filter_uses_trigger_code_and_create_time() {
        let predicate = execution_predicate();
        assert!(predicate.contains("trigger_type=$4"));
        assert!(predicate.contains("create_time >= $5"));
        assert!(predicate.contains("create_time <= $6"));

        for (trigger, code) in [(TriggerType::Scheduled, "S"), (TriggerType::Misfire, "F"), (TriggerType::Manual, "M")] {
            let filter = JobLogListFilter {
                trigger: Some(trigger),
                ..JobLogListFilter::default()
            };
            assert_eq!(execution_trigger_code(&filter), Some(code));
        }
    }
}
