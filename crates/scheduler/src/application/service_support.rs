use crate::{
    application::{SchedulerError, SchedulerResult, cron::validate_cron, task::ScheduledTaskDefinition, validation::require_text},
    domain::{Job, RegistryStatus},
};

use super::{ImportJobCommand, PersistJobReplacement, PersistNewJob, ReplaceJobCommand, task::TaskCatalog, validation::validate_json_object};

struct JobValidation<'a> {
    name: &'a str,
    group: &'a str,
    cron: &'a str,
    params: &'a serde_json::Value,
}

pub(super) fn validate_import(command: &ImportJobCommand) -> SchedulerResult<()> {
    require_text(&command.task_key, "errors.scheduler.task_key_required")?;
    validate_common(JobValidation {
        name: &command.name,
        group: &command.group,
        cron: &command.cron_expression,
        params: &command.task_params,
    })
}

pub(super) fn validate_replace(command: &ReplaceJobCommand) -> SchedulerResult<()> {
    validate_common(JobValidation {
        name: &command.name,
        group: &command.group,
        cron: &command.cron_expression,
        params: &command.task_params,
    })
}

pub(super) fn new_job(command: ImportJobCommand, definition: ScheduledTaskDefinition) -> SchedulerResult<PersistNewJob> {
    let invoke_target = (definition.params.render_invoke_target)(definition.task_key, &command.task_params)?;
    Ok(PersistNewJob {
        input: command,
        params_schema_version: definition.params.schema_version,
        repeatable: definition.repeatable,
        invoke_target,
    })
}

pub(super) fn replacement(command: ReplaceJobCommand, definition: ScheduledTaskDefinition) -> SchedulerResult<PersistJobReplacement> {
    let invoke_target = (definition.params.render_invoke_target)(definition.task_key, &command.task_params)?;
    Ok(PersistJobReplacement {
        input: command,
        params_schema_version: definition.params.schema_version,
        invoke_target,
    })
}

pub(super) fn registry_status(catalog: &dyn TaskCatalog, job: &Job) -> RegistryStatus {
    let Some(definition) = catalog.get(&job.task_key) else {
        return RegistryStatus::Missing;
    };
    if definition.repeatable != job.repeatable {
        return RegistryStatus::RepeatableMismatch;
    }
    if definition.params.schema_version != job.params_schema_version || (definition.params.validate)(&job.task_params).is_err() {
        return RegistryStatus::InvalidParams;
    }
    RegistryStatus::Ok
}

pub(super) fn require_runnable(catalog: &dyn TaskCatalog, job: &Job) -> SchedulerResult<ScheduledTaskDefinition> {
    let definition = catalog
        .get(&job.task_key)
        .ok_or_else(|| SchedulerError::InvalidInput(super::error::localized("errors.scheduler.task_missing")))?;
    if definition.repeatable != job.repeatable {
        return Err(SchedulerError::InvalidInput(super::error::localized("errors.scheduler.repeatable_mismatch")));
    }
    if definition.params.schema_version != job.params_schema_version {
        return Err(SchedulerError::InvalidInput(super::error::localized("errors.scheduler.invalid_params")));
    }
    (definition.params.validate)(&job.task_params)?;
    validate_cron(&job.cron_expression)?;
    Ok(definition)
}

pub(super) fn require_editable(catalog: &dyn TaskCatalog, job: &Job) -> SchedulerResult<ScheduledTaskDefinition> {
    let definition = catalog
        .get(&job.task_key)
        .ok_or_else(|| SchedulerError::InvalidInput(super::error::localized("errors.scheduler.task_missing")))?;
    if definition.repeatable == job.repeatable {
        return Ok(definition);
    }
    Err(SchedulerError::InvalidInput(super::error::localized("errors.scheduler.repeatable_mismatch")))
}

fn validate_common(input: JobValidation<'_>) -> SchedulerResult<()> {
    require_text(input.name, "errors.scheduler.job_name_required")?;
    require_text(input.group, "errors.scheduler.job_group_required")?;
    require_text(input.cron, "errors.scheduler.cron_required")?;
    validate_cron(input.cron)?;
    validate_json_object(input.params)
}
