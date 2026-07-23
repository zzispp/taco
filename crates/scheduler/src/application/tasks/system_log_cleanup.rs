use async_trait::async_trait;
use scheduler_macros::scheduled_task;
use serde::{Deserialize, Serialize};

use crate::application::task::{ScheduledTask, TaskExecutionContext, TaskExecutionDetailPayload, TaskExecutionFailure, TaskExecutionOutput, TaskInvocation};
use crate::{
    application::{SchedulerError, SchedulerResult},
    domain::{Execution, ExecutionDetail, ExecutionOutcome, ExecutionState},
};

use super::system_log_cleanup_params::{
    SYSTEM_LOG_CLEANUP_JOB_ID, SYSTEM_LOG_CLEANUP_TASK_KEY, SystemLogCleanupParams, is_manual_system_log_cleanup, manual_cleanup_filter,
    parse_system_log_cleanup_params,
};

pub const SYSTEM_LOG_CLEANUP_DETAIL_KIND: &str = "system_log_cleanup";
pub const SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION: i16 = 1;

#[scheduled_task(
    task_key = SYSTEM_LOG_CLEANUP_TASK_KEY,
    name_key = "scheduler.tasks.observability.system_log_cleanup.name",
    group = "SYSTEM",
    group_key = "scheduler.task_groups.system",
    description_key = "scheduler.tasks.observability.system_log_cleanup.description",
    repeatable = false,
    lifecycle = scheduler::application::task::TaskLifecyclePolicy::RequiredEnabled,
    params = SystemLogCleanupParams,
)]
#[derive(Default)]
pub struct SystemLogCleanupTask;

