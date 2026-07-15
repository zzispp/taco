use serde_json::Value;
use sqlx::{AssertSqlSafe, PgConnection, query, query_as, query_scalar};

use crate::{
    application::{
        ClaimExecutionRequest, FinishExecutionRequest, InterruptExecutionRequest, SchedulerResult,
        tasks::{HTTP_REQUEST_TASK_KEY, redacted_http_invoke_target},
    },
    domain::{Execution, ExecutionDetail, ExecutionOutcome, ExecutionState, TriggerType},
};

use super::{
    StorageSchedulerRepository,
    mapping::{map_execution, map_sqlx_error, params_value},
    records::ExecutionRecord,
    sql::EXECUTION_COLUMNS,
};

const INTERRUPTED_EXECUTOR_LOST: &str = "scheduler.execution.interrupted_executor_lost";
const CANCELLED_EDITED: &str = "scheduler.execution.cancelled_edited";
const CANCELLED_DELETED: &str = "scheduler.execution.cancelled_deleted";

struct PendingIdentity {
    job_id: String,
    job_revision: i64,
    trigger: TriggerType,
}

#[derive(Debug, PartialEq, Eq)]
enum ClaimDecision {
    Claim,
    Cancel(&'static str),
}

struct FinishValues {
    message_params: Value,
    error_key: Option<String>,
    error_params: Value,
    detail_kind: Option<String>,
    detail_schema_version: Option<i16>,
    detail_payload: Option<Value>,
}

pub async fn claim_execution(repository: &StorageSchedulerRepository, request: ClaimExecutionRequest) -> SchedulerResult<Option<Execution>> {
    let mut transaction = repository.pool().begin().await.map_err(map_sqlx_error)?;
    let Some(identity) = pending_identity(&mut transaction, &request.execution_id).await? else {
        transaction.commit().await.map_err(map_sqlx_error)?;
        return Ok(None);
    };
    let current_revision = lock_current_revision(&mut transaction, &identity.job_id).await?;
    let decision = claim_decision(&identity, current_revision);
    if let ClaimDecision::Cancel(message_key) = decision {
        cancel_stale_pending(&mut transaction, &request.execution_id, message_key).await?;
        transaction.commit().await.map_err(map_sqlx_error)?;
        return Ok(None);
    }
    let sql = format!(
        "UPDATE sys_job_execution SET state=$1, executor_epoch=$2, start_time=$3 \
         WHERE execution_id=$4 AND state=$5 RETURNING {EXECUTION_COLUMNS}"
    );
    let record = query_as::<_, ExecutionRecord>(AssertSqlSafe(sql))
        .bind(ExecutionState::Running.code())
        .bind(request.executor_epoch)
        .bind(request.started_at)
        .bind(request.execution_id)
        .bind(ExecutionState::Pending.code())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(map_sqlx_error)?;
    let execution = record.map(map_execution).transpose()?;
    transaction.commit().await.map_err(map_sqlx_error)?;
    Ok(execution)
}

pub async fn finish_execution(repository: &StorageSchedulerRepository, request: FinishExecutionRequest) -> SchedulerResult<bool> {
    let values = finish_values(&request);
    let redacted_target = redacted_http_invoke_target();
    let rows = query(
        "UPDATE sys_job_execution SET state=$1, outcome=$2, message_key=$3, message_params=$4, \
         error_key=$5, error_params=$6, detail_kind=$7, detail_schema_version=$8, \
         detail_payload=$9, task_params=CASE WHEN task_key=$10 THEN '{}'::jsonb ELSE task_params END, \
         invoke_target=CASE WHEN task_key=$10 THEN $11 ELSE invoke_target END, \
         end_time=$12 WHERE execution_id=$13 AND state=$14",
    )
    .bind(ExecutionState::Terminal.code())
    .bind(request.outcome.code())
    .bind(&request.message.key)
    .bind(&values.message_params)
    .bind(values.error_key.as_deref())
    .bind(&values.error_params)
    .bind(values.detail_kind.as_deref())
    .bind(values.detail_schema_version)
    .bind(&values.detail_payload)
    .bind(HTTP_REQUEST_TASK_KEY)
    .bind(&redacted_target)
    .bind(request.ended_at)
    .bind(&request.execution_id)
    .bind(ExecutionState::Running.code())
    .execute(repository.pool())
    .await
    .map_err(map_sqlx_error)?
    .rows_affected();
    let transitioned = transitioned_once(rows)?;
    let matching_terminal = if transitioned {
        false
    } else {
        terminal_matches(repository, &request, &values).await?
    };
    Ok(terminalization_succeeded(transitioned, matching_terminal))
}

pub async fn interrupt_execution(repository: &StorageSchedulerRepository, request: InterruptExecutionRequest) -> SchedulerResult<bool> {
    let redacted_target = redacted_http_invoke_target();
    let rows = query(
        "UPDATE sys_job_execution SET state=$1, outcome=$2, message_key=$3, \
         message_params='{}'::jsonb, task_params=CASE WHEN task_key=$5 THEN '{}'::jsonb ELSE task_params END, \
         invoke_target=CASE WHEN task_key=$5 THEN $6 ELSE invoke_target END, \
         end_time=$4 WHERE execution_id=$7 AND state=$8",
    )
    .bind(ExecutionState::Terminal.code())
    .bind(ExecutionOutcome::Interrupted.code())
    .bind(INTERRUPTED_EXECUTOR_LOST)
    .bind(request.ended_at)
    .bind(HTTP_REQUEST_TASK_KEY)
    .bind(&redacted_target)
    .bind(request.execution_id)
    .bind(ExecutionState::Running.code())
    .execute(repository.pool())
    .await
    .map_err(map_sqlx_error)?
    .rows_affected();
    Ok(rows == 1)
}

async fn pending_identity(connection: &mut PgConnection, execution_id: &str) -> SchedulerResult<Option<PendingIdentity>> {
    let row = query_as::<_, (String, i64, String)>("SELECT job_id, job_revision, trigger_type FROM sys_job_execution WHERE execution_id=$1 AND state=$2")
        .bind(execution_id)
        .bind(ExecutionState::Pending.code())
        .fetch_optional(connection)
        .await
        .map_err(map_sqlx_error)?;
    row.map(|(job_id, job_revision, trigger)| {
        let trigger =
            TriggerType::parse(&trigger).ok_or_else(|| super::mapping::invalid_record(format!("invalid pending execution trigger code: {trigger}")))?;
        Ok(PendingIdentity { job_id, job_revision, trigger })
    })
    .transpose()
}

async fn lock_current_revision(connection: &mut PgConnection, job_id: &str) -> SchedulerResult<Option<i64>> {
    query_scalar::<_, i64>("SELECT schedule_revision FROM sys_job WHERE job_id=$1 FOR UPDATE")
        .bind(job_id)
        .fetch_optional(connection)
        .await
        .map_err(map_sqlx_error)
}

fn claim_decision(identity: &PendingIdentity, current_revision: Option<i64>) -> ClaimDecision {
    match current_revision {
        None => ClaimDecision::Cancel(CANCELLED_DELETED),
        Some(_) if identity.trigger == TriggerType::Manual => ClaimDecision::Claim,
        Some(revision) if revision == identity.job_revision => ClaimDecision::Claim,
        Some(_) => ClaimDecision::Cancel(CANCELLED_EDITED),
    }
}

async fn cancel_stale_pending(connection: &mut PgConnection, execution_id: &str, message_key: &str) -> SchedulerResult<()> {
    let redacted_target = redacted_http_invoke_target();
    query(
        "UPDATE sys_job_execution SET state=$1, outcome=$2, message_key=$3, \
         message_params='{}'::jsonb, task_params=CASE WHEN task_key=$4 THEN '{}'::jsonb ELSE task_params END, \
         invoke_target=CASE WHEN task_key=$4 THEN $5 ELSE invoke_target END, \
         end_time=clock_timestamp() WHERE execution_id=$6 AND state=$7",
    )
    .bind(ExecutionState::Terminal.code())
    .bind(ExecutionOutcome::Skipped.code())
    .bind(message_key)
    .bind(HTTP_REQUEST_TASK_KEY)
    .bind(&redacted_target)
    .bind(execution_id)
    .bind(ExecutionState::Pending.code())
    .execute(connection)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

async fn terminal_matches(repository: &StorageSchedulerRepository, request: &FinishExecutionRequest, values: &FinishValues) -> SchedulerResult<bool> {
    query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sys_job_execution WHERE execution_id=$1 AND state=$2 \
         AND outcome=$3 AND message_key=$4 AND message_params=$5 AND error_key IS NOT DISTINCT FROM $6 \
         AND error_params=$7 AND detail_kind IS NOT DISTINCT FROM $8 \
         AND detail_schema_version IS NOT DISTINCT FROM $9 AND detail_payload IS NOT DISTINCT FROM $10 \
         AND end_time=$11)",
    )
    .bind(&request.execution_id)
    .bind(ExecutionState::Terminal.code())
    .bind(request.outcome.code())
    .bind(&request.message.key)
    .bind(&values.message_params)
    .bind(values.error_key.as_deref())
    .bind(&values.error_params)
    .bind(values.detail_kind.as_deref())
    .bind(values.detail_schema_version)
    .bind(&values.detail_payload)
    .bind(request.ended_at)
    .fetch_one(repository.pool())
    .await
    .map_err(map_sqlx_error)
}

