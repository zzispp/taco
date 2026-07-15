use std::sync::Arc;

use axum::{http::StatusCode, response::IntoResponse};
use scheduler::{
    api::SchedulerApiError,
    application::{
        ClaimExecutionRequest, FinishExecutionRequest, ImportJobCommand, SchedulerError, SchedulerRuntimeStore, SchedulerService, SchedulerServiceParts,
        SchedulerUseCase, UpdateJobStatusCommand,
        task::{ScheduledTaskMetadata, StaticTaskCatalog},
        tasks::{HttpRequestTask, RefreshConfigCacheTask, RefreshDictCacheTask},
    },
    domain::{ConcurrentPolicy, ExecutionDetail, ExecutionOutcome, ExecutionState, JobStatus, LocalizedMessage, MisfirePolicy},
    infra::StorageSchedulerRepository,
};
use serde_json::json;
use sqlx::PgPool;
use storage::Database;

use super::{TestDatabase, fresh};

mod leases;

const EXECUTOR_EPOCH: &str = "executor-test-epoch";
const SCHEDULER_EXECUTION_ACTIVE: &str = "scheduler_execution_active";
const LARGE_DETAIL_BODY_LENGTH: usize = 32_768;

#[tokio::test]
async fn scheduler_repository_serializes_manual_runs_and_preserves_running_snapshots() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();
    let harness = SchedulerHarness::new(database.pool());
    let job_id = harness.import_job(ConcurrentPolicy::Disallow).await;

    let pending_id = assert_one_manual_run_is_accepted(&harness.service, &job_id).await;
    harness.pause_and_assert_pending_cancelled(&job_id, &pending_id).await;
    harness.run_delete_and_finish_snapshot(&job_id).await;

    database.drop().await;
}

#[tokio::test]
async fn scheduler_repository_allows_overlapping_manual_runs_when_policy_allows() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();
    let harness = SchedulerHarness::new(database.pool());
    let job_id = harness.import_job(ConcurrentPolicy::Allow).await;

    let execution_ids = assert_two_manual_runs_are_accepted(&harness.service, &job_id).await;

    assert_ne!(execution_ids[0], execution_ids[1]);
    database.drop().await;
}

pub(super) struct SchedulerHarness {
    pub(super) service: Arc<SchedulerService>,
    pub(super) repository: Arc<StorageSchedulerRepository>,
}

impl SchedulerHarness {
    pub(super) fn new(pool: &PgPool) -> Self {
        let repository = Arc::new(StorageSchedulerRepository::new(Database::new(pool.clone())));
        let catalog = StaticTaskCatalog::try_new([
            HttpRequestTask::descriptor(),
            RefreshConfigCacheTask::descriptor(),
            RefreshDictCacheTask::descriptor(),
        ])
        .unwrap();
        let service = Arc::new(SchedulerService::new(SchedulerServiceParts {
            query: repository.clone(),
            commands: repository.clone(),
            audited_commands: repository.clone(),
            catalog,
            clock: repository.clone(),
        }));
        Self { service, repository }
    }

    pub(super) async fn import_job(&self, concurrent: ConcurrentPolicy) -> String {
        self.service.import_job(test_job_command(concurrent)).await.unwrap().job.id
    }

    async fn pause_and_assert_pending_cancelled(&self, job_id: &str, execution_id: &str) {
        self.service
            .update_job_status(UpdateJobStatusCommand {
                id: job_id.to_owned(),
                status: JobStatus::Paused,
                operator: "pause-operator".into(),
            })
            .await
            .unwrap();

        let execution = self.service.get_job_log(execution_id).await.unwrap();
        assert_eq!(execution.outcome, ExecutionOutcome::Skipped);
        assert_eq!(execution.start_time, None);
        assert_eq!(execution.message.key, "scheduler.execution.cancelled_paused");
        assert!(!execution.has_detail);
        let detail = self.service.get_job_log_detail(execution_id).await.unwrap();
        assert_eq!(detail.detail, None);
        assert_eq!(detail.task_params, json!({}));
    }

    async fn run_delete_and_finish_snapshot(&self, job_id: &str) {
        let execution_id = self.service.run_job(job_id, "running-user").await.unwrap();
        let started_at = self.repository.database_now().await.unwrap();
        let claimed = self
            .repository
            .claim_execution(ClaimExecutionRequest {
                execution_id: execution_id.clone(),
                executor_epoch: EXECUTOR_EPOCH.into(),
                started_at,
            })
            .await
            .unwrap()
            .unwrap();
        assert_eq!(claimed.state, ExecutionState::Running);

        self.service.delete_job(job_id).await.unwrap();
        assert_running_execution(self.repository.as_ref(), &execution_id).await;
        self.finish_and_assert_terminal(&execution_id, job_id).await;
    }

