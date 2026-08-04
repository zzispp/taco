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
    SchedulerAuditedUseCase, SchedulerError, SchedulerUseCase, SystemLogCleanupAuditRequest,
    task::{SystemLogCleanupFilter, SystemLogCleanupLevel},
    tasks::{
        LEGACY_SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION, ManualSystemLogCleanupExecution, ManualSystemLogCleanupExecutionState,
        SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION, SYSTEM_LOG_CLEANUP_JOB_ID, SystemLogCleanupExecutionReport,
    },
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
            .run_system_log_cleanup_with_audit(SystemLogCleanupAuditRequest {
                job_id: SYSTEM_LOG_CLEANUP_JOB_ID.to_owned(),
                filter: scheduler_filter(request.filter)?,
                requested_by: request.requested_by,
                audit: request.audit,
            })
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
    let metrics = cleanup_execution_metrics(execution.report);
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
        detail_schema_version: metrics.detail_schema_version,
        rows_deleted: metrics.rows_deleted,
        dropped_partitions: metrics.dropped_partitions,
        legacy_total_deleted: metrics.legacy_total_deleted,
        batches: metrics.batches,
    }
}

#[derive(Default)]
struct CleanupExecutionMetrics {
    detail_schema_version: Option<i16>,
    rows_deleted: Option<u64>,
    dropped_partitions: Option<u64>,
    legacy_total_deleted: Option<u64>,
    batches: Option<u64>,
}

fn cleanup_execution_metrics(report: Option<SystemLogCleanupExecutionReport>) -> CleanupExecutionMetrics {
    match report {
        Some(SystemLogCleanupExecutionReport::Current(report)) => CleanupExecutionMetrics {
            detail_schema_version: Some(SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION),
            rows_deleted: Some(report.rows_deleted),
            dropped_partitions: Some(report.dropped_partitions),
            legacy_total_deleted: None,
            batches: Some(report.batches),
        },
        Some(SystemLogCleanupExecutionReport::Legacy(report)) => CleanupExecutionMetrics {
            detail_schema_version: Some(LEGACY_SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION),
            rows_deleted: None,
            dropped_partitions: None,
            legacy_total_deleted: Some(report.legacy_total_deleted),
            batches: Some(report.batches),
        },
        None => CleanupExecutionMetrics::default(),
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

#[cfg(test)]
mod tests {
    use scheduler::application::tasks::{
        LEGACY_SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION, LegacySystemLogCleanupReport, ManualSystemLogCleanupExecution, ManualSystemLogCleanupExecutionState,
        SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION, SystemLogCleanupExecutionReport, SystemLogCleanupReport,
    };

    use super::adapt_cleanup_execution;

    #[test]
    fn current_execution_exposes_row_and_partition_metrics_separately() {
        let execution = adapt_cleanup_execution(ManualSystemLogCleanupExecution {
            execution_id: "execution".into(),
            state: ManualSystemLogCleanupExecutionState::Succeeded,
            report: Some(SystemLogCleanupExecutionReport::Current(SystemLogCleanupReport::new(5, 2, 3))),
        });

        assert_eq!(execution.detail_schema_version, Some(SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION));
        assert_eq!(execution.rows_deleted, Some(5));
        assert_eq!(execution.dropped_partitions, Some(2));
        assert_eq!(execution.legacy_total_deleted, None);
        assert_eq!(execution.batches, Some(3));
    }

    #[test]
    fn legacy_execution_does_not_relabel_its_total_as_rows_or_partitions() {
        let execution = adapt_cleanup_execution(ManualSystemLogCleanupExecution {
            execution_id: "execution".into(),
            state: ManualSystemLogCleanupExecutionState::Succeeded,
            report: Some(SystemLogCleanupExecutionReport::Legacy(LegacySystemLogCleanupReport {
                legacy_total_deleted: 7,
                batches: 4,
            })),
        });

        assert_eq!(execution.detail_schema_version, Some(LEGACY_SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION));
        assert_eq!(execution.rows_deleted, None);
        assert_eq!(execution.dropped_partitions, None);
        assert_eq!(execution.legacy_total_deleted, Some(7));
        assert_eq!(execution.batches, Some(4));
    }
}
