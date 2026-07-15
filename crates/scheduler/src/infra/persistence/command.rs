use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgConnection, query, query_scalar};

use crate::{
    application::{
        ManualExecutionRequest, PersistJobReplacement, PersistNewJob, SchedulerCommandStore, SchedulerError, SchedulerResult, UpdateJobStatusCommand,
    },
    domain::{Job, JobStatus, TriggerType},
};

use super::{
    StorageSchedulerRepository,
    mapping::{invalid_record, map_sqlx_error},
    query::find_job,
    write_support::{ExecutionInsert, PendingCancellation, cancel_pending, ensure_manual_execution_can_start, insert_execution, lock_job, notify_change},
};

pub(super) const CANCELLED_EDITED: &str = "scheduler.execution.cancelled_edited";
pub(super) const CANCELLED_PAUSED: &str = "scheduler.execution.cancelled_paused";
pub(super) const CANCELLED_DELETED: &str = "scheduler.execution.cancelled_deleted";

#[async_trait]
impl SchedulerCommandStore for StorageSchedulerRepository {
    async fn insert_job(&self, command: PersistNewJob) -> SchedulerResult<Job> {
        let id = self.database.next_id();
        query(
            r#"INSERT INTO sys_job (
                job_id, job_name, job_group, task_key, task_params, params_schema_version,
                repeatable, invoke_target, cron_expression, misfire_policy, concurrent,
                status, create_by, create_time, remark
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,clock_timestamp(),$14)"#,
        )
        .bind(&id)
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
        .execute(self.pool())
        .await
        .map_err(map_job_write_error)?;
        find_job(self.pool(), &id).await
    }

    async fn replace_job(&self, command: PersistJobReplacement) -> SchedulerResult<Job> {
        let id = command.input.id.clone();
        let mut transaction = self.pool().begin().await.map_err(map_sqlx_error)?;
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
        let rows = update_job(&mut transaction, command).await?;
        ensure_one_row(rows)?;
        notify_change(&mut transaction, &id).await?;
        transaction.commit().await.map_err(map_sqlx_error)?;
        find_job(self.pool(), &id).await
    }

    async fn update_job_status(&self, command: UpdateJobStatusCommand) -> SchedulerResult<Job> {
        let mut transaction = self.pool().begin().await.map_err(map_sqlx_error)?;
        lock_job(&mut transaction, &command.id).await?;
        let now = database_now(&mut transaction).await?;
        if command.status == JobStatus::Paused {
            cancel_pending(
                &mut transaction,
                PendingCancellation {
                    job_id: &command.id,
                    message_key: CANCELLED_PAUSED,
                    ended_at: now,
                },
            )
            .await?;
        }
        let rows = query(
            "UPDATE sys_job SET status=$1, schedule_revision=schedule_revision+1, next_run_at=NULL, runtime_error_code=NULL, runtime_error_time=NULL, update_by=$2, update_time=$3 WHERE job_id=$4",
        )
        .bind(command.status.code())
        .bind(command.operator)
        .bind(now)
        .bind(&command.id)
        .execute(&mut *transaction)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();
        ensure_one_row(rows)?;
        notify_change(&mut transaction, &command.id).await?;
        transaction.commit().await.map_err(map_sqlx_error)?;
        find_job(self.pool(), &command.id).await
    }

    async fn enqueue_manual(&self, request: ManualExecutionRequest) -> SchedulerResult<String> {
        let id = self.database.next_id();
        let mut transaction = self.pool().begin().await.map_err(map_sqlx_error)?;
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
        transaction.commit().await.map_err(map_sqlx_error)?;
        Ok(id)
    }

    async fn delete_job(&self, id: &str) -> SchedulerResult<()> {
        delete_jobs(self, vec![id.to_owned()]).await
    }

    async fn delete_jobs(&self, ids: Vec<String>) -> SchedulerResult<()> {
        delete_jobs(self, ids).await
    }

    async fn delete_execution_log(&self, id: &str) -> SchedulerResult<()> {
        delete_execution_logs(self, vec![id.to_owned()]).await
    }

    async fn delete_execution_logs(&self, ids: Vec<String>) -> SchedulerResult<()> {
        delete_execution_logs(self, ids).await
    }

    async fn clear_execution_logs(&self) -> SchedulerResult<()> {
        let mut transaction = self.pool().begin().await.map_err(map_sqlx_error)?;
        query("LOCK TABLE sys_job_execution IN SHARE ROW EXCLUSIVE MODE")
            .execute(&mut *transaction)
            .await
            .map_err(map_sqlx_error)?;
        let active = query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_job_execution WHERE state<>$1)")
            .bind(crate::domain::ExecutionState::Terminal.code())
            .fetch_one(&mut *transaction)
            .await
            .map_err(map_sqlx_error)?;
        if active {
            transaction.rollback().await.map_err(map_sqlx_error)?;
            return Err(SchedulerError::conflict("scheduler_execution_active", "errors.scheduler.execution_active"));
        }
        query("DELETE FROM sys_job_execution WHERE state=$1")
            .bind(crate::domain::ExecutionState::Terminal.code())
            .execute(&mut *transaction)
            .await
            .map_err(map_sqlx_error)?;
        transaction.commit().await.map_err(map_sqlx_error)?;
        Ok(())
    }
}

