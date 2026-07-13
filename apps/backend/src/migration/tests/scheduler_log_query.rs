use kernel::pagination::{PageRequest, PageSliceRequest};
use scheduler::{
    application::SchedulerUseCase,
    domain::{ExecutionOutcome, JobLogListFilter, TriggerType},
};
use sqlx::{PgPool, query};

use super::{TestDatabase, fresh, scheduler_runtime::SchedulerHarness};

const PAGE_SIZE: u64 = 2;

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn job_log_query_combines_filters_and_keeps_export_pages_identical() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();
    insert_execution_fixtures(database.pool()).await;
    let harness = SchedulerHarness::new(database.pool());
    let filter = matching_filter();

    let first = harness.service.page_job_logs(filter.clone(), list_page(1)).await.unwrap();
    let second = harness.service.page_job_logs(filter.clone(), list_page(2)).await.unwrap();

    assert_eq!(first.total, 3);
    assert_eq!(execution_ids(&first.items), ["match-end", "match-middle"]);
    assert_eq!(second.total, 3);
    assert_eq!(execution_ids(&second.items), ["match-begin"]);

    let exported_first = harness.service.export_job_logs_page(filter.clone(), export_page(1, 0)).await.unwrap();
    let exported_second = harness.service.export_job_logs_page(filter, export_page(2, PAGE_SIZE)).await.unwrap();
    assert_eq!(exported_first, first);
    assert_eq!(exported_second, second);

    database.drop().await;
}

fn matching_filter() -> JobLogListFilter {
    JobLogListFilter {
        name: Some("billing".into()),
        group: Some("OPS".into()),
        outcome: Some(ExecutionOutcome::Failed),
        trigger: Some(TriggerType::Manual),
        begin_time: Some("2026-07-10T10:00:00Z".parse().unwrap()),
        end_time: Some("2026-07-10T10:30:00Z".parse().unwrap()),
    }
}

fn list_page(page: u64) -> PageRequest {
    PageRequest { page, page_size: PAGE_SIZE }
}

fn export_page(page: u64, offset: u64) -> PageSliceRequest {
    PageSliceRequest {
        offset,
        limit: PAGE_SIZE,
        page,
        page_size: PAGE_SIZE,
    }
}

fn execution_ids(items: &[scheduler::application::ExecutionLogSummary]) -> Vec<&str> {
    items.iter().map(|item| item.id.as_str()).collect()
}

async fn insert_execution_fixtures(pool: &PgPool) {
    let fixtures = [
        baseline("match-end", "2026-07-10T10:30:00Z"),
        baseline("match-middle", "2026-07-10T10:15:00Z"),
        baseline("match-begin", "2026-07-10T10:00:00Z"),
        ExecutionFixture {
            name: "Nightly cleanup",
            ..baseline("wrong-name", "2026-07-10T10:20:00Z")
        },
        ExecutionFixture {
            group: "FINANCE",
            ..baseline("wrong-group", "2026-07-10T10:19:00Z")
        },
        ExecutionFixture {
            outcome: ExecutionOutcome::Success.code(),
            ..baseline("wrong-outcome", "2026-07-10T10:18:00Z")
        },
        ExecutionFixture {
            trigger: TriggerType::Scheduled.code(),
            ..baseline("wrong-trigger", "2026-07-10T10:17:00Z")
        },
        baseline("before-range", "2026-07-10T09:59:59Z"),
        baseline("after-range", "2026-07-10T10:30:01Z"),
    ];
    for fixture in fixtures {
        insert_execution(pool, fixture).await;
    }
}

async fn insert_execution(pool: &PgPool, fixture: ExecutionFixture) {
    let requested_by = (fixture.trigger == TriggerType::Manual.code()).then_some("integration-tester");
    let (message_key, error_key) = if fixture.outcome == ExecutionOutcome::Failed.code() {
        ("scheduler.execution.failed", Some("errors.scheduler.task_http_request_failed"))
    } else {
        ("scheduler.execution.success", None)
    };
    query(
        "INSERT INTO sys_job_execution (execution_id, job_id, job_revision, job_name, job_group, task_key, task_params, \
         params_schema_version, repeatable, invoke_target, concurrent, trigger_type, scheduled_at, state, outcome, requested_by, \
         message_key, error_key, start_time, end_time, create_time) \
         VALUES ($1, $2, 1, $3, $4, 'httpClient.request', '{}'::jsonb, 1, TRUE, 'httpClient.request', '0', $5, $6::timestamptz, \
         'T', $7, $8, $9, $10, $6::timestamptz, $6::timestamptz, $6::timestamptz)",
    )
    .bind(fixture.id)
    .bind(format!("job-{}", fixture.id))
    .bind(fixture.name)
    .bind(fixture.group)
    .bind(fixture.trigger)
    .bind(fixture.create_time)
    .bind(fixture.outcome)
    .bind(requested_by)
    .bind(message_key)
    .bind(error_key)
    .execute(pool)
    .await
    .unwrap();
}

fn baseline(id: &'static str, create_time: &'static str) -> ExecutionFixture {
    ExecutionFixture {
        id,
        name: "Nightly BILLING",
        group: "OPS",
        outcome: ExecutionOutcome::Failed.code(),
        trigger: TriggerType::Manual.code(),
        create_time,
    }
}

struct ExecutionFixture {
    id: &'static str,
    name: &'static str,
    group: &'static str,
    outcome: &'static str,
    trigger: &'static str,
    create_time: &'static str,
}
