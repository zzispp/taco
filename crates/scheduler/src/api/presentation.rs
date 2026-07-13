use types::http::{Locale, translate_message};

use crate::domain::{ConcurrentPolicy, ExecutionOutcome, JobStatus, RegistryStatus, RuntimeErrorCode, TriggerType};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct EnumPresentation {
    pub wire_value: &'static str,
    translation_key: &'static str,
}

impl EnumPresentation {
    const fn new(wire_value: &'static str, translation_key: &'static str) -> Self {
        Self { wire_value, translation_key }
    }

    pub fn localized(self, locale: Locale) -> String {
        translate_message(locale, self.translation_key)
    }
}

pub(super) const fn job_status(value: JobStatus) -> EnumPresentation {
    match value {
        JobStatus::Normal => EnumPresentation::new(value.code(), "scheduler.status.normal"),
        JobStatus::Paused => EnumPresentation::new(value.code(), "scheduler.status.paused"),
    }
}

pub(super) const fn concurrent_policy(value: ConcurrentPolicy) -> EnumPresentation {
    match value {
        ConcurrentPolicy::Allow => EnumPresentation::new(value.code(), "scheduler.concurrent.allow"),
        ConcurrentPolicy::Disallow => EnumPresentation::new(value.code(), "scheduler.concurrent.disallow"),
    }
}

pub(super) const fn registry_status(value: RegistryStatus) -> EnumPresentation {
    match value {
        RegistryStatus::Ok => EnumPresentation::new(value.code(), "scheduler.registry_status.ok"),
        RegistryStatus::Missing => EnumPresentation::new(value.code(), "scheduler.registry_status.missing"),
        RegistryStatus::RepeatableMismatch => EnumPresentation::new(value.code(), "scheduler.registry_status.repeatable_mismatch"),
        RegistryStatus::InvalidParams => EnumPresentation::new(value.code(), "scheduler.registry_status.invalid_params"),
    }
}

pub(super) const fn trigger_type(value: TriggerType) -> EnumPresentation {
    match value {
        TriggerType::Scheduled => EnumPresentation::new("scheduled", "scheduler.trigger.scheduled"),
        TriggerType::Misfire => EnumPresentation::new("misfire", "scheduler.trigger.misfire"),
        TriggerType::Manual => EnumPresentation::new("manual", "scheduler.trigger.manual"),
    }
}

pub(super) const fn execution_outcome(value: ExecutionOutcome) -> EnumPresentation {
    match value {
        ExecutionOutcome::Success => EnumPresentation::new(value.code(), "scheduler.execution.status.success"),
        ExecutionOutcome::Failed => EnumPresentation::new(value.code(), "scheduler.execution.status.failed"),
        ExecutionOutcome::Skipped => EnumPresentation::new(value.code(), "scheduler.execution.status.skipped"),
        ExecutionOutcome::Interrupted => EnumPresentation::new(value.code(), "scheduler.execution.status.interrupted"),
    }
}

pub(super) const fn runtime_error(value: RuntimeErrorCode) -> EnumPresentation {
    match value {
        RuntimeErrorCode::TaskMissing => EnumPresentation::new(value.code(), "scheduler.runtime_error.task_missing"),
        RuntimeErrorCode::RepeatableMismatch => EnumPresentation::new(value.code(), "scheduler.runtime_error.repeatable_mismatch"),
        RuntimeErrorCode::InvalidParams => EnumPresentation::new(value.code(), "scheduler.runtime_error.invalid_params"),
        RuntimeErrorCode::InvalidCron => EnumPresentation::new(value.code(), "scheduler.runtime_error.invalid_cron"),
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::{ConcurrentPolicy, ExecutionOutcome, JobStatus, RegistryStatus, RuntimeErrorCode, TriggerType};

    use super::{concurrent_policy, execution_outcome, job_status, registry_status, runtime_error, trigger_type};

    #[test]
    fn enum_presentations_preserve_public_wire_values() {
        assert_eq!(job_status(JobStatus::Normal).wire_value, "0");
        assert_eq!(job_status(JobStatus::Paused).wire_value, "1");
        assert_eq!(concurrent_policy(ConcurrentPolicy::Allow).wire_value, "0");
        assert_eq!(concurrent_policy(ConcurrentPolicy::Disallow).wire_value, "1");
        assert_eq!(registry_status(RegistryStatus::RepeatableMismatch).wire_value, "repeatable_mismatch");
        assert_eq!(trigger_type(TriggerType::Manual).wire_value, "manual");
        assert_eq!(execution_outcome(ExecutionOutcome::Interrupted).wire_value, "3");
    }

    #[test]
    fn runtime_error_presentations_cover_every_domain_code() {
        let cases = [
            (RuntimeErrorCode::TaskMissing, "task_missing"),
            (RuntimeErrorCode::RepeatableMismatch, "repeatable_mismatch"),
            (RuntimeErrorCode::InvalidParams, "invalid_params"),
            (RuntimeErrorCode::InvalidCron, "invalid_cron"),
        ];

        for (code, expected) in cases {
            assert_eq!(runtime_error(code).wire_value, expected);
        }
    }
}