#[async_trait]
impl ScheduledTask for SystemLogCleanupTask {
    async fn execute(&self, context: TaskExecutionContext, invocation: TaskInvocation) -> Result<TaskExecutionOutput, TaskExecutionFailure> {
        let result = match parse_system_log_cleanup_params(&invocation.task_params).map_err(task_params_failure)? {
            SystemLogCleanupParams::Retention(params) => context.system_log_cleanup.cleanup_expired(params.retention_days, params.batch_size).await?,
            SystemLogCleanupParams::Manual(params) => {
                context
                    .system_log_cleanup
                    .cleanup_filtered(manual_cleanup_filter(params.filter).map_err(task_params_failure)?, params.batch_size)
                    .await?
            }
        };
        Ok(TaskExecutionOutput::with_detail(SystemLogCleanupReport {
            deleted: result.deleted,
            batches: result.batches,
        }))
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SystemLogCleanupReport {
    pub deleted: u64,
    pub batches: u64,
}

impl SystemLogCleanupReport {
    pub const fn new(deleted: u64, batches: u64) -> Self {
        Self { deleted, batches }
    }

    pub fn from_execution_detail(detail: &ExecutionDetail) -> SchedulerResult<Self> {
        if detail.kind() != SYSTEM_LOG_CLEANUP_DETAIL_KIND || detail.schema_version() != SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION {
            return Err(invalid_cleanup_execution());
        }
        serde_json::from_value(detail.payload().clone()).map_err(|_| invalid_cleanup_execution())
    }
}

impl TaskExecutionDetailPayload for SystemLogCleanupReport {
    const KIND: &'static str = SYSTEM_LOG_CLEANUP_DETAIL_KIND;
    const SCHEMA_VERSION: i16 = SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManualSystemLogCleanupExecutionState {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
    Interrupted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManualSystemLogCleanupExecution {
    pub execution_id: String,
    pub state: ManualSystemLogCleanupExecutionState,
    pub report: Option<SystemLogCleanupReport>,
}

pub fn manual_system_log_cleanup_execution(execution: &Execution, detail: Option<&ExecutionDetail>) -> SchedulerResult<ManualSystemLogCleanupExecution> {
    ensure_manual_cleanup_execution(execution)?;
    let state = execution_state(execution)?;
    let report = detail.map(SystemLogCleanupReport::from_execution_detail).transpose()?;
    if state == ManualSystemLogCleanupExecutionState::Succeeded && report.is_none() {
        return Err(invalid_cleanup_execution());
    }
    Ok(ManualSystemLogCleanupExecution {
        execution_id: execution.id.clone(),
        state,
        report,
    })
}

fn ensure_manual_cleanup_execution(execution: &Execution) -> SchedulerResult<()> {
    if execution.snapshot.job_id != SYSTEM_LOG_CLEANUP_JOB_ID
        || execution.snapshot.task_key != SYSTEM_LOG_CLEANUP_TASK_KEY
        || !is_manual_system_log_cleanup(&execution.snapshot.task_params)
    {
        return Err(SchedulerError::NotFound);
    }
    Ok(())
}

fn execution_state(execution: &Execution) -> SchedulerResult<ManualSystemLogCleanupExecutionState> {
    match execution.state {
        ExecutionState::Pending => Ok(ManualSystemLogCleanupExecutionState::Pending),
        ExecutionState::Running => Ok(ManualSystemLogCleanupExecutionState::Running),
        ExecutionState::Terminal => match execution.outcome.ok_or_else(invalid_cleanup_execution)? {
            ExecutionOutcome::Success => Ok(ManualSystemLogCleanupExecutionState::Succeeded),
            ExecutionOutcome::Failed => Ok(ManualSystemLogCleanupExecutionState::Failed),
            ExecutionOutcome::Skipped => Ok(ManualSystemLogCleanupExecutionState::Skipped),
            ExecutionOutcome::Interrupted => Ok(ManualSystemLogCleanupExecutionState::Interrupted),
        },
    }
}

fn invalid_cleanup_execution() -> SchedulerError {
    SchedulerError::Infrastructure("scheduler stored an invalid system-log cleanup execution".into())
}

fn task_params_failure(error: crate::application::SchedulerError) -> TaskExecutionFailure {
    TaskExecutionFailure::new(kernel::error::LocalizedError::new("errors.scheduler.invalid_params"), error.to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use serde_json::{Map, json};

    use crate::application::task::{
        FileCleanupPort, FileTrashCleanupResult, FileUploadSessionCleanupResult, HttpTaskClient, OutboundHttpFailure, OutboundHttpRequest,
        OutboundHttpResponse, ScheduledTask, SystemCacheRefreshPort, SystemLogCleanupFilter, SystemLogCleanupLevel, SystemLogCleanupPort,
        SystemLogCleanupResult, TaskExecutionContext, TaskExecutionFailure, TaskInvocation,
    };

    use super::{
        SYSTEM_LOG_CLEANUP_DETAIL_KIND, SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION, SYSTEM_LOG_CLEANUP_TASK_KEY, SystemLogCleanupReport, SystemLogCleanupTask,
    };

    #[tokio::test]
    async fn manual_cleanup_task_reports_aggregated_deleted_rows_and_batches() {
        let output = SystemLogCleanupTask.execute(context(), manual_invocation()).await.unwrap();

        let detail = output.detail.unwrap();
        assert_eq!(detail.kind(), SYSTEM_LOG_CLEANUP_DETAIL_KIND);
        assert_eq!(detail.schema_version(), SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION);
        assert_eq!(detail.payload(), &json!({"deleted": 9, "batches": 3}));
    }

    #[test]
    fn cleanup_report_rejects_a_detail_with_an_unknown_schema_version() {
        let detail = crate::domain::ExecutionDetail::new(
            SYSTEM_LOG_CLEANUP_DETAIL_KIND,
            SYSTEM_LOG_CLEANUP_DETAIL_SCHEMA_VERSION + 1,
            Map::from_iter([("deleted".into(), json!(2)), ("batches".into(), json!(1))]),
        );

        assert!(SystemLogCleanupReport::from_execution_detail(&detail).is_err());
    }

    fn context() -> TaskExecutionContext {
        TaskExecutionContext {
            http_client: Arc::new(UnexpectedHttpClient),
            system_cache: Arc::new(UnexpectedSystemCache),
            system_log_cleanup: Arc::new(CompletedCleanup),
            file_cleanup: Arc::new(UnexpectedFileCleanup),
        }
    }

    fn manual_invocation() -> TaskInvocation {
        TaskInvocation {
            execution_id: "execution".into(),
            job_id: "job".into(),
            task_key: SYSTEM_LOG_CLEANUP_TASK_KEY.into(),
            task_params: json!({"filter": {"keyword": null, "levels": ["error"], "target": null, "begin_time": "2026-07-16T00:00:00Z", "end_time": "2026-07-17T00:00:00Z"}, "batch_size": 1000}),
            invoke_target: SYSTEM_LOG_CLEANUP_TASK_KEY.into(),
        }
    }

    struct UnexpectedHttpClient;
    struct UnexpectedSystemCache;
    struct CompletedCleanup;
    struct UnexpectedFileCleanup;

    #[async_trait]
    impl HttpTaskClient for UnexpectedHttpClient {
        async fn send(&self, _: OutboundHttpRequest) -> Result<OutboundHttpResponse, OutboundHttpFailure> {
            unreachable!()
        }
    }

    #[async_trait]
    impl SystemCacheRefreshPort for UnexpectedSystemCache {
        async fn refresh_config_cache(&self) -> Result<(), TaskExecutionFailure> {
            unreachable!()
        }

        async fn refresh_dict_cache(&self) -> Result<(), TaskExecutionFailure> {
            unreachable!()
        }
    }

    #[async_trait]
    impl SystemLogCleanupPort for CompletedCleanup {
        async fn cleanup_expired(&self, _: u64, _: u64) -> Result<SystemLogCleanupResult, TaskExecutionFailure> {
            unreachable!()
        }

        async fn cleanup_filtered(&self, filter: SystemLogCleanupFilter, batch_size: u64) -> Result<SystemLogCleanupResult, TaskExecutionFailure> {
            assert_eq!(filter.levels, vec![SystemLogCleanupLevel::Error]);
            assert_eq!(batch_size, 1_000);
            Ok(SystemLogCleanupResult { deleted: 9, batches: 3 })
        }
    }

    #[async_trait]
    impl FileCleanupPort for UnexpectedFileCleanup {
        async fn purge_trash(&self, _: u64, _: u64) -> Result<FileTrashCleanupResult, TaskExecutionFailure> {
            panic!("system log cleanup test invoked file trash cleanup")
        }

        async fn cleanup_upload_sessions(&self, _: u64) -> Result<FileUploadSessionCleanupResult, TaskExecutionFailure> {
            panic!("system log cleanup test invoked upload session cleanup")
        }
    }
}
