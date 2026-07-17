use scheduler_macros::ScheduledTaskParams;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    application::{
        SchedulerResult,
        task::{SystemLogCleanupFilter, SystemLogCleanupLevel, TaskParams, invalid_task_params},
    },
    domain::TaskParamFormSpec,
};

pub const SYSTEM_LOG_CLEANUP_TASK_KEY: &str = "observability.cleanupSystemLogs";
pub const SYSTEM_LOG_CLEANUP_JOB_ID: &str = "system-log-cleanup";
const PARAMS_SCHEMA_VERSION: i16 = 1;
const DEFAULT_RETENTION_DAYS: u64 = 7;
const DEFAULT_BATCH_SIZE: u64 = 1_000;
const MAX_BATCH_SIZE: u64 = 10_000;
const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, ScheduledTaskParams, Serialize)]
#[task_params(schema_version = PARAMS_SCHEMA_VERSION, validate_with = Self::validate_bounds, render_with = Self::render_target)]
pub(crate) struct RetentionSystemLogCleanupParams {
    #[param_field(required, widget = "number", label_key = "scheduler.param_fields.system_log_cleanup.retention_days", default = DEFAULT_RETENTION_DAYS)]
    pub(crate) retention_days: u64,
    #[param_field(required, widget = "number", label_key = "scheduler.param_fields.system_log_cleanup.batch_size", default = DEFAULT_BATCH_SIZE)]
    pub(crate) batch_size: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManualSystemLogCleanupParams {
    pub(crate) filter: PersistedSystemLogCleanupFilter,
    pub(crate) batch_size: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PersistedSystemLogCleanupFilter {
    pub(crate) keyword: Option<String>,
    pub(crate) levels: Vec<String>,
    pub(crate) target: Option<String>,
    pub(crate) begin_time: String,
    pub(crate) end_time: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub(crate) enum SystemLogCleanupParams {
    Retention(RetentionSystemLogCleanupParams),
    Manual(ManualSystemLogCleanupParams),
}

impl TaskParams for SystemLogCleanupParams {
    const SCHEMA_VERSION: i16 = PARAMS_SCHEMA_VERSION;

    fn form() -> TaskParamFormSpec {
        RetentionSystemLogCleanupParams::form()
    }

    fn default_params() -> Value {
        RetentionSystemLogCleanupParams::default_params()
    }

    fn validate(value: &Value) -> SchedulerResult<()> {
        match parse(value)? {
            Self::Retention(_) => RetentionSystemLogCleanupParams::validate(value),
            Self::Manual(params) => validate_manual(&params),
        }
    }

    fn validate_persisted(value: &Value) -> SchedulerResult<()> {
        RetentionSystemLogCleanupParams::validate(value)
    }

    fn render_invoke_target(task_key: &str, value: &Value) -> SchedulerResult<String> {
        match parse(value)? {
            Self::Retention(_) => RetentionSystemLogCleanupParams::render_invoke_target(task_key, value),
            Self::Manual(params) => {
                validate_manual(&params)?;
                Ok(format!("{task_key}(manual_filter,batch_size={})", params.batch_size))
            }
        }
    }
}

pub(crate) fn parse_system_log_cleanup_params(value: &Value) -> SchedulerResult<SystemLogCleanupParams> {
    SystemLogCleanupParams::validate(value)?;
    parse(value)
}

pub fn manual_system_log_cleanup_params(retention_params: &Value, filter: SystemLogCleanupFilter) -> SchedulerResult<Value> {
    RetentionSystemLogCleanupParams::validate(retention_params)?;
    let retention = serde_json::from_value::<RetentionSystemLogCleanupParams>(retention_params.clone()).map_err(|_| invalid_task_params())?;
    serde_json::to_value(SystemLogCleanupParams::Manual(ManualSystemLogCleanupParams {
        filter: persisted_filter(filter)?,
        batch_size: retention.batch_size,
    }))
    .map_err(|_| invalid_task_params())
}

pub fn is_manual_system_log_cleanup(value: &Value) -> bool {
    matches!(parse_system_log_cleanup_params(value), Ok(SystemLogCleanupParams::Manual(_)))
}

pub(crate) fn manual_cleanup_filter(filter: PersistedSystemLogCleanupFilter) -> SchedulerResult<SystemLogCleanupFilter> {
    let levels = filter
        .levels
        .into_iter()
        .map(|level| SystemLogCleanupLevel::parse(&level).ok_or_else(invalid_task_params))
        .collect::<SchedulerResult<Vec<_>>>()?;
    let begin_time = parse_time(&filter.begin_time)?;
    let end_time = parse_time(&filter.end_time)?;
    if begin_time > end_time {
        return Err(invalid_task_params());
    }
    Ok(SystemLogCleanupFilter {
        keyword: filter.keyword,
        levels,
        target: filter.target,
        begin_time,
        end_time,
    })
}

impl RetentionSystemLogCleanupParams {
    fn validate_bounds(params: &Self) -> SchedulerResult<()> {
        if !retention_window_is_representable(params.retention_days) {
            return Err(invalid_task_params());
        }
        validate_batch_size(params.batch_size)
    }

    fn render_target(task_key: &str, params: &Self) -> SchedulerResult<String> {
        Ok(format!("{task_key}(retention_days={},batch_size={})", params.retention_days, params.batch_size))
    }
}

fn parse(value: &Value) -> SchedulerResult<SystemLogCleanupParams> {
    serde_json::from_value(value.clone()).map_err(|_| invalid_task_params())
}

fn validate_manual(params: &ManualSystemLogCleanupParams) -> SchedulerResult<()> {
    validate_batch_size(params.batch_size)?;
    manual_cleanup_filter(params.filter.clone()).map(|_| ())
}

fn persisted_filter(filter: SystemLogCleanupFilter) -> SchedulerResult<PersistedSystemLogCleanupFilter> {
    if filter.begin_time > filter.end_time {
        return Err(invalid_task_params());
    }
    Ok(PersistedSystemLogCleanupFilter {
        keyword: filter.keyword,
        levels: filter.levels.into_iter().map(SystemLogCleanupLevel::code).map(str::to_owned).collect(),
        target: filter.target,
        begin_time: filter.begin_time.format(&Rfc3339).map_err(|_| invalid_task_params())?,
        end_time: filter.end_time.format(&Rfc3339).map_err(|_| invalid_task_params())?,
    })
}

fn parse_time(value: &str) -> SchedulerResult<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| invalid_task_params())
}

fn validate_batch_size(batch_size: u64) -> SchedulerResult<()> {
    if !(1..=MAX_BATCH_SIZE).contains(&batch_size) {
        return Err(invalid_task_params());
    }
    Ok(())
}

fn retention_window_is_representable(retention_days: u64) -> bool {
    if retention_days == 0 {
        return false;
    }
    let Some(days) = i64::try_from(retention_days).ok() else {
        return false;
    };
    let Some(seconds) = days.checked_mul(SECONDS_PER_DAY) else {
        return false;
    };
    OffsetDateTime::now_utc().checked_sub(time::Duration::seconds(seconds)).is_some()
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use crate::application::task::{SystemLogCleanupFilter, SystemLogCleanupLevel, TaskParams};

    use super::{SystemLogCleanupParams, is_manual_system_log_cleanup, manual_system_log_cleanup_params};

    #[test]
    fn persisted_cleanup_job_rejects_manual_filter_parameters() {
        let value = manual_system_log_cleanup_params(&json!({"retention_days": 7, "batch_size": 1000}), filter()).unwrap();

        assert!(is_manual_system_log_cleanup(&value));
        assert!(SystemLogCleanupParams::validate(&value).is_ok());
        assert!(SystemLogCleanupParams::validate_persisted(&value).is_err());
    }

    #[test]
    fn manual_filter_requires_a_valid_time_range_and_level() {
        let value = json!({"filter": {"keyword": null, "levels": ["invalid"], "target": null, "begin_time": "2026-07-17T00:00:00Z", "end_time": "2026-07-16T00:00:00Z"}, "batch_size": 1000});

        assert!(SystemLogCleanupParams::validate(&value).is_err());
    }

    fn filter() -> SystemLogCleanupFilter {
        SystemLogCleanupFilter {
            keyword: Some("error".into()),
            levels: vec![SystemLogCleanupLevel::Error],
            target: Some("user".into()),
            begin_time: OffsetDateTime::parse("2026-07-16T00:00:00Z", &Rfc3339).unwrap(),
            end_time: OffsetDateTime::parse("2026-07-17T00:00:00Z", &Rfc3339).unwrap(),
        }
    }
}
