use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{AssertSqlSafe, query, query_as, query_scalar};

use crate::{
    application::{
        ClaimExecutionRequest, Clock, FinishExecutionRequest, InterruptExecutionRequest, OccurrenceAction, OccurrenceRequest, OccurrenceResult,
        RuntimeErrorUpdate, ScheduleInitialization, SchedulerResult, SchedulerRuntimeStore,
    },
    domain::{ConcurrentPolicy, Execution, ExecutionOutcome, ExecutionSnapshot, ExecutionState, Job, JobStatus, LocalizedMessage, TriggerType},
};

use super::{
    StorageSchedulerRepository, execution_store,
    mapping::{invalid_record, map_execution, map_job, map_sqlx_error},
    records::{ExecutionRecord, JobRecord},
    sql::{EXECUTION_COLUMNS, JOB_COLUMNS},
    write_support::{ExecutionInsert, TerminalInsert, has_active_execution, insert_execution, lock_job, notify_change},
};

pub(super) const SKIPPED_OVERLAP: &str = "scheduler.execution.skipped_overlap";
const SKIPPED_MISFIRE: &str = "scheduler.execution.skipped_misfire";

struct OccurrenceInsertContext<'a> {
    id: &'a str,
    job: &'a Job,
    request: &'a OccurrenceRequest,
    now: DateTime<Utc>,
}

pub(super) struct OccurrenceDecision<'a> {
    pub action: &'a OccurrenceAction,
    pub concurrent: ConcurrentPolicy,
    pub has_active: bool,
    pub now: DateTime<Utc>,
}
#[async_trait]
impl Clock for StorageSchedulerRepository {
    async fn now(&self) -> SchedulerResult<DateTime<Utc>> {
        database_now(self).await
    }
}

#[async_trait]
impl SchedulerRuntimeStore for StorageSchedulerRepository {
    async fn database_now(&self) -> SchedulerResult<DateTime<Utc>> {
        database_now(self).await
    }

    async fn schedulable_jobs(&self) -> SchedulerResult<Vec<Job>> {
        let sql = format!("SELECT {JOB_COLUMNS} FROM sys_job WHERE status=$1 ORDER BY job_id");
        let records = query_as::<_, JobRecord>(AssertSqlSafe(sql))
            .bind(JobStatus::Normal.code())
            .fetch_all(self.pool())
            .await
            .map_err(map_sqlx_error)?;
        records.into_iter().map(map_job).collect()
    }

    async fn initialize_schedule(&self, request: ScheduleInitialization) -> SchedulerResult<bool> {
        let rows = query(
            "UPDATE sys_job SET next_run_at=$1, runtime_error_code=NULL, runtime_error_time=NULL \
             WHERE job_id=$2 AND schedule_revision=$3 AND status=$4 AND next_run_at IS NULL",
        )
        .bind(request.next_run_at)
        .bind(request.job_id)
        .bind(request.expected_revision)
        .bind(JobStatus::Normal.code())
        .execute(self.pool())
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();
        Ok(rows == 1)
    }

    async fn materialize_occurrence(&self, request: OccurrenceRequest) -> SchedulerResult<OccurrenceResult> {
        materialize_occurrence(self, request).await
    }

    async fn pending_executions(&self) -> SchedulerResult<Vec<Execution>> {
        executions_by_state(self, ExecutionState::Pending.code()).await
    }

    async fn running_executions(&self) -> SchedulerResult<Vec<Execution>> {
        executions_by_state(self, ExecutionState::Running.code()).await
    }

    async fn claim_execution(&self, request: ClaimExecutionRequest) -> SchedulerResult<Option<Execution>> {
        execution_store::claim_execution(self, request).await
    }

    async fn finish_execution(&self, request: FinishExecutionRequest) -> SchedulerResult<bool> {
        execution_store::finish_execution(self, request).await
    }

    async fn interrupt_execution(&self, request: InterruptExecutionRequest) -> SchedulerResult<bool> {
        execution_store::interrupt_execution(self, request).await
    }

