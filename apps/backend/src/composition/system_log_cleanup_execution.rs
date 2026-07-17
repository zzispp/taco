use std::sync::Arc;

use async_trait::async_trait;
use observability::{
    application::{
        ManualSystemLogCleanupRequest, ObservabilityError, ObservabilityResult, SystemLogCleanupExecution, SystemLogCleanupExecutionPort,
        SystemLogCleanupExecutionState, localized,
    },
    domain::SystemLogFilter,
};
use scheduler::application::{
    SchedulerAuditedUseCase, SchedulerError, SchedulerUseCase,
    task::{SystemLogCleanupFilter, SystemLogCleanupLevel},
    tasks::{ManualSystemLogCleanupExecution, ManualSystemLogCleanupExecutionState, SYSTEM_LOG_CLEANUP_JOB_ID},
};

#[derive(Clone)]
pub(super) struct SchedulerSystemLogCleanupExecutionAdapter {
    scheduler: Arc<dyn SchedulerUseCase>,
    audited_scheduler: Arc<dyn SchedulerAuditedUseCase>,
}

impl SchedulerSystemLogCleanupExecutionAdapter {
    pub(super) fn new(scheduler: Arc<dyn SchedulerUseCase>, audited_scheduler: Arc<dyn SchedulerAuditedUseCase>) -> Self {
        Self { scheduler, audited_scheduler }
    }
}

#[async_trait]
impl SystemLogCleanupExecutionPort for SchedulerSystemLogCleanupExecutionAdapter {
    async fn enqueue_manual_cleanup(&self, request: ManualSystemLogCleanupRequest) -> ObservabilityResult<String> {
        self.audited_scheduler
            .run_system_log_cleanup_with_audit(
                SYSTEM_LOG_CLEANUP_JOB_ID,
                scheduler_filter(request.filter)?,
                &request.requested_by,
                request.audit,
            )
            .await
            .map_err(scheduler_error)
    }

    async fn cleanup_execution(&self, execution_id: &str) -> ObservabilityResult<SystemLogCleanupExecution> {
        self.scheduler
            .get_manual_system_log_cleanup_execution(execution_id)
            .await
            .map(adapt_cleanup_execution)
            .map_err(scheduler_error)
    }
}

fn adapt_cleanup_execution(execution: ManualSystemLogCleanupExecution) -> SystemLogCleanupExecution {
    SystemLogCleanupExecution {
        execution_id: execution.execution_id,
        state: match execution.state {
            ManualSystemLogCleanupExecutionState::Pending => SystemLogCleanupExecutionState::Pending,
            ManualSystemLogCleanupExecutionState::Running => SystemLogCleanupExecutionState::Running,
            ManualSystemLogCleanupExecutionState::Succeeded => SystemLogCleanupExecutionState::Succeeded,
            ManualSystemLogCleanupExecutionState::Failed => SystemLogCleanupExecutionState::Failed,
            ManualSystemLogCleanupExecutionState::Skipped => SystemLogCleanupExecutionState::Skipped,
            ManualSystemLogCleanupExecutionState::Interrupted => SystemLogCleanupExecutionState::Interrupted,
        },
        deleted: execution.report.as_ref().map(|report| report.deleted),
        batches: execution.report.map(|report| report.batches),
    }
}

fn scheduler_filter(filter: SystemLogFilter) -> ObservabilityResult<SystemLogCleanupFilter> {
    let begin_time = filter.begin_time.ok_or_else(missing_time_range)?;
    let end_time = filter.end_time.ok_or_else(missing_time_range)?;
    Ok(SystemLogCleanupFilter {
        keyword: filter.keyword,
        levels: filter.levels.into_iter().map(scheduler_level).collect(),
        target: filter.target,
        begin_time,
        end_time,
    })
}

fn scheduler_level(level: observability::domain::SystemLogLevel) -> SystemLogCleanupLevel {
    match level {
        observability::domain::SystemLogLevel::Trace => SystemLogCleanupLevel::Trace,
        observability::domain::SystemLogLevel::Debug => SystemLogCleanupLevel::Debug,
        observability::domain::SystemLogLevel::Info => SystemLogCleanupLevel::Info,
        observability::domain::SystemLogLevel::Warn => SystemLogCleanupLevel::Warn,
        observability::domain::SystemLogLevel::Error => SystemLogCleanupLevel::Error,
    }
}

fn scheduler_error(error: SchedulerError) -> ObservabilityError {
    match error {
        SchedulerError::NotFound => ObservabilityError::NotFound,
        SchedulerError::Conflict { code, details } => ObservabilityError::conflict(code, details),
        SchedulerError::InvalidInput(error) => ObservabilityError::InvalidInput(error),
        other => ObservabilityError::Infrastructure(format!("scheduler system-log cleanup execution failed: {other}")),
    }
}

fn missing_time_range() -> ObservabilityError {
    ObservabilityError::InvalidInput(localized("errors.observability.time_range_required"))
}
