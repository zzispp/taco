use std::collections::BTreeMap;

use serde_json::Value;

use crate::{
    application::{ExecutionLogDetail, ExecutionLogSummary, SchedulerError, SchedulerResult},
    domain::{
        ConcurrentPolicy, Execution, ExecutionDetail, ExecutionOutcome, ExecutionSnapshot, ExecutionState, Job, JobStatus, LocalizedMessage, MisfirePolicy,
        RuntimeErrorCode, RuntimeErrorState, TriggerType,
    },
};

use super::records::{ExecutionLogDetailRecord, ExecutionLogSummaryRecord, ExecutionRecord, JobRecord};

pub fn map_job(record: JobRecord) -> SchedulerResult<Job> {
    let runtime_error = match (record.runtime_error_code, record.runtime_error_time) {
        (Some(code), Some(occurred_at)) => Some(RuntimeErrorState {
            code: parse_code(&code, RuntimeErrorCode::parse, "runtime_error_code")?,
            occurred_at,
        }),
        (None, None) => None,
        _ => return Err(invalid_record("sys_job runtime error columns are inconsistent")),
    };
    Ok(Job {
        id: record.job_id,
        name: record.job_name,
        group: record.job_group,
        task_key: record.task_key,
        task_params: record.task_params,
        params_schema_version: record.params_schema_version,
        repeatable: record.repeatable,
        invoke_target: record.invoke_target,
        cron_expression: record.cron_expression,
        misfire_policy: parse_code(&record.misfire_policy, MisfirePolicy::parse, "misfire_policy")?,
        concurrent: parse_code(&record.concurrent, ConcurrentPolicy::parse, "concurrent")?,
        status: parse_code(&record.status, JobStatus::parse, "status")?,
        schedule_revision: record.schedule_revision,
        next_run_at: record.next_run_at,
        runtime_error,
        create_by: record.create_by,
        create_time: record.create_time,
        update_by: record.update_by,
        update_time: record.update_time,
        remark: record.remark,
    })
}

pub fn map_execution(record: ExecutionRecord) -> SchedulerResult<Execution> {
    let message = localized_message(record.message_key, record.message_params)?;
    let error = localized_message(record.error_key, record.error_params)?;
    Ok(Execution {
        id: record.execution_id,
        snapshot: ExecutionSnapshot {
            job_id: record.job_id,
            job_revision: record.job_revision,
            job_name: record.job_name,
            job_group: record.job_group,
            task_key: record.task_key,
            task_params: record.task_params,
            params_schema_version: record.params_schema_version,
            repeatable: record.repeatable,
            invoke_target: record.invoke_target,
            concurrent: parse_code(&record.concurrent, ConcurrentPolicy::parse, "execution concurrent")?,
        },
        trigger: parse_code(&record.trigger_type, TriggerType::parse, "trigger_type")?,
        scheduled_at: record.scheduled_at,
        state: parse_code(&record.state, ExecutionState::parse, "execution state")?,
        outcome: record
            .outcome
            .map(|value| parse_code(&value, ExecutionOutcome::parse, "execution outcome"))
            .transpose()?,
        executor_epoch: record.executor_epoch,
        requested_by: record.requested_by,
        message,
        error,
        start_time: record.start_time,
        end_time: record.end_time,
        create_time: record.create_time,
    })
}

pub fn map_execution_log(record: ExecutionLogSummaryRecord) -> SchedulerResult<ExecutionLogSummary> {
    let outcome = required_code(record.outcome, ExecutionOutcome::parse, "execution log outcome")?;
    let message = required_message(record.message_key, record.message_params, "execution log message")?;
    let error = localized_message(record.error_key, record.error_params)?;
    let end_time = record.end_time.ok_or_else(|| invalid_record("terminal execution log has no end time"))?;
    Ok(ExecutionLogSummary {
        id: record.execution_id,
        job_id: record.job_id,
        job_name: record.job_name,
        job_group: record.job_group,
        task_key: record.task_key,
        invoke_target: record.invoke_target,
        trigger: parse_code(&record.trigger_type, TriggerType::parse, "execution log trigger")?,
        scheduled_at: record.scheduled_at,
        outcome,
        message,
        error,
        start_time: record.start_time,
        end_time,
        create_time: record.create_time,
        has_detail: record.has_detail,
    })
}

pub fn map_execution_log_detail(record: ExecutionLogDetailRecord) -> SchedulerResult<ExecutionLogDetail> {
    let detail = map_execution_detail(
        record.summary.has_detail,
        (record.detail_kind, record.detail_schema_version, record.detail_payload),
    )?;
    Ok(ExecutionLogDetail {
        summary: map_execution_log(record.summary)?,
        job_revision: record.job_revision,
        requested_by: record.requested_by,
        task_params: record.task_params,
        detail,
    })
}

pub fn params_value(message: &LocalizedMessage) -> Value {
    serde_json::to_value(&message.params).expect("localized message parameters are serializable")
}

fn localized_message(key: Option<String>, params: Value) -> SchedulerResult<Option<LocalizedMessage>> {
    let params =
        serde_json::from_value::<BTreeMap<String, String>>(params).map_err(|error| invalid_record(format!("invalid localized message parameters: {error}")))?;
    match key {
        Some(key) => Ok(Some(LocalizedMessage { key, params })),
        None if params.is_empty() => Ok(None),
        None => Err(invalid_record("localized message params exist without a key")),
    }
}

fn required_message(key: Option<String>, params: Value, field: &str) -> SchedulerResult<LocalizedMessage> {
    localized_message(key, params)?.ok_or_else(|| invalid_record(format!("{field} is missing")))
}

fn required_code<T: Copy>(value: Option<String>, parser: impl FnOnce(&str) -> Option<T>, field: &str) -> SchedulerResult<T> {
    let value = value.ok_or_else(|| invalid_record(format!("{field} is missing")))?;
    parse_code(&value, parser, field)
}

fn map_execution_detail(has_detail: bool, columns: (Option<String>, Option<i16>, Option<Value>)) -> SchedulerResult<Option<ExecutionDetail>> {
    match columns {
        (None, None, None) if !has_detail => Ok(None),
        (Some(kind), Some(schema_version), Some(Value::Object(payload))) if has_detail => {
            validate_execution_detail(&kind, schema_version)?;
            Ok(Some(ExecutionDetail::new(kind, schema_version, payload)))
        }
        _ => Err(invalid_record("sys_job_execution detail columns are inconsistent")),
    }
}

fn validate_execution_detail(kind: &str, schema_version: i16) -> SchedulerResult<()> {
    if !ExecutionDetail::kind_is_valid(kind) {
        return Err(invalid_record("execution detail kind is blank or too long"));
    }
    if schema_version <= 0 {
        return Err(invalid_record("execution detail schema version is not positive"));
    }
    Ok(())
}

fn parse_code<T: Copy>(value: &str, parser: impl FnOnce(&str) -> Option<T>, field: &str) -> SchedulerResult<T> {
    parser(value).ok_or_else(|| invalid_record(format!("invalid {field} code: {value}")))
}

pub fn map_sqlx_error(error: sqlx::Error) -> SchedulerError {
    match error {
        sqlx::Error::RowNotFound => SchedulerError::NotFound,
        other => SchedulerError::Infrastructure(other.to_string()),
    }
}

pub fn invalid_record(message: impl Into<String>) -> SchedulerError {
    SchedulerError::Infrastructure(message.into())
}