    async fn set_runtime_error(&self, request: RuntimeErrorUpdate) -> SchedulerResult<()> {
        query("UPDATE sys_job SET runtime_error_code=$1, runtime_error_time=$2 WHERE job_id=$3 AND schedule_revision=$4")
            .bind(request.code.code())
            .bind(request.occurred_at)
            .bind(request.job_id)
            .bind(request.expected_revision)
            .execute(self.pool())
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn clear_runtime_error(&self, job_id: &str, expected_revision: i64) -> SchedulerResult<()> {
        query("UPDATE sys_job SET runtime_error_code=NULL, runtime_error_time=NULL WHERE job_id=$1 AND schedule_revision=$2")
            .bind(job_id)
            .bind(expected_revision)
            .execute(self.pool())
            .await
            .map_err(map_sqlx_error)?;
        Ok(())
    }
}

async fn materialize_occurrence(repository: &StorageSchedulerRepository, request: OccurrenceRequest) -> SchedulerResult<OccurrenceResult> {
    let mut transaction = repository.pool().begin().await.map_err(map_sqlx_error)?;
    let job = lock_job(&mut transaction, &request.job_id).await?;
    if !matches_expected_job(&job, &request) {
        return Ok(OccurrenceResult::Stale);
    }
    if occurrence_exists(&mut transaction, &request).await? {
        advance_cursor(&mut transaction, &request).await?;
        transaction.commit().await.map_err(map_sqlx_error)?;
        return Ok(OccurrenceResult::AlreadyMaterialized);
    }
    let now = query_scalar("SELECT clock_timestamp()")
        .fetch_one(&mut *transaction)
        .await
        .map_err(map_sqlx_error)?;
    let id = repository.database.next_id();
    insert_occurrence(
        &mut transaction,
        OccurrenceInsertContext {
            id: &id,
            job: &job,
            request: &request,
            now,
        },
    )
    .await?;
    advance_cursor(&mut transaction, &request).await?;
    notify_change(&mut transaction, &job.id).await?;
    transaction.commit().await.map_err(map_sqlx_error)?;
    Ok(OccurrenceResult::Materialized)
}

async fn insert_occurrence(connection: &mut sqlx::PgConnection, context: OccurrenceInsertContext<'_>) -> SchedulerResult<()> {
    let snapshot = ExecutionSnapshot::from(context.job);
    let (trigger, terminal) = occurrence_insert_values(connection, &context).await?;
    insert_execution(
        connection,
        ExecutionInsert {
            id: context.id,
            snapshot: &snapshot,
            trigger,
            scheduled_at: context.request.expected_due_at,
            requested_by: None,
            terminal,
        },
    )
    .await
}

async fn occurrence_insert_values(
    connection: &mut sqlx::PgConnection,
    context: &OccurrenceInsertContext<'_>,
) -> SchedulerResult<(TriggerType, Option<TerminalInsert>)> {
    let check_overlap = context.job.concurrent == ConcurrentPolicy::Disallow && matches!(context.request.action, OccurrenceAction::Queue(_));
    let has_active = check_overlap && has_active_execution(connection, &context.job.id).await?;
    Ok(occurrence_decision(OccurrenceDecision {
        action: &context.request.action,
        concurrent: context.job.concurrent,
        has_active,
        now: context.now,
    }))
}

pub(super) fn occurrence_decision(decision: OccurrenceDecision<'_>) -> (TriggerType, Option<TerminalInsert>) {
    match decision.action {
        OccurrenceAction::SkipMisfire => terminal_occurrence(TriggerType::Misfire, SKIPPED_MISFIRE, decision.now),
        OccurrenceAction::Queue(trigger) if decision.concurrent == ConcurrentPolicy::Disallow && decision.has_active => {
            terminal_occurrence(*trigger, SKIPPED_OVERLAP, decision.now)
        }
        OccurrenceAction::Queue(trigger) => (*trigger, None),
    }
}

fn terminal_occurrence(trigger: TriggerType, key: &str, now: DateTime<Utc>) -> (TriggerType, Option<TerminalInsert>) {
    (
        trigger,
        Some(TerminalInsert {
            outcome: ExecutionOutcome::Skipped,
            message: LocalizedMessage::new(key),
            error: None,
            ended_at: now,
        }),
    )
}

async fn advance_cursor(connection: &mut sqlx::PgConnection, request: &OccurrenceRequest) -> SchedulerResult<()> {
    let rows = query(
        "UPDATE sys_job SET next_run_at=$1, runtime_error_code=NULL, runtime_error_time=NULL WHERE job_id=$2 AND schedule_revision=$3 AND next_run_at=$4",
    )
    .bind(request.next_run_at)
    .bind(&request.job_id)
    .bind(request.expected_revision)
    .bind(request.expected_due_at)
    .execute(connection)
    .await
    .map_err(map_sqlx_error)?
    .rows_affected();
    if rows != 1 {
        return Err(invalid_record(format!("scheduler cursor update affected {rows} rows")));
    }
    Ok(())
}

async fn occurrence_exists(connection: &mut sqlx::PgConnection, request: &OccurrenceRequest) -> SchedulerResult<bool> {
    query_scalar("SELECT EXISTS(SELECT 1 FROM sys_job_execution WHERE job_id=$1 AND job_revision=$2 AND scheduled_at=$3 AND trigger_type IN ($4,$5))")
        .bind(&request.job_id)
        .bind(request.expected_revision)
        .bind(request.expected_due_at)
        .bind(TriggerType::Scheduled.code())
        .bind(TriggerType::Misfire.code())
        .fetch_one(connection)
        .await
        .map_err(map_sqlx_error)
}

fn matches_expected_job(job: &Job, request: &OccurrenceRequest) -> bool {
    job.schedule_revision == request.expected_revision && job.next_run_at == Some(request.expected_due_at)
}

async fn executions_by_state(repository: &StorageSchedulerRepository, state: &str) -> SchedulerResult<Vec<Execution>> {
    let sql = format!("SELECT {EXECUTION_COLUMNS} FROM sys_job_execution WHERE state=$1 ORDER BY scheduled_at, create_time");
    let records = query_as::<_, ExecutionRecord>(AssertSqlSafe(sql))
        .bind(state)
        .fetch_all(repository.pool())
        .await
        .map_err(map_sqlx_error)?;
    records.into_iter().map(map_execution).collect()
}

async fn database_now(repository: &StorageSchedulerRepository) -> SchedulerResult<DateTime<Utc>> {
    query_scalar("SELECT clock_timestamp()")
        .fetch_one(repository.pool())
        .await
        .map_err(map_sqlx_error)
}
