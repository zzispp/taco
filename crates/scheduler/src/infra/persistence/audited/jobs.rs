use audit_contract::AuditOutboxRecord;
use sqlx::{Postgres, Transaction, query, query_scalar};
use storage::outbox::append_audit_record;

use crate::{
    application::{ManualExecutionRequest, PersistJobReplacement, PersistNewJob, SchedulerError, SchedulerResult, UpdateJobStatusCommand},
    domain::{Job, JobStatus, TriggerType},
};

use super::super::{
    StorageSchedulerRepository,
    command::{CANCELLED_DELETED, CANCELLED_EDITED, CANCELLED_PAUSED, database_now, ensure_one_row, map_job_write_error, update_job},
    mapping::map_sqlx_error,
    query::find_job,
    write_support::{ExecutionInsert, PendingCancellation, cancel_pending, ensure_manual_execution_can_start, insert_execution, lock_job, notify_change},
};

pub(super) async fn insert_job(repository: &StorageSchedulerRepository, command: PersistNewJob, audit: &AuditOutboxRecord) -> SchedulerResult<Job> {
    let id = repository.database.next_id();
    let mut transaction = repository.pool().begin().await.map_err(map_sqlx_error)?;
    insert_job_record(&mut transaction, &id, command).await?;
    commit_with_audit(transaction, audit).await?;
    find_job(repository.pool(), &id).await
}

pub(super) async fn replace_job(repository: &StorageSchedulerRepository, command: PersistJobReplacement, audit: &AuditOutboxRecord) -> SchedulerResult<Job> {
    let id = command.input.id.clone();
    let mut transaction = repository.pool().begin().await.map_err(map_sqlx_error)?;
    lock_job(&mut transaction, &id).await?;
    let now = database_now(&mut transaction).await?;
    cancel_pending(
        &mut transaction,
        PendingCancellation {
            job_id: &id,
            message_key: CANCELLED_EDITED,
            ended_at: now,
        },
    )
    .await?;
    ensure_one_row(update_job(&mut transaction, command).await?)?;
    notify_change(&mut transaction, &id).await?;
    commit_with_audit(transaction, audit).await?;
    find_job(repository.pool(), &id).await
}

pub(super) async fn update_job_status(
    repository: &StorageSchedulerRepository,
    command: UpdateJobStatusCommand,
    audit: &AuditOutboxRecord,
) -> SchedulerResult<Job> {
    let id = command.id.clone();
    let mut transaction = repository.pool().begin().await.map_err(map_sqlx_error)?;
    lock_job(&mut transaction, &id).await?;
    let now = database_now(&mut transaction).await?;
    if command.status == JobStatus::Paused {
        cancel_pending(
            &mut transaction,
            PendingCancellation {
                job_id: &id,
                message_key: CANCELLED_PAUSED,
                ended_at: now,
            },
        )
        .await?;
    }
    update_status_record(&mut transaction, command, now).await?;
    notify_change(&mut transaction, &id).await?;
    commit_with_audit(transaction, audit).await?;
    find_job(repository.pool(), &id).await
}

pub(super) async fn enqueue_manual(
    repository: &StorageSchedulerRepository,
    request: ManualExecutionRequest,
    audit: &AuditOutboxRecord,
) -> SchedulerResult<String> {
    let id = repository.database.next_id();
    let mut transaction = repository.pool().begin().await.map_err(map_sqlx_error)?;
    let job = lock_job(&mut transaction, &request.snapshot.job_id).await?;
    ensure_manual_execution_can_start(&mut transaction, &job, &request).await?;
    insert_execution(
        &mut transaction,
        ExecutionInsert {
            id: &id,
            snapshot: &request.snapshot,
            trigger: TriggerType::Manual,
            scheduled_at: request.scheduled_at,
            requested_by: Some(&request.requested_by),
            terminal: None,
        },
    )
    .await?;
    notify_change(&mut transaction, &job.id).await?;
    commit_with_audit(transaction, audit).await?;
    Ok(id)
}

pub(super) async fn delete_jobs(repository: &StorageSchedulerRepository, ids: Vec<String>, audit: &AuditOutboxRecord) -> SchedulerResult<()> {
    let mut transaction = repository.pool().begin().await.map_err(map_sqlx_error)?;
    lock_jobs(&mut transaction, &ids).await?;
    let now = database_now(&mut transaction).await?;
    for id in &ids {
        cancel_pending(
            &mut transaction,
            PendingCancellation {
                job_id: id,
                message_key: CANCELLED_DELETED,
                ended_at: now,
            },
        )
        .await?;
    }
    query("DELETE FROM sys_job WHERE job_id = ANY($1)")
        .bind(&ids)
        .execute(&mut *transaction)
        .await
        .map_err(map_sqlx_error)?;
    notify_change(&mut transaction, "jobs_deleted").await?;
    commit_with_audit(transaction, audit).await
}

async fn insert_job_record(transaction: &mut Transaction<'_, Postgres>, id: &str, command: PersistNewJob) -> SchedulerResult<()> {
    query(
        r#"INSERT INTO sys_job (
            job_id, job_name, job_group, task_key, task_params, params_schema_version,
            repeatable, invoke_target, cron_expression, misfire_policy, concurrent,
            status, create_by, create_time, remark
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,clock_timestamp(),$14)"#,
    )
    .bind(id)
    .bind(command.input.name)
    .bind(command.input.group)
    .bind(command.input.task_key)
    .bind(command.input.task_params)
    .bind(command.params_schema_version)
    .bind(command.repeatable)
    .bind(command.invoke_target)
    .bind(command.input.cron_expression)
    .bind(command.input.misfire_policy.code())
    .bind(command.input.concurrent.code())
    .bind(JobStatus::Paused.code())
    .bind(command.input.operator)
    .bind(command.input.remark)
    .execute(&mut **transaction)
    .await
    .map_err(map_job_write_error)?;
    Ok(())
}

async fn update_status_record(
    transaction: &mut Transaction<'_, Postgres>,
    command: UpdateJobStatusCommand,
    now: chrono::DateTime<chrono::Utc>,
) -> SchedulerResult<()> {
    let rows = query(
        "UPDATE sys_job SET status=$1, schedule_revision=schedule_revision+1, next_run_at=NULL, runtime_error_code=NULL, runtime_error_time=NULL, update_by=$2, update_time=$3 WHERE job_id=$4",
    )
    .bind(command.status.code())
    .bind(command.operator)
    .bind(now)
    .bind(command.id)
    .execute(&mut **transaction)
    .await
    .map_err(map_sqlx_error)?
    .rows_affected();
    ensure_one_row(rows)
}

async fn lock_jobs(transaction: &mut Transaction<'_, Postgres>, ids: &[String]) -> SchedulerResult<()> {
    let locked = query_scalar::<_, String>("SELECT job_id FROM sys_job WHERE job_id = ANY($1) ORDER BY job_id FOR UPDATE")
        .bind(ids)
        .fetch_all(&mut **transaction)
        .await
        .map_err(map_sqlx_error)?;
    if locked.len() == ids.len() {
        return Ok(());
    }
    Err(SchedulerError::NotFound)
}

async fn commit_with_audit(mut transaction: Transaction<'_, Postgres>, audit: &AuditOutboxRecord) -> SchedulerResult<()> {
    match append_audit_record(&mut transaction, audit).await {
        Ok(()) => transaction.commit().await.map_err(map_sqlx_error),
        Err(error) => {
            transaction.rollback().await.map_err(map_sqlx_error)?;
            Err(SchedulerError::Infrastructure(error.to_string()))
        }
    }
}