fn finish_values(request: &FinishExecutionRequest) -> FinishValues {
    let error_key = request.error.as_ref().map(|error| error.key.clone());
    let error_params = request.error.as_ref().map(params_value).unwrap_or_else(empty_params);
    let detail_kind = request.detail.as_ref().map(|detail| detail.kind().to_owned());
    let detail_schema_version = request.detail.as_ref().map(ExecutionDetail::schema_version);
    let detail_payload = request.detail.as_ref().map(|detail| detail.payload().clone());
    FinishValues {
        message_params: params_value(&request.message),
        error_key,
        error_params,
        detail_kind,
        detail_schema_version,
        detail_payload,
    }
}

fn transitioned_once(rows: u64) -> SchedulerResult<bool> {
    match rows {
        0 => Ok(false),
        1 => Ok(true),
        count => Err(super::mapping::invalid_record(format!("execution terminalization affected {count} rows"))),
    }
}

fn terminalization_succeeded(transitioned: bool, matching_terminal: bool) -> bool {
    transitioned || matching_terminal
}

fn empty_params() -> Value {
    serde_json::json!({})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending(trigger: TriggerType, revision: i64) -> PendingIdentity {
        PendingIdentity {
            job_id: "job-1".into(),
            job_revision: revision,
            trigger,
        }
    }

    #[test]
    fn scheduled_stale_revision_is_terminalized() {
        let decision = claim_decision(&pending(TriggerType::Scheduled, 1), Some(2));
        assert_eq!(decision, ClaimDecision::Cancel(CANCELLED_EDITED));
    }

    #[test]
    fn manual_execution_survives_status_only_revision_change() {
        let decision = claim_decision(&pending(TriggerType::Manual, 1), Some(2));
        assert_eq!(decision, ClaimDecision::Claim);
    }

    #[test]
    fn pending_execution_without_job_is_terminalized() {
        let decision = claim_decision(&pending(TriggerType::Manual, 1), None);
        assert_eq!(decision, ClaimDecision::Cancel(CANCELLED_DELETED));
    }

    #[test]
    fn uncertain_terminalization_retry_accepts_only_the_same_terminal_result() {
        assert!(transitioned_once(1).unwrap());
        assert!(!transitioned_once(0).unwrap());
        assert!(terminalization_succeeded(false, true));
        assert!(!terminalization_succeeded(false, false));
        assert!(transitioned_once(2).is_err());
    }
}
