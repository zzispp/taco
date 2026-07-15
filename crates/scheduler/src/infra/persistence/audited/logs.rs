use audit_contract::AuditOutboxRecord;
use sqlx::{Postgres, Transaction, query, query_scalar};
use storage::outbox::append_audit_record;

use crate::{
    application::{SchedulerError, SchedulerResult},
    domain::ExecutionState,
};

use super::super::{StorageSchedulerRepository, mapping::map_sqlx_error};

pub(super) async fn delete_execution_logs(repository: &StorageSchedulerRepository, ids: Vec<String>, audit: &AuditOutboxRecord) -> SchedulerResult<()> {
    let mut transaction = repository.pool().begin().await.map_err(map_sqlx_error)?;
    lock_terminal_execution_logs(&mut transaction, &ids).await?;
    query("DELETE FROM sys_job_execution WHERE execution_id=ANY($1) AND state=$2")
        .bind(&ids)
        .bind(ExecutionState::Terminal.code())
        .execute(&mut *transaction)
        .await
        .map_err(map_sqlx_error)?;
    commit_with_audit(transaction, audit).await
}

pub(super) async fn clear_execution_logs(repository: &StorageSchedulerRepository, audit: &AuditOutboxRecord) -> SchedulerResult<()> {
    let mut transaction = repository.pool().begin().await.map_err(map_sqlx_error)?;
    query("LOCK TABLE sys_job_execution IN SHARE ROW EXCLUSIVE MODE")
        .execute(&mut *transaction)
        .await
        .map_err(map_sqlx_error)?;
    let active = query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM sys_job_execution WHERE state<>$1)")
        .bind(ExecutionState::Terminal.code())
        .fetch_one(&mut *transaction)
        .await
        .map_err(map_sqlx_error)?;
    if active {
        return Err(SchedulerError::conflict("scheduler_execution_active", "errors.scheduler.execution_active"));
    }
    query("DELETE FROM sys_job_execution WHERE state=$1")
        .bind(ExecutionState::Terminal.code())
        .execute(&mut *transaction)
        .await
        .map_err(map_sqlx_error)?;
    commit_with_audit(transaction, audit).await
}

async fn lock_terminal_execution_logs(transaction: &mut Transaction<'_, Postgres>, ids: &[String]) -> SchedulerResult<()> {
    let states = query_scalar::<_, String>("SELECT state FROM sys_job_execution WHERE execution_id=ANY($1) FOR UPDATE")
        .bind(ids)
        .fetch_all(&mut **transaction)
        .await
        .map_err(map_sqlx_error)?;
    if states.len() != ids.len() {
        return Err(SchedulerError::NotFound);
    }
    if states.iter().all(|state| state == ExecutionState::Terminal.code()) {
        return Ok(());
    }
    Err(SchedulerError::conflict("scheduler_execution_active", "errors.scheduler.execution_active"))
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
