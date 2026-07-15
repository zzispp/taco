use chrono::{TimeZone, Utc};
use kernel::pagination::{CursorDirection, CursorPageRequest};
use serde_json::json;

use crate::{
    application::{SchedulerCursorSlice, SchedulerError, job_cursor_page, job_cursor_query},
    domain::{ConcurrentPolicy, Job, JobListFilter, JobStatus, MisfirePolicy},
};

#[test]
fn job_cursor_round_trip_binds_filter_limit_sort_and_snapshot() {
    let filter = JobListFilter::default();
    let first_request = CursorPageRequest { limit: 1, cursor: None };
    let first_query = job_cursor_query(&filter, &first_request).unwrap();
    let point = crate::application::job_point(&job("job-1"));
    let page = job_cursor_page(
        &filter,
        &first_query,
        SchedulerCursorSlice {
            items: vec![job("job-1")],
            snapshot: Some(point.clone()),
            has_next: true,
            has_previous: false,
        },
    )
    .unwrap();
    let next = CursorPageRequest {
        limit: 1,
        cursor: page.next_cursor,
    };

    let decoded = job_cursor_query(&filter, &next).unwrap();

    assert_eq!(decoded.direction, CursorDirection::Next);
    assert_eq!(decoded.boundary, Some(point.clone()));
    assert_eq!(decoded.snapshot, Some(point));
}

#[test]
fn cursor_rejects_filter_limit_and_malformed_mismatches() {
    let filter = JobListFilter::default();
    let first_request = CursorPageRequest { limit: 1, cursor: None };
    let first_query = job_cursor_query(&filter, &first_request).unwrap();
    let point = crate::application::job_point(&job("job-1"));
    let cursor = job_cursor_page(
        &filter,
        &first_query,
        SchedulerCursorSlice {
            items: vec![job("job-1")],
            snapshot: Some(point),
            has_next: true,
            has_previous: false,
        },
    )
    .unwrap()
    .next_cursor
    .unwrap();
    let mut changed = filter.clone();
    changed.status = Some(JobStatus::Paused);

    for result in [
        job_cursor_query(
            &changed,
            &CursorPageRequest {
                limit: 1,
                cursor: Some(cursor.clone()),
            },
        ),
        job_cursor_query(
            &filter,
            &CursorPageRequest {
                limit: 2,
                cursor: Some(cursor),
            },
        ),
        job_cursor_query(
            &filter,
            &CursorPageRequest {
                limit: 1,
                cursor: Some("malformed".into()),
            },
        ),
    ] {
        assert!(matches!(result, Err(SchedulerError::InvalidCursor)));
    }
}

#[test]
fn empty_next_page_exposes_a_previous_recovery_cursor() {
    let filter = JobListFilter::default();
    let first_request = CursorPageRequest { limit: 1, cursor: None };
    let first_query = job_cursor_query(&filter, &first_request).unwrap();
    let point = crate::application::job_point(&job("job-1"));
    let first_page = job_cursor_page(
        &filter,
        &first_query,
        SchedulerCursorSlice {
            items: vec![job("job-1")],
            snapshot: Some(point),
            has_next: true,
            has_previous: false,
        },
    )
    .unwrap();
    let request = CursorPageRequest {
        limit: 1,
        cursor: first_page.next_cursor,
    };
    let query = job_cursor_query(&filter, &request).unwrap();
    let page = job_cursor_page(
        &filter,
        &query,
        SchedulerCursorSlice {
            items: Vec::new(),
            snapshot: query.snapshot.clone(),
            has_next: false,
            has_previous: false,
        },
    )
    .unwrap();

    assert!(page.items.is_empty());
    assert!(page.has_previous);
    assert!(page.previous_cursor.is_some());
    assert!(!page.has_next);
}

#[test]
fn empty_previous_page_exposes_a_next_recovery_cursor() {
    let filter = JobListFilter::default();
    let first_request = CursorPageRequest { limit: 1, cursor: None };
    let first_query = job_cursor_query(&filter, &first_request).unwrap();
    let page = job_cursor_page(
        &filter,
        &first_query,
        SchedulerCursorSlice {
            items: vec![job("job-1")],
            snapshot: Some(crate::application::job_point(&job("snapshot"))),
            has_next: false,
            has_previous: true,
        },
    )
    .unwrap();
    let request = CursorPageRequest {
        limit: 1,
        cursor: page.previous_cursor,
    };
    let query = job_cursor_query(&filter, &request).unwrap();
    let page = job_cursor_page(
        &filter,
        &query,
        SchedulerCursorSlice {
            items: Vec::new(),
            snapshot: query.snapshot.clone(),
            has_next: false,
            has_previous: false,
        },
    )
    .unwrap();

    assert!(page.items.is_empty());
    assert!(page.has_next);
    assert!(page.next_cursor.is_some());
    assert!(!page.has_previous);
}

#[test]
fn cursor_limit_is_validated_before_querying() {
    for limit in [0, 101] {
        assert!(matches!(
            job_cursor_query(&JobListFilter::default(), &CursorPageRequest { limit, cursor: None }),
            Err(SchedulerError::InvalidInput(_))
        ));
    }
}

fn job(id: &str) -> Job {
    Job {
        id: id.into(),
        name: "job".into(),
        group: "SYSTEM".into(),
        task_key: "task".into(),
        task_params: json!({}),
        params_schema_version: 1,
        repeatable: false,
        invoke_target: "task".into(),
        cron_expression: "0 * * * * *".into(),
        misfire_policy: MisfirePolicy::DoNothing,
        concurrent: ConcurrentPolicy::Disallow,
        status: JobStatus::Normal,
        schedule_revision: 1,
        next_run_at: None,
        runtime_error: None,
        create_by: "tester".into(),
        create_time: Utc.with_ymd_and_hms(2026, 7, 15, 1, 2, 3).unwrap(),
        update_by: "tester".into(),
        update_time: None,
        remark: None,
    }
}
