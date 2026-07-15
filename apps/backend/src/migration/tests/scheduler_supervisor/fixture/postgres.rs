use scheduler::{
    application::ImportJobCommand,
    domain::{ConcurrentPolicy, ExecutionOutcome, ExecutionState, JobStatus, MisfirePolicy, TriggerType},
};
use serde_json::json;
use sqlx::{PgPool, query_as, query_scalar};
use time::OffsetDateTime;
use tokio::time::sleep;

use super::{OccurrenceTarget, POLL_INTERVAL};

const ANNUAL_CRON: &str = "0 0 0 1 1 *";
const TEST_OPERATOR: &str = "scheduler-runtime-test";

pub(super) fn job_command(name: &str, url: &str) -> ImportJobCommand {
    ImportJobCommand {
        task_key: "httpClient.request".into(),
        name: format!("runtime-{name}"),
        group: "SYSTEM".into(),
        cron_expression: ANNUAL_CRON.into(),
        misfire_policy: MisfirePolicy::FireOnce,
        concurrent: ConcurrentPolicy::Allow,
        task_params: json!({"method": "GET", "url": url, "headers": {}}),
        remark: None,
        operator: TEST_OPERATOR.into(),
    }
}

pub(super) async fn schedule_without_notification(pool: &PgPool, job_id: &str, delay_ms: i64) -> OccurrenceTarget {
    let (scheduled_at, revision) = query_as::<_, (OffsetDateTime, i64)>(
        "UPDATE sys_job SET status=$1, schedule_revision=schedule_revision+1, \
         next_run_at=clock_timestamp()+($2 * INTERVAL '1 millisecond'), update_by=$3, update_time=clock_timestamp() \
         WHERE job_id=$4 RETURNING next_run_at, schedule_revision",
    )
    .bind(JobStatus::Normal.code())
    .bind(delay_ms)
    .bind(TEST_OPERATOR)
    .bind(job_id)
    .fetch_one(pool)
    .await
    .unwrap();
    OccurrenceTarget {
        job_id: job_id.into(),
        revision,
        scheduled_at,
    }
}

pub(super) async fn wait_for_success(pool: &PgPool, target: &OccurrenceTarget) {
    loop {
        let rows = occurrence_rows(pool, target).await;
        assert!(rows.len() <= 1, "duplicate scheduler occurrence rows: {rows:?}");
        if let Some((_, state, outcome)) = rows.first().filter(|row| row.1 == ExecutionState::Terminal.code()) {
            assert_eq!(state, ExecutionState::Terminal.code());
            assert_eq!(outcome.as_deref(), Some(ExecutionOutcome::Success.code()));
            return;
        }
        sleep(POLL_INTERVAL).await;
    }
}

pub(super) async fn occurrence_count(pool: &PgPool, target: &OccurrenceTarget) -> i64 {
    query_scalar(
        "SELECT COUNT(*) FROM sys_job_execution \
         WHERE job_id=$1 AND job_revision=$2 AND scheduled_at=$3 AND trigger_type IN ($4,$5)",
    )
    .bind(&target.job_id)
    .bind(target.revision)
    .bind(target.scheduled_at)
    .bind(TriggerType::Scheduled.code())
    .bind(TriggerType::Misfire.code())
    .fetch_one(pool)
    .await
    .unwrap()
}

pub(super) async fn advisory_lock_count(pool: &PgPool) -> i64 {
    i64::try_from(advisory_lock_pids(pool).await.len()).unwrap()
}

pub(super) async fn advisory_lock_pids(pool: &PgPool) -> Vec<i32> {
    query_scalar(
        "SELECT pid FROM pg_locks WHERE locktype='advisory' AND granted \
         AND database=(SELECT oid FROM pg_database WHERE datname=current_database()) ORDER BY pid",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

async fn occurrence_rows(pool: &PgPool, target: &OccurrenceTarget) -> Vec<(String, String, Option<String>)> {
    query_as(
        "SELECT execution_id, state, outcome FROM sys_job_execution \
         WHERE job_id=$1 AND job_revision=$2 AND scheduled_at=$3 AND trigger_type IN ($4,$5)",
    )
    .bind(&target.job_id)
    .bind(target.revision)
    .bind(target.scheduled_at)
    .bind(TriggerType::Scheduled.code())
    .bind(TriggerType::Misfire.code())
    .fetch_all(pool)
    .await
    .unwrap()
}
