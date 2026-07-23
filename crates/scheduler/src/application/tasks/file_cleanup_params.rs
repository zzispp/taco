use scheduler_macros::ScheduledTaskParams;
use serde::{Deserialize, Serialize};

use crate::application::{SchedulerResult, task::invalid_task_params};

pub const PURGE_TRASH_TASK_KEY: &str = "file.purgeTrash";
pub const CLEANUP_UPLOAD_SESSIONS_TASK_KEY: &str = "file.cleanupUploadSessions";
const PARAMS_SCHEMA_VERSION: i16 = 1;
const DEFAULT_TRASH_RETENTION_DAYS: u64 = 30;
const DEFAULT_BATCH_SIZE: u64 = 1_000;
const MAX_BATCH_SIZE: u64 = 10_000;
const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, ScheduledTaskParams, Serialize)]
#[task_params(schema_version = PARAMS_SCHEMA_VERSION, validate_with = Self::validate_bounds, render_with = Self::render_target)]
pub(crate) struct PurgeTrashParams {
    #[param_field(required, widget = "number", label_key = "scheduler.param_fields.file_cleanup.retention_days", default = DEFAULT_TRASH_RETENTION_DAYS)]
    pub(crate) retention_days: u64,
    #[param_field(required, widget = "number", label_key = "scheduler.param_fields.file_cleanup.batch_size", default = DEFAULT_BATCH_SIZE)]
    pub(crate) batch_size: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, ScheduledTaskParams, Serialize)]
#[task_params(schema_version = PARAMS_SCHEMA_VERSION, validate_with = Self::validate_bounds, render_with = Self::render_target)]
pub(crate) struct CleanupUploadSessionsParams {
    #[param_field(required, widget = "number", label_key = "scheduler.param_fields.file_cleanup.batch_size", default = DEFAULT_BATCH_SIZE)]
    pub(crate) batch_size: u64,
}

impl PurgeTrashParams {
    fn validate_bounds(params: &Self) -> SchedulerResult<()> {
        validate_retention_days(params.retention_days)?;
        validate_batch_size(params.batch_size)
    }

    fn render_target(task_key: &str, params: &Self) -> SchedulerResult<String> {
        Ok(format!("{task_key}(retention_days={},batch_size={})", params.retention_days, params.batch_size))
    }
}

impl CleanupUploadSessionsParams {
    fn validate_bounds(params: &Self) -> SchedulerResult<()> {
        validate_batch_size(params.batch_size)
    }

    fn render_target(task_key: &str, params: &Self) -> SchedulerResult<String> {
        Ok(format!("{task_key}(batch_size={})", params.batch_size))
    }
}

fn validate_retention_days(retention_days: u64) -> SchedulerResult<()> {
    let representable = i64::try_from(retention_days).ok().and_then(|days| days.checked_mul(SECONDS_PER_DAY)).is_some();
    if retention_days == 0 || !representable {
        return Err(invalid_task_params());
    }
    Ok(())
}

fn validate_batch_size(batch_size: u64) -> SchedulerResult<()> {
    if !(1..=MAX_BATCH_SIZE).contains(&batch_size) {
        return Err(invalid_task_params());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::application::task::TaskParams;

    use super::{CleanupUploadSessionsParams, PurgeTrashParams};

    #[test]
    fn file_cleanup_defaults_match_seeded_job_parameters() {
        assert_eq!(PurgeTrashParams::default_params(), json!({"retention_days": 30, "batch_size": 1000}));
        assert_eq!(CleanupUploadSessionsParams::default_params(), json!({"batch_size": 1000}));
    }

    #[test]
    fn purge_trash_rejects_invalid_retention_and_batch_bounds() {
        for params in [
            json!({"retention_days": 0, "batch_size": 1000}),
            json!({"retention_days": 30, "batch_size": 0}),
            json!({"retention_days": 30, "batch_size": 10001}),
        ] {
            assert!(PurgeTrashParams::validate(&params).is_err());
        }
    }

    #[test]
    fn cleanup_upload_sessions_rejects_invalid_batch_bounds() {
        for params in [json!({"batch_size": 0}), json!({"batch_size": 10001})] {
            assert!(CleanupUploadSessionsParams::validate(&params).is_err());
        }
    }

    #[test]
    fn file_cleanup_invoke_targets_match_seeded_jobs() {
        assert_eq!(
            PurgeTrashParams::render_invoke_target("file.purgeTrash", &json!({"retention_days": 30, "batch_size": 1000})).unwrap(),
            "file.purgeTrash(retention_days=30,batch_size=1000)"
        );
        assert_eq!(
            CleanupUploadSessionsParams::render_invoke_target("file.cleanupUploadSessions", &json!({"batch_size": 1000})).unwrap(),
            "file.cleanupUploadSessions(batch_size=1000)"
        );
    }
}
