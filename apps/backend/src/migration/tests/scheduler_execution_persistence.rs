use scheduler::{
    application::{ClaimExecutionRequest, FinishExecutionRequest, InterruptExecutionRequest, SchedulerRuntimeStore, SchedulerUseCase},
    domain::{ExecutionDetail, ExecutionOutcome, LocalizedMessage},
};
use serde_json::json;

use super::{TestDatabase, fresh, scheduler_runtime::SchedulerHarness};

const EXECUTOR_EPOCH: &str = "detail-persistence-epoch";

#[tokio::test]
async fn terminal_detail_is_atomic_for_failure_and_absent_for_interruption() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();
    let harness = SchedulerHarness::new(database.pool());
    let job_id = harness.import_job(scheduler::domain::ConcurrentPolicy::Allow).await;

    assert_failed_detail(&harness, &job_id).await;
    assert_interrupted_has_no_detail(&harness, &job_id).await;

    database.drop().await;
}

async fn assert_failed_detail(harness: &SchedulerHarness, job_id: &str) {
    let execution_id = claim_manual(harness, job_id).await;
    let detail = failure_detail();
    let request = FinishExecutionRequest {
        execution_id: execution_id.clone(),
        outcome: ExecutionOutcome::Failed,
        message: LocalizedMessage::new("scheduler.execution.failed"),
        error: Some(LocalizedMessage::new("errors.scheduler.task_http_request_failed")),
        detail: Some(detail.clone()),
        ended_at: harness.repository.database_now().await.unwrap(),
    };

    assert!(harness.repository.finish_execution(request.clone()).await.unwrap());
    assert!(harness.repository.finish_execution(request).await.unwrap());
    let stored = harness.service.get_job_log_detail(&execution_id).await.unwrap();
    assert_eq!(stored.summary.outcome, ExecutionOutcome::Failed);
    assert_eq!(stored.detail, Some(detail));
}

async fn assert_interrupted_has_no_detail(harness: &SchedulerHarness, job_id: &str) {
    let execution_id = claim_manual(harness, job_id).await;
    let request = InterruptExecutionRequest {
        execution_id: execution_id.clone(),
        ended_at: harness.repository.database_now().await.unwrap(),
    };

    assert!(harness.repository.interrupt_execution(request).await.unwrap());
    let stored = harness.service.get_job_log_detail(&execution_id).await.unwrap();
    assert_eq!(stored.summary.outcome, ExecutionOutcome::Interrupted);
    assert_eq!(stored.detail, None);
}

async fn claim_manual(harness: &SchedulerHarness, job_id: &str) -> String {
    let execution_id = harness.service.run_job(job_id, "detail-tester").await.unwrap();
    let request = ClaimExecutionRequest {
        execution_id: execution_id.clone(),
        executor_epoch: EXECUTOR_EPOCH.into(),
        started_at: harness.repository.database_now().await.unwrap(),
    };
    assert!(harness.repository.claim_execution(request).await.unwrap().is_some());
    execution_id
}

fn failure_detail() -> ExecutionDetail {
    let payload = json!({
        "duration_ms": 5,
        "request": {"method": "GET", "url": "http://localhost", "headers": [], "body": null},
        "response": null,
        "failure": {"code": "connect"}
    });
    ExecutionDetail::new("http_exchange", 1, payload.as_object().unwrap().clone())
}
