use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{AssertSqlSafe, PgConnection, query, query_as, query_scalar};

use crate::{
    application::{SchedulerError, SchedulerResult},
    domain::{ExecutionOutcome, ExecutionSnapshot, ExecutionState, LocalizedMessage, TriggerType},
};

use super::{
    mapping::{map_job, map_sqlx_error, params_value},
    records::JobRecord,
    sql::{INSERT_EXECUTION, JOB_COLUMNS, NOTIFY_CHANGE},
};

pub struct ExecutionInsert<'a> {
    pub id: &'a str,
    pub snapshot: &'a ExecutionSnapshot,
    pub trigger: TriggerType,
    pub scheduled_at: DateTime<Utc>,
    pub requested_by: Option<&'a str>,
    pub terminal: Option<TerminalInsert>,
}

pub struct TerminalInsert {
    pub outcome: ExecutionOutcome,
    pub message: LocalizedMessage,
    pub error: Option<LocalizedMessage>,
    pub ended_at: DateTime<Utc>,
}

pub struct PendingCancellation<'a> {
    pub job_id: &'a str,
    pub message_key: &'a str,
    pub ended_at: DateTime<Utc>,
}

struct TerminalValues {
    state: &'static str,
    outcome: Option<&'static str>,
    message_key: Option<String>,
    message_params: serde_json::Value,
    error_key: Option<String>,
    error_params: serde_json::Value,
    end_time: Option<DateTime<Utc>>,
}

pub async fn insert_execution(connection: &mut PgConnection, input: ExecutionInsert<'_>) -> SchedulerResult<()> {
    let terminal = terminal_values(input.terminal);
    query(INSERT_EXECUTION)
        .bind(input.id)
        .bind(&input.snapshot.job_id)
        .bind(input.snapshot.job_revision)
        .bind(&input.snapshot.job_name)
        .bind(&input.snapshot.job_group)
        .bind(&input.snapshot.task_key)
        .bind(&input.snapshot.task_params)
        .bind(input.snapshot.params_schema_version)
        .bind(input.snapshot.repeatable)
        .bind(&input.snapshot.invoke_target)
        .bind(input.snapshot.concurrent.code())
        .bind(input.trigger.code())
        .bind(input.scheduled_at)
        .bind(terminal.state)
        .bind(terminal.outcome)
        .bind(Option::<&str>::None)
        .bind(input.requested_by)
        .bind(terminal.message_key)
        .bind(terminal.message_params)
        .bind(terminal.error_key)
        .bind(terminal.error_params)
        .bind(Option::<DateTime<Utc>>::None)
        .bind(terminal.end_time)
        .execute(connection)
        .await
        .map_err(map_write_error)?;
    Ok(())
}

pub async fn lock_job(connection: &mut PgConnection, id: &str) -> SchedulerResult<crate::domain::Job> {
    let sql = format!("SELECT {JOB_COLUMNS} FROM sys_job WHERE job_id=$1 FOR UPDATE");
    let record = query_as::<_, JobRecord>(AssertSqlSafe(sql))
        .bind(id)
        .fetch_one(connection)
        .await
        .map_err(map_sqlx_error)?;
    map_job(record)
}

pub async fn cancel_pending(connection: &mut PgConnection, cancellation: PendingCancellation<'_>) -> SchedulerResult<u64> {
    let rows = query(
        "UPDATE sys_job_execution SET state=$1, outcome=$2, message_key=$3, message_params='{}'::jsonb, \
         end_time=$4 WHERE job_id=$5 AND state=$6",
    )
    .bind(ExecutionState::Terminal.code())
    .bind(ExecutionOutcome::Skipped.code())
    .bind(cancellation.message_key)
    .bind(cancellation.ended_at)
    .bind(cancellation.job_id)
    .bind(ExecutionState::Pending.code())
    .execute(connection)
    .await
    .map_err(map_sqlx_error)?
    .rows_affected();
    Ok(rows)
}

pub async fn has_active_execution(connection: &mut PgConnection, job_id: &str) -> SchedulerResult<bool> {
    query_scalar("SELECT EXISTS(SELECT 1 FROM sys_job_execution WHERE job_id=$1 AND state IN ($2,$3))")
        .bind(job_id)
        .bind(ExecutionState::Pending.code())
        .bind(ExecutionState::Running.code())
        .fetch_one(connection)
        .await
        .map_err(map_sqlx_error)
}

pub async fn notify_change(connection: &mut PgConnection, job_id: &str) -> SchedulerResult<()> {
    query(NOTIFY_CHANGE).bind(job_id).execute(connection).await.map_err(map_sqlx_error)?;
    Ok(())
}

fn terminal_values(terminal: Option<TerminalInsert>) -> TerminalValues {
    let Some(terminal) = terminal else {
        return TerminalValues {
            state: ExecutionState::Pending.code(),
            outcome: None,
            message_key: None,
            message_params: json!({}),
            error_key: None,
            error_params: json!({}),
            end_time: None,
        };
    };
    let error_key = terminal.error.as_ref().map(|error| error.key.clone());
    let error_params = terminal.error.as_ref().map(params_value).unwrap_or_else(|| json!({}));
    TerminalValues {
        state: ExecutionState::Terminal.code(),
        outcome: Some(terminal.outcome.code()),
        message_key: Some(terminal.message.key.clone()),
        message_params: params_value(&terminal.message),
        error_key,
        error_params,
        end_time: Some(terminal.ended_at),
    }
}

fn map_write_error(error: sqlx::Error) -> SchedulerError {
    if error.as_database_error().and_then(|database| database.constraint()) == Some("idx_sys_job_execution_occurrence") {
        return SchedulerError::conflict("scheduler_occurrence_exists", "errors.scheduler.occurrence_already_materialized");
    }
    map_sqlx_error(error)
}
