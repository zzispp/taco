use sqlx::{query, query_scalar};
use time::OffsetDateTime;

use super::{TestDatabase, fresh};

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn scheduler_schema_enforces_typed_time_and_journal_invariants() {
    let database = TestDatabase::create().await;
    let pool = database.pool();
    fresh(pool).await.unwrap();

    insert_valid_job(pool).await;
    assert_next_run_at_round_trip(pool).await;
    assert_job_constraints(pool).await;
    assert_execution_constraints(pool).await;

    database.drop().await;
}

async fn insert_valid_job(pool: &sqlx::PgPool) {
    query("INSERT INTO sys_job (job_id, job_name, job_group, task_key, invoke_target, cron_expression, next_run_at, create_time) VALUES ('job-1', 'job', 'SYSTEM', 'task.one', 'task.one', '0 * * * * *', TIMESTAMPTZ '2026-07-10 12:34:56+00', CURRENT_TIMESTAMP)")
        .execute(pool)
        .await
        .unwrap();
}

async fn assert_next_run_at_round_trip(pool: &sqlx::PgPool) {
    let value: OffsetDateTime = query_scalar("SELECT next_run_at FROM sys_job WHERE job_id = 'job-1'")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(value.unix_timestamp(), 1_783_686_896);
}

async fn assert_job_constraints(pool: &sqlx::PgPool) {
    let invalid_params = query("UPDATE sys_job SET task_params = '[]'::jsonb WHERE job_id = 'job-1'").execute(pool).await;
    let invalid_misfire = query("UPDATE sys_job SET misfire_policy = '1' WHERE job_id = 'job-1'").execute(pool).await;
    let split_runtime_error = query("UPDATE sys_job SET runtime_error_code = 'invalid_cron' WHERE job_id = 'job-1'")
        .execute(pool)
        .await;

    assert!(invalid_params.is_err());
    assert!(invalid_misfire.is_err());
    assert!(split_runtime_error.is_err());
}

async fn assert_execution_constraints(pool: &sqlx::PgPool) {
    insert_pending_execution(pool, PendingExecution::scheduled("execution-1", "1")).await.unwrap();
    let duplicate_occurrence = insert_pending_execution(pool, PendingExecution::misfire("execution-2", "0")).await;
    let duplicate_disallow = insert_pending_execution_at(pool, "execution-3", "2026-07-10 12:35:56+00").await;
    let invalid_terminal = query("INSERT INTO sys_job_execution (execution_id, job_id, job_revision, job_name, job_group, task_key, task_params, params_schema_version, repeatable, invoke_target, concurrent, trigger_type, scheduled_at, state, create_time) VALUES ('execution-4', 'job-2', 1, 'job', 'SYSTEM', 'task.two', '{}'::jsonb, 1, FALSE, 'task.two', '0', 'S', CURRENT_TIMESTAMP, 'T', CURRENT_TIMESTAMP)")
        .execute(pool)
        .await;
    let missing_terminal_message = query("INSERT INTO sys_job_execution (execution_id, job_id, job_revision, job_name, job_group, task_key, task_params, params_schema_version, repeatable, invoke_target, concurrent, trigger_type, scheduled_at, state, outcome, end_time, create_time) VALUES ('execution-5', 'job-2', 1, 'job', 'SYSTEM', 'task.two', '{}'::jsonb, 1, FALSE, 'task.two', '0', 'S', CURRENT_TIMESTAMP, 'T', '2', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)")
        .execute(pool)
        .await;

    assert!(duplicate_occurrence.is_err());
    assert!(duplicate_disallow.is_err());
    assert!(invalid_terminal.is_err());
    assert!(missing_terminal_message.is_err());
}

async fn insert_pending_execution(pool: &sqlx::PgPool, input: PendingExecution<'_>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    query("INSERT INTO sys_job_execution (execution_id, job_id, job_revision, job_name, job_group, task_key, task_params, params_schema_version, repeatable, invoke_target, concurrent, trigger_type, scheduled_at, state, create_time) VALUES ($1, 'job-1', 1, 'job', 'SYSTEM', 'task.one', '{}'::jsonb, 1, FALSE, 'task.one', $2, $3, TIMESTAMPTZ '2026-07-10 12:34:56+00', 'P', CURRENT_TIMESTAMP)")
        .bind(input.execution_id)
        .bind(input.concurrent)
        .bind(input.trigger_type)
        .execute(pool)
        .await
}

async fn insert_pending_execution_at(pool: &sqlx::PgPool, execution_id: &str, scheduled_at: &str) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    query("INSERT INTO sys_job_execution (execution_id, job_id, job_revision, job_name, job_group, task_key, task_params, params_schema_version, repeatable, invoke_target, concurrent, trigger_type, scheduled_at, state, create_time) VALUES ($1, 'job-1', 1, 'job', 'SYSTEM', 'task.one', '{}'::jsonb, 1, FALSE, 'task.one', $2, 'S', $3::timestamptz, 'P', CURRENT_TIMESTAMP)")
        .bind(execution_id)
        .bind("1")
        .bind(scheduled_at)
        .execute(pool)
        .await
}

struct PendingExecution<'a> {
    execution_id: &'a str,
    concurrent: &'static str,
    trigger_type: &'static str,
}

impl<'a> PendingExecution<'a> {
    const fn scheduled(execution_id: &'a str, concurrent: &'static str) -> Self {
        Self {
            execution_id,
            concurrent,
            trigger_type: "S",
        }
    }

    const fn misfire(execution_id: &'a str, concurrent: &'static str) -> Self {
        Self {
            execution_id,
            concurrent,
            trigger_type: "F",
        }
    }
}
