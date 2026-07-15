use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::{AssertSqlSafe, PgConnection, Postgres, Transaction, query, query_as, query_scalar};

use crate::{
    application::{
        ManualExecutionRequest, SchedulerError, SchedulerResult,
        tasks::{HTTP_REQUEST_TASK_KEY, redacted_http_invoke_target, sanitize_execution_task_params, sanitize_http_invoke_target},
    },
    domain::{ConcurrentPolicy, ExecutionOutcome, ExecutionSnapshot, ExecutionState, Job, LocalizedMessage, TriggerType},
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
    let task_params = persisted_task_params(&input.snapshot.task_key, &input.snapshot.task_params, input.terminal.is_some());
    let invoke_target = persisted_invoke_target(&input.snapshot.task_key, &input.snapshot.invoke_target, input.terminal.is_some());
    let terminal = terminal_values(input.terminal);
    query(INSERT_EXECUTION)
        .bind(input.id)
        .bind(&input.snapshot.job_id)
        .bind(input.snapshot.job_revision)
        .bind(&input.snapshot.job_name)
        .bind(&input.snapshot.job_group)
        .bind(&input.snapshot.task_key)
        .bind(task_params)
        .bind(input.snapshot.params_schema_version)
        .bind(input.snapshot.repeatable)
        .bind(invoke_target)
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

pub async fn ensure_manual_execution_can_start(
    transaction: &mut Transaction<'_, Postgres>,
    job: &Job,
    request: &ManualExecutionRequest,
) -> SchedulerResult<()> {
    if job.schedule_revision != request.expected_revision {
        return Err(SchedulerError::conflict("scheduler_job_changed", "errors.scheduler.job_changed"));
    }
    if job.concurrent == ConcurrentPolicy::Disallow && has_active_execution(transaction, &job.id).await? {
        return Err(SchedulerError::conflict("scheduler_execution_active", "errors.scheduler.execution_active"));
    }
    Ok(())
}

pub async fn cancel_pending(connection: &mut PgConnection, cancellation: PendingCancellation<'_>) -> SchedulerResult<u64> {
    let redacted_target = redacted_http_invoke_target();
    let rows = query(
        "UPDATE sys_job_execution SET state=$1, outcome=$2, message_key=$3, message_params='{}'::jsonb, \
         task_params=CASE WHEN task_key=$5 THEN '{}'::jsonb ELSE task_params END, \
         invoke_target=CASE WHEN task_key=$5 THEN $6 ELSE invoke_target END, \
         end_time=$4 WHERE job_id=$7 AND state=$8",
    )
    .bind(ExecutionState::Terminal.code())
    .bind(ExecutionOutcome::Skipped.code())
    .bind(cancellation.message_key)
    .bind(cancellation.ended_at)
    .bind(HTTP_REQUEST_TASK_KEY)
    .bind(&redacted_target)
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

fn persisted_task_params(task_key: &str, task_params: &Value, terminal: bool) -> Value {
    if terminal {
        return sanitize_execution_task_params(task_key, task_params.clone());
    }
    task_params.clone()
}

fn persisted_invoke_target(task_key: &str, invoke_target: &str, terminal: bool) -> String {
    if terminal {
        return sanitize_http_invoke_target(task_key, invoke_target);
    }
    invoke_target.into()
}

fn map_write_error(error: sqlx::Error) -> SchedulerError {
    if error.as_database_error().and_then(|database| database.constraint()) == Some("idx_sys_job_execution_occurrence") {
        return SchedulerError::conflict("scheduler_occurrence_exists", "errors.scheduler.occurrence_already_materialized");
    }
    map_sqlx_error(error)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{HTTP_REQUEST_TASK_KEY, persisted_invoke_target, persisted_task_params};

    #[test]
    fn terminal_http_execution_storage_drops_credential_bearing_params() {
        let raw = json!({
            "method": "POST",
            "url": "https://url-user:url-password@example.test/run?token=query-token",
            "headers": {"Authorization": "request-token"},
            "body": "request-file-content",
        });

        let pending = persisted_task_params(HTTP_REQUEST_TASK_KEY, &raw, false);
        let terminal = persisted_task_params(HTTP_REQUEST_TASK_KEY, &raw, true);
        let target = "httpClient.request(POST, https://url-user:url-password@example.test/run?token=query-token)";

        assert_eq!(pending, raw);
        assert_eq!(terminal, json!({"method": "POST", "url": "https://example.test/run"}));
        assert_eq!(persisted_invoke_target(HTTP_REQUEST_TASK_KEY, target, false), target);
        assert_eq!(persisted_invoke_target(HTTP_REQUEST_TASK_KEY, target, true), "httpClient.request(...)");
    }
}
