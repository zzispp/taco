use crate::{
    application::{
        SchedulerError, SchedulerResult,
        cron::validate_cron,
        task::{ScheduledTaskDefinition, TaskLifecycleCapabilities},
        validation::require_text,
    },
    domain::{ConcurrentPolicy, Job, JobStatus, MisfirePolicy, RegistryStatus},
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
    require_replacement_policy_allowed(definition, &command)?;
    let invoke_target = (definition.params.render_invoke_target)(definition.task_key, &command.task_params)?;
    Ok(PersistJobReplacement {
        input: command,
        params_schema_version: definition.params.schema_version,
        invoke_target,
    })
}

pub(super) fn lifecycle_capabilities(catalog: &dyn TaskCatalog, job: &Job) -> TaskLifecycleCapabilities {
    catalog
        .get(&job.task_key)
        .map(|definition| definition.lifecycle.capabilities())
        .unwrap_or(TaskLifecycleCapabilities::ADMINISTRABLE)
}

pub(super) fn registry_status(catalog: &dyn TaskCatalog, job: &Job) -> RegistryStatus {
    let Some(definition) = catalog.get(&job.task_key) else {
        return RegistryStatus::Missing;
    };
    if definition.repeatable != job.repeatable {
        return RegistryStatus::RepeatableMismatch;
    }
    if definition.params.schema_version != job.params_schema_version || (definition.params.validate_persisted)(&job.task_params).is_err() {
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
    (definition.params.validate_persisted)(&job.task_params)?;
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

pub(super) fn require_status_change_allowed(catalog: &dyn TaskCatalog, job: &Job, status: JobStatus) -> SchedulerResult<()> {
    if status == JobStatus::Paused && catalog.get(&job.task_key).is_some_and(|definition| !definition.lifecycle.can_disable()) {
        return Err(SchedulerError::Forbidden(super::error::localized(
            "errors.scheduler.required_task_cannot_disable",
        )));
    }
    Ok(())
}

pub(super) fn require_deletion_allowed(catalog: &dyn TaskCatalog, job: &Job) -> SchedulerResult<()> {
    if catalog.get(&job.task_key).is_some_and(|definition| !definition.lifecycle.can_delete()) {
        return Err(SchedulerError::Forbidden(super::error::localized(
            "errors.scheduler.required_task_cannot_delete",
        )));
    }
    Ok(())
}

fn validate_common(input: JobValidation<'_>) -> SchedulerResult<()> {
    require_text(input.name, "errors.scheduler.job_name_required")?;
    require_text(input.group, "errors.scheduler.job_group_required")?;
    require_text(input.cron, "errors.scheduler.cron_required")?;
    validate_cron(input.cron)?;
    validate_json_object(input.params)
}

fn require_replacement_policy_allowed(definition: ScheduledTaskDefinition, command: &ReplaceJobCommand) -> SchedulerResult<()> {
    if definition.lifecycle.can_edit_execution_policy()
        || (command.misfire_policy == MisfirePolicy::FireOnce && command.concurrent == ConcurrentPolicy::Disallow)
    {
        return Ok(());
    }
    Err(SchedulerError::Forbidden(super::error::localized(
        "errors.scheduler.required_task_policy_locked",
    )))
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use serde_json::json;

    use crate::{
        application::task::{ScheduledTaskMetadata, StaticTaskCatalog, TaskLifecyclePolicy},
        domain::{ConcurrentPolicy, Job, JobStatus, MisfirePolicy},
    };

    use super::{SchedulerError, replacement, require_deletion_allowed, require_status_change_allowed};
    use crate::application::{ReplaceJobCommand, tasks::SystemLogCleanupTask};

    #[test]
    fn required_task_cannot_be_paused_or_deleted() {
        let mut definition = crate::application::tasks::RefreshConfigCacheTask::descriptor();
        definition.lifecycle = TaskLifecyclePolicy::RequiredEnabled;
        let catalog = StaticTaskCatalog::try_new([definition]).unwrap();
        let job = job(definition.task_key);

        for result in [
            require_status_change_allowed(catalog.as_ref(), &job, JobStatus::Paused),
            require_deletion_allowed(catalog.as_ref(), &job),
        ] {
            let SchedulerError::Forbidden(error) = result.unwrap_err() else {
                panic!("required task action must be forbidden");
            };
            assert!(matches!(
                error.key(),
                "errors.scheduler.required_task_cannot_disable" | "errors.scheduler.required_task_cannot_delete"
            ));
        }
    }

    #[test]
    fn administrable_task_keeps_existing_status_and_delete_behavior() {
        let definition = crate::application::tasks::RefreshConfigCacheTask::descriptor();
        let catalog = StaticTaskCatalog::try_new([definition]).unwrap();
        let job = job(definition.task_key);

        assert!(require_status_change_allowed(catalog.as_ref(), &job, JobStatus::Paused).is_ok());
        assert!(require_deletion_allowed(catalog.as_ref(), &job).is_ok());
    }

    #[test]
    fn required_pausable_task_can_pause_and_change_policy_but_cannot_be_deleted() {
        let mut definition = SystemLogCleanupTask::descriptor();
        definition.lifecycle = TaskLifecyclePolicy::RequiredPausable;
        let catalog = StaticTaskCatalog::try_new([definition]).unwrap();
        let job = job(definition.task_key);

        assert!(require_status_change_allowed(catalog.as_ref(), &job, JobStatus::Paused).is_ok());
        assert!(replacement(replacement_command(MisfirePolicy::DoNothing, ConcurrentPolicy::Allow), definition).is_ok());
        let SchedulerError::Forbidden(error) = require_deletion_allowed(catalog.as_ref(), &job).unwrap_err() else {
            panic!("required pausable task deletion must be forbidden");
        };
        assert_eq!(error.key(), "errors.scheduler.required_task_cannot_delete");
    }

    #[test]
    fn required_task_replacement_rejects_unsafe_execution_policies() {
        let definition = SystemLogCleanupTask::descriptor();
        let unsafe_policies = [
            (MisfirePolicy::DoNothing, ConcurrentPolicy::Disallow),
            (MisfirePolicy::FireOnce, ConcurrentPolicy::Allow),
        ];

        for (misfire_policy, concurrent) in unsafe_policies {
            let error = replacement(replacement_command(misfire_policy, concurrent), definition).unwrap_err();
            let SchedulerError::Forbidden(error) = error else {
                panic!("required task policy replacement must be forbidden");
            };
            assert_eq!(error.key(), "errors.scheduler.required_task_policy_locked");
        }

        assert!(replacement(replacement_command(MisfirePolicy::FireOnce, ConcurrentPolicy::Disallow), definition,).is_ok());
    }

    fn job(task_key: &str) -> Job {
        Job {
            id: "job-1".into(),
            name: "job".into(),
            group: "SYSTEM".into(),
            task_key: task_key.into(),
            task_params: json!({}),
            params_schema_version: 1,
            repeatable: false,
            invoke_target: task_key.into(),
            cron_expression: "0 0 * * * *".into(),
            misfire_policy: MisfirePolicy::FireOnce,
            concurrent: ConcurrentPolicy::Disallow,
            status: JobStatus::Normal,
            schedule_revision: 1,
            next_run_at: None,
            runtime_error: None,
            create_by: "tester".into(),
            create_time: Utc::now(),
            update_by: "tester".into(),
            update_time: None,
            remark: None,
        }
    }

    fn replacement_command(misfire_policy: MisfirePolicy, concurrent: ConcurrentPolicy) -> ReplaceJobCommand {
        ReplaceJobCommand {
            id: "job-1".into(),
            name: "job".into(),
            group: "SYSTEM".into(),
            cron_expression: "0 0 * * * *".into(),
            misfire_policy,
            concurrent,
            task_params: json!({ "retention_days": 7, "batch_size": 1000 }),
            remark: None,
            operator: "tester".into(),
        }
    }
}
