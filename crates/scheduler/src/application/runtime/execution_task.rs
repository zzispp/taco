use chrono::{DateTime, Utc};
use kernel::error::LocalizedError;

use crate::{
    application::{
        FinishExecutionRequest,
        task::{TaskExecutionFailure, TaskExecutionOutput, TaskInvocation},
    },
    domain::{Execution, ExecutionDetail, ExecutionOutcome, LocalizedMessage},
};

use super::SchedulerRuntimeParts;

pub(super) struct TaskCompletion {
    pub execution: Execution,
    pub result: Result<TaskExecutionOutput, TaskExecutionFailure>,
}

pub(super) struct PendingFinish {
    pub execution: Execution,
    pub outcome: ExecutionOutcome,
    pub message: LocalizedMessage,
    pub error: Option<LocalizedMessage>,
    pub detail: Option<ExecutionDetail>,
    terminalization: TerminalizationAttempt,
}

#[derive(Default)]
struct TerminalizationAttempt {
    ended_at: Option<DateTime<Utc>>,
}

impl TerminalizationAttempt {
    fn resolve_ended_at(&mut self, candidate: DateTime<Utc>) -> DateTime<Utc> {
        *self.ended_at.get_or_insert(candidate)
    }
}

impl PendingFinish {
    pub fn ended_at(&self) -> Option<DateTime<Utc>> {
        self.terminalization.ended_at
    }

    pub fn request(&mut self, ended_at: DateTime<Utc>) -> FinishExecutionRequest {
        let ended_at = self.terminalization.resolve_ended_at(ended_at);
        FinishExecutionRequest {
            execution_id: self.execution.id.clone(),
            outcome: self.outcome,
            message: self.message.clone(),
            error: self.error.clone(),
            detail: self.detail.clone(),
            ended_at,
        }
    }
}

pub(super) async fn execute_task(parts: &SchedulerRuntimeParts, execution: &Execution) -> Result<TaskExecutionOutput, TaskExecutionFailure> {
    let definition = parts.catalog.get(&execution.snapshot.task_key).ok_or_else(task_definition_changed)?;
    if definition.params.schema_version != execution.snapshot.params_schema_version {
        return Err(task_definition_changed());
    }
    (definition.params.validate)(&execution.snapshot.task_params).map_err(task_params_changed)?;
    let task = (definition.factory)();
    task.execute(
        parts.task_context.clone(),
        TaskInvocation {
            execution_id: execution.id.clone(),
            job_id: execution.snapshot.job_id.clone(),
            task_key: execution.snapshot.task_key.clone(),
            task_params: execution.snapshot.task_params.clone(),
            invoke_target: execution.snapshot.invoke_target.clone(),
        },
    )
    .await
}

pub(super) fn pending_finish(completion: TaskCompletion) -> PendingFinish {
    match completion.result {
        Ok(output) => successful_finish(completion.execution, output),
        Err(error) => failed_finish(completion.execution, error),
    }
}

pub(super) fn panicked_finish(execution: Execution, error: tokio::task::JoinError) -> PendingFinish {
    taco_tracing::error_with_fields!("scheduled task panicked", &error, execution_id = execution.id.as_str());
    PendingFinish {
        execution,
        outcome: ExecutionOutcome::Failed,
        message: LocalizedMessage::new("scheduler.execution.failed"),
        error: Some(LocalizedMessage::new("scheduler.execution.task_panicked")),
        detail: None,
        terminalization: TerminalizationAttempt::default(),
    }
}

fn successful_finish(execution: Execution, output: TaskExecutionOutput) -> PendingFinish {
    PendingFinish {
        execution,
        outcome: ExecutionOutcome::Success,
        message: LocalizedMessage::new("scheduler.execution.success"),
        error: None,
        detail: output.detail,
        terminalization: TerminalizationAttempt::default(),
    }
}

