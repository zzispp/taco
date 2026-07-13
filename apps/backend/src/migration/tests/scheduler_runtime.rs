use std::sync::Arc;

use axum::{http::StatusCode, response::IntoResponse};
use scheduler::{
    api::SchedulerApiError,
    application::{
        ClaimExecutionRequest, ExecutionLease, FinishExecutionRequest, ImportJobCommand, LeaderLease, SchedulerError, SchedulerRuntimeStore, SchedulerService,
        SchedulerServiceParts, SchedulerUseCase, UpdateJobStatusCommand,
        task::{ScheduledTaskMetadata, StaticTaskCatalog},
        tasks::{HttpRequestTask, RefreshConfigCacheTask, RefreshDictCacheTask},
    },
    domain::{ConcurrentPolicy, ExecutionDetail, ExecutionOutcome, ExecutionState, JobStatus, LocalizedMessage, MisfirePolicy},
    infra::{PostgresExecutionLease, PostgresLeaderLease, StorageSchedulerRepository},
};
use serde_json::json;
use sqlx::{PgPool, query_scalar};
use storage::Database;

use super::{TestDatabase, fresh};

const EXECUTION_ID: &str = "execution-lease-test";
const SECOND_EXECUTION_ID: &str = "execution-lease-test-two";
const EXECUTOR_EPOCH: &str = "executor-test-epoch";
const SCHEDULER_EXECUTION_ACTIVE: &str = "scheduler_execution_active";
const LARGE_DETAIL_BODY_LENGTH: usize = 32_768;

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn postgres_leases_enforce_single_ownership_and_allow_takeover() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();

    assert_leader_release_takeover(database.pool()).await;
    assert_leader_session_loss_takeover(database.pool()).await;
    assert_execution_lease_ownership(database.pool()).await;

    database.drop().await;
}

async fn assert_leader_release_takeover(pool: &PgPool) {
    let first = PostgresLeaderLease::new(pool.clone());
    let second = PostgresLeaderLease::new(pool.clone());
    let mut leader = first.try_acquire().await.unwrap().unwrap();

    assert!(leader.is_alive().await.unwrap());
    assert!(second.try_acquire().await.unwrap().is_none());
    leader.release().await.unwrap();

    let mut successor = second.try_acquire().await.unwrap().unwrap();
    assert!(successor.is_alive().await.unwrap());
    successor.release().await.unwrap();
}

async fn assert_leader_session_loss_takeover(pool: &PgPool) {
    let first = PostgresLeaderLease::new(pool.clone());
    let second = PostgresLeaderLease::new(pool.clone());
    let mut leader = first.try_acquire().await.unwrap().unwrap();

    terminate_advisory_lock_holder(pool).await;
    assert!(matches!(leader.is_alive().await, Err(SchedulerError::Infrastructure(_))));
    drop(leader);

    let mut successor = second.try_acquire().await.unwrap().unwrap();
    assert!(successor.is_alive().await.unwrap());
    successor.release().await.unwrap();
}

async fn terminate_advisory_lock_holder(pool: &PgPool) {
    let pid = query_scalar::<_, i32>(
        "SELECT pid FROM pg_locks WHERE locktype='advisory' AND database=(SELECT oid FROM pg_database WHERE datname=current_database()) AND granted",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let terminated = query_scalar::<_, bool>("SELECT pg_terminate_backend($1)")
        .bind(pid)
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(terminated);
}

async fn assert_execution_lease_ownership(pool: &PgPool) {
    let lease = PostgresExecutionLease::new(pool.clone());
    let mut session = lease.open_session().await.unwrap();

    assert!(session.try_acquire(EXECUTION_ID).await.unwrap());
    assert!(lease.is_owned(EXECUTION_ID).await.unwrap());
    session.release(EXECUTION_ID).await.unwrap();
    assert!(!lease.is_owned(EXECUTION_ID).await.unwrap());

    assert!(session.try_acquire(EXECUTION_ID).await.unwrap());
    assert!(session.try_acquire(SECOND_EXECUTION_ID).await.unwrap());
    session.release_all().await.unwrap();
    assert!(!lease.is_owned(EXECUTION_ID).await.unwrap());
    assert!(!lease.is_owned(SECOND_EXECUTION_ID).await.unwrap());
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn scheduler_repository_serializes_manual_runs_and_preserves_running_snapshots() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();
    let harness = SchedulerHarness::new(database.pool());
    let job_id = harness.import_job().await;

    let pending_id = assert_one_manual_run_is_accepted(&harness.service, &job_id).await;
    harness.pause_and_assert_pending_cancelled(&job_id, &pending_id).await;
    harness.run_delete_and_finish_snapshot(&job_id).await;

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
            catalog,
            clock: repository.clone(),
        }));
        Self { service, repository }
    }

    pub(super) async fn import_job(&self) -> String {
        self.service.import_job(test_job_command()).await.unwrap().job.id
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
        assert_eq!(detail.task_params, test_job_command().task_params);
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
        assert_eq!(stored.task_params, test_job_command().task_params);
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

fn test_job_command() -> ImportJobCommand {
    ImportJobCommand {
        task_key: "httpClient.request".into(),
        name: "manual concurrency test".into(),
        group: "SYSTEM".into(),
        cron_expression: "0 * * * * *".into(),
        misfire_policy: MisfirePolicy::DoNothing,
        concurrent: ConcurrentPolicy::Allow,
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