    async fn finish_and_assert_terminal(&self, execution_id: &str, job_id: &str) {
        let detail = test_execution_detail();
        let request = FinishExecutionRequest {
            execution_id: execution_id.to_owned(),
            outcome: ExecutionOutcome::Success,
            message: LocalizedMessage::new("scheduler.execution.success"),
            error: None,
            detail: Some(detail.clone()),
            ended_at: self.repository.database_now().await.unwrap(),
        };
        assert!(self.repository.finish_execution(request.clone()).await.unwrap());
        assert!(self.repository.finish_execution(request.clone()).await.unwrap());
        assert!(!self.repository.finish_execution(changed_detail_request(request)).await.unwrap());

        let execution = self.service.get_job_log(execution_id).await.unwrap();
        assert_eq!(execution.outcome, ExecutionOutcome::Success);
        assert_eq!(execution.job_id, job_id);
        assert!(execution.has_detail);
        let stored = self.service.get_job_log_detail(execution_id).await.unwrap();
        assert_eq!(stored.detail, Some(detail));
        assert_eq!(stored.requested_by.as_deref(), Some("running-user"));
        assert_eq!(stored.task_params, json!({}));
        assert_eq!(self.repository.running_executions().await.unwrap().len(), 0);
    }
}

async fn assert_one_manual_run_is_accepted(service: &SchedulerService, job_id: &str) -> String {
    let (first, second) = tokio::join!(service.run_job(job_id, "manual-one"), service.run_job(job_id, "manual-two"));
    let mut accepted = Vec::new();
    let mut conflicts = 0;
    for result in [first, second] {
        match result {
            Ok(execution_id) => accepted.push(execution_id),
            Err(error @ SchedulerError::Conflict { .. }) => {
                assert_active_conflict(error);
                conflicts += 1;
            }
            Err(error) => panic!("unexpected manual run result: {error}"),
        }
    }
    assert_eq!(accepted.len(), 1);
    assert_eq!(conflicts, 1);
    accepted.pop().unwrap()
}

async fn assert_two_manual_runs_are_accepted(service: &SchedulerService, job_id: &str) -> [String; 2] {
    let (first, second) = tokio::join!(service.run_job(job_id, "manual-one"), service.run_job(job_id, "manual-two"));
    [first.unwrap(), second.unwrap()]
}

fn assert_active_conflict(error: SchedulerError) {
    match &error {
        SchedulerError::Conflict { code, details } => {
            assert_eq!(*code, SCHEDULER_EXECUTION_ACTIVE);
            assert_eq!(details.key(), "errors.scheduler.execution_active");
        }
        other => panic!("expected active execution conflict, got {other}"),
    }
    assert_eq!(SchedulerApiError(error).into_response().status(), StatusCode::CONFLICT);
}

async fn assert_running_execution(repository: &StorageSchedulerRepository, execution_id: &str) {
    let running = repository.running_executions().await.unwrap();
    assert_eq!(running.len(), 1);
    assert_eq!(running[0].id, execution_id);
    assert_eq!(running[0].state, ExecutionState::Running);
}

fn test_job_command(concurrent: ConcurrentPolicy) -> ImportJobCommand {
    ImportJobCommand {
        task_key: "httpClient.request".into(),
        name: "manual concurrency test".into(),
        group: "SYSTEM".into(),
        cron_expression: "0 * * * * *".into(),
        misfire_policy: MisfirePolicy::DoNothing,
        concurrent,
        task_params: json!({"method": "GET", "url": "https://example.test", "headers": {}}),
        remark: None,
        operator: "test-operator".into(),
    }
}

fn test_execution_detail() -> ExecutionDetail {
    let payload = json!({
        "duration_ms": 12,
        "request": {
            "method": "GET",
            "url": "https://example.test",
            "headers": [{
                "name": "Authorization",
                "value": {"encoding": "utf8", "content": "Bearer secret", "byte_length": 13}
            }],
            "body": null
        },
        "response": {
            "status": 200,
            "final_url": "https://example.test",
            "headers": [{
                "name": "x-nul",
                "value": {"encoding": "base64", "content": "YQBi", "byte_length": 3}
            }],
            "body": {"encoding": "utf8", "content": "x".repeat(LARGE_DETAIL_BODY_LENGTH), "byte_length": LARGE_DETAIL_BODY_LENGTH}
        },
        "failure": null
    });
    ExecutionDetail::new("http_exchange", 1, payload.as_object().unwrap().clone())
}

fn changed_detail_request(mut request: FinishExecutionRequest) -> FinishExecutionRequest {
    request.detail = Some(ExecutionDetail::new("http_exchange", 1, json!({"changed": true}).as_object().unwrap().clone()));
    request
}
