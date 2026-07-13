#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobStatus {
    Normal,
    Paused,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConcurrentPolicy {
    Allow,
    Disallow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MisfirePolicy {
    FireOnce,
    DoNothing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistryStatus {
    Ok,
    Missing,
    RepeatableMismatch,
    InvalidParams,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeErrorCode {
    TaskMissing,
    RepeatableMismatch,
    InvalidParams,
    InvalidCron,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionState {
    Pending,
    Running,
    Terminal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionOutcome {
    Success,
    Failed,
    Skipped,
    Interrupted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerType {
    Scheduled,
    Misfire,
    Manual,
}

macro_rules! coded_enum {
    ($ty:ty, {$($variant:path => $code:literal),+ $(,)?}) => {
        impl $ty {
            pub const fn code(self) -> &'static str {
                match self { $($variant => $code),+ }
            }

            pub fn parse(value: &str) -> Option<Self> {
                match value { $($code => Some($variant)),+, _ => None }
            }
        }
    };
}

coded_enum!(JobStatus, { JobStatus::Normal => "0", JobStatus::Paused => "1" });
coded_enum!(ConcurrentPolicy, { ConcurrentPolicy::Allow => "0", ConcurrentPolicy::Disallow => "1" });
coded_enum!(MisfirePolicy, { MisfirePolicy::FireOnce => "2", MisfirePolicy::DoNothing => "3" });
coded_enum!(ExecutionState, { ExecutionState::Pending => "P", ExecutionState::Running => "R", ExecutionState::Terminal => "T" });
coded_enum!(ExecutionOutcome, {
    ExecutionOutcome::Success => "0",
    ExecutionOutcome::Failed => "1",
    ExecutionOutcome::Skipped => "2",
    ExecutionOutcome::Interrupted => "3",
});
coded_enum!(TriggerType, { TriggerType::Scheduled => "S", TriggerType::Misfire => "F", TriggerType::Manual => "M" });
coded_enum!(RuntimeErrorCode, {
    RuntimeErrorCode::TaskMissing => "task_missing",
    RuntimeErrorCode::RepeatableMismatch => "repeatable_mismatch",
    RuntimeErrorCode::InvalidParams => "invalid_params",
    RuntimeErrorCode::InvalidCron => "invalid_cron",
});

impl RegistryStatus {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Missing => "missing",
            Self::RepeatableMismatch => "repeatable_mismatch",
            Self::InvalidParams => "invalid_params",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimeErrorCode;

    #[test]
    fn runtime_error_codes_round_trip() {
        let cases = [
            (RuntimeErrorCode::TaskMissing, "task_missing"),
            (RuntimeErrorCode::RepeatableMismatch, "repeatable_mismatch"),
            (RuntimeErrorCode::InvalidParams, "invalid_params"),
            (RuntimeErrorCode::InvalidCron, "invalid_cron"),
        ];
        for (value, code) in cases {
            assert_eq!(value.code(), code);
            assert_eq!(RuntimeErrorCode::parse(code), Some(value));
        }
        assert_eq!(RuntimeErrorCode::parse("unexpected"), None);
    }
}