fn failed_finish(execution: Execution, error: TaskExecutionFailure) -> PendingFinish {
    taco_tracing::error_with_fields!(
        "scheduled task execution failed",
        &error,
        execution_id = execution.id.as_str(),
        job_id = execution.snapshot.job_id.as_str(),
        task_key = execution.snapshot.task_key.as_str()
    );
    let public_error = LocalizedMessage::from(&error.public);
    PendingFinish {
        execution,
        outcome: ExecutionOutcome::Failed,
        message: LocalizedMessage::new("scheduler.execution.failed"),
        error: Some(public_error),
        detail: error.detail.map(|detail| *detail),
        terminalization: TerminalizationAttempt::default(),
    }
}

fn task_definition_changed() -> TaskExecutionFailure {
    TaskExecutionFailure::new(
        LocalizedError::new("errors.scheduler.task_definition_changed"),
        "pending execution task definition is unavailable or changed",
    )
}

fn task_params_changed(error: impl std::fmt::Display) -> TaskExecutionFailure {
    TaskExecutionFailure::new(
        LocalizedError::new("errors.scheduler.task_definition_changed"),
        format!("pending execution parameters no longer match task definition: {error}"),
    )
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use kernel::error::LocalizedError;
    use serde::Serialize;
    use serde_json::json;

    use crate::{
        application::task::{TaskExecutionDetailPayload, TaskExecutionFailure, TaskExecutionOutput},
        domain::{ConcurrentPolicy, Execution, ExecutionSnapshot, ExecutionState, TriggerType},
    };

    use super::{failed_finish, panicked_finish, successful_finish, task_definition_changed, task_params_changed};

    #[test]
    fn terminalization_retry_keeps_timestamp_and_exact_detail() {
        let first = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).expect("test timestamp must be valid");
        let retry = DateTime::<Utc>::from_timestamp(1_700_000_001, 0).expect("test timestamp must be valid");
        let output = TaskExecutionOutput::with_detail(TestDetail { marker: "complete".into() });
        let expected_detail = output.detail.clone();
        let mut pending = successful_finish(execution(), output);

        let first_request = pending.request(first);
        let retry_request = pending.request(retry);

        assert_eq!(first_request.ended_at, first);
        assert_eq!(retry_request.ended_at, first);
        assert_eq!(first_request.detail, expected_detail);
        assert_eq!(retry_request.detail, expected_detail);
    }

    #[test]
    fn failed_task_detail_is_forwarded_without_rewriting() {
        let failure = TaskExecutionFailure::new(LocalizedError::new("errors.scheduler.task_http_request_failed"), "stable diagnostic")
            .with_detail(TestDetail { marker: "failed".into() });
        let expected_detail = failure.detail.as_deref().cloned();

        let pending = failed_finish(execution(), failure);

        assert_eq!(pending.detail, expected_detail);
    }

    #[tokio::test]
    async fn panic_does_not_fabricate_execution_detail() {
        let join_error = tokio::spawn(async { panic!("scheduled task test panic") }).await.unwrap_err();

        let pending = panicked_finish(execution(), join_error);

        assert_eq!(pending.detail, None);
    }

    #[test]
    fn definition_and_validation_failures_do_not_fabricate_detail() {
        assert_eq!(task_definition_changed().detail, None);
        assert_eq!(task_params_changed("invalid params").detail, None);
    }

    #[derive(Serialize)]
    struct TestDetail {
        marker: String,
    }

    impl TaskExecutionDetailPayload for TestDetail {
        const KIND: &'static str = "test";
        const SCHEMA_VERSION: i16 = 1;
    }

    fn execution() -> Execution {
        let now = DateTime::<Utc>::from_timestamp(1_700_000_000, 0).expect("test timestamp must be valid");
        Execution {
            id: "execution-id".into(),
            snapshot: ExecutionSnapshot {
                job_id: "job-id".into(),
                job_revision: 7,
                job_name: "job".into(),
                job_group: "SYSTEM".into(),
                task_key: "task".into(),
                task_params: json!({}),
                params_schema_version: 1,
                repeatable: true,
                invoke_target: "task()".into(),
                concurrent: ConcurrentPolicy::Allow,
            },
            trigger: TriggerType::Manual,
            scheduled_at: now,
            state: ExecutionState::Running,
            outcome: None,
            executor_epoch: Some("executor".into()),
            requested_by: Some("tester".into()),
            message: None,
            error: None,
            start_time: Some(now),
            end_time: None,
            create_time: now,
        }
    }
}
