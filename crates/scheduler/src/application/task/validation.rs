use serde_json::Value;

use crate::application::{SchedulerError, SchedulerResult};

pub fn invalid_task_params() -> SchedulerError {
    SchedulerError::InvalidInput(kernel::error::LocalizedError::new("errors.scheduler.invalid_params"))
}

pub fn validate_param_object_keys(value: &Value, allowed: &[&str]) -> SchedulerResult<()> {
    let Some(object) = value.as_object() else {
        return Err(invalid_task_params());
    };
    if object.keys().all(|key| allowed.contains(&key.as_str())) {
        return Ok(());
    }
    Err(invalid_task_params())
}

pub fn validate_required_param_fields(value: &Value, required: &[&str]) -> SchedulerResult<()> {
    let Some(object) = value.as_object() else {
        return Err(invalid_task_params());
    };
    if required.iter().all(|key| object.get(*key).is_some_and(|item| !item.is_null())) {
        return Ok(());
    }
    Err(invalid_task_params())
}

pub fn validate_param_enum(value: &str, options: &[&str]) -> SchedulerResult<()> {
    if options.contains(&value) {
        return Ok(());
    }
    Err(invalid_task_params())
}

pub fn validate_param_pattern(value: &str, pattern: &str) -> SchedulerResult<()> {
    let regex = regex::Regex::new(pattern).map_err(|_| invalid_task_params())?;
    if regex.is_match(value) {
        return Ok(());
    }
    Err(invalid_task_params())
}