pub(super) async fn update_job(connection: &mut PgConnection, command: PersistJobReplacement) -> SchedulerResult<u64> {
    let rows = query(
        r#"UPDATE sys_job SET job_name=$1, job_group=$2, task_params=$3,
        params_schema_version=$4, invoke_target=$5, cron_expression=$6,
        misfire_policy=$7, concurrent=$8, schedule_revision=schedule_revision+1,
        next_run_at=NULL, runtime_error_code=NULL, runtime_error_time=NULL,
        update_by=$9, update_time=clock_timestamp(), remark=$10 WHERE job_id=$11"#,
    )
    .bind(command.input.name)
    .bind(command.input.group)
    .bind(command.input.task_params)
    .bind(command.params_schema_version)
    .bind(command.invoke_target)
    .bind(command.input.cron_expression)
    .bind(command.input.misfire_policy.code())
    .bind(command.input.concurrent.code())
    .bind(command.input.operator)
    .bind(command.input.remark)
    .bind(command.input.id)
    .execute(connection)
    .await
    .map_err(map_sqlx_error)?
    .rows_affected();
    Ok(rows)
}

async fn delete_jobs(repository: &StorageSchedulerRepository, ids: Vec<String>) -> SchedulerResult<()> {
    let mut transaction = repository.pool().begin().await.map_err(map_sqlx_error)?;
    let locked = query_scalar::<_, String>("SELECT job_id FROM sys_job WHERE job_id = ANY($1) ORDER BY job_id FOR UPDATE")
        .bind(&ids)
        .fetch_all(&mut *transaction)
        .await
        .map_err(map_sqlx_error)?;
    if locked.len() != ids.len() {
        return Err(SchedulerError::NotFound);
    }
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
    transaction.commit().await.map_err(map_sqlx_error)?;
    Ok(())
}

async fn delete_execution_logs(repository: &StorageSchedulerRepository, ids: Vec<String>) -> SchedulerResult<()> {
    let active = query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_job_execution WHERE execution_id=ANY($1) AND state<>$2)")
        .bind(&ids)
        .bind(crate::domain::ExecutionState::Terminal.code())
        .fetch_one(repository.pool())
        .await
        .map_err(map_sqlx_error)?;
    if active {
        return Err(SchedulerError::conflict("scheduler_execution_active", "errors.scheduler.execution_active"));
    }
    let rows = query("DELETE FROM sys_job_execution WHERE execution_id=ANY($1) AND state=$2")
        .bind(&ids)
        .bind(crate::domain::ExecutionState::Terminal.code())
        .execute(repository.pool())
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();
    let expected =
        u64::try_from(ids.len()).map_err(|error| SchedulerError::Infrastructure(format!("execution log delete count conversion failed: {error}")))?;
    if rows != expected {
        return Err(SchedulerError::NotFound);
    }
    Ok(())
}

pub(super) async fn database_now(connection: &mut PgConnection) -> SchedulerResult<DateTime<Utc>> {
    query_scalar("SELECT clock_timestamp()").fetch_one(connection).await.map_err(map_sqlx_error)
}

pub(super) fn ensure_one_row(rows: u64) -> SchedulerResult<()> {
    match rows {
        1 => Ok(()),
        0 => Err(SchedulerError::NotFound),
        count => Err(invalid_record(format!("scheduler mutation affected {count} rows"))),
    }
}

pub(super) fn map_job_write_error(error: sqlx::Error) -> SchedulerError {
    if error.as_database_error().and_then(|database| database.constraint()) == Some("idx_sys_job_task_key_singleton") {
        return SchedulerError::conflict("scheduler_task_already_imported", "errors.scheduler.task_already_imported");
    }
    map_sqlx_error(error)
}
