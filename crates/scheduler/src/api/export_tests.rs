use chrono::{DateTime, Utc};
use kernel::excel::{StreamingXlsxWriter, read_xlsx};
use serde_json::json;
use types::http::Locale;

use crate::{
    application::{ExecutionLogSummary, SchedulerError},
    domain::{ExecutionOutcome, LocalizedMessage, TriggerType},
};

use super::{ExportCursor, JOB_HEADER_KEYS, LOG_HEADER_KEYS, log_row, translated_headers};

const AUTHORIZATION_MARKER: &str = "Authorization: Bearer execution-detail-secret";
const COOKIE_MARKER: &str = "Cookie: session=execution-detail-secret";
const RESPONSE_BODY_MARKER: &str = "execution-detail-response-body";

#[test]
fn export_cursor_advances_with_a_stable_snapshot_and_no_offset() {
    let mut cursor = ExportCursor::new(2).unwrap();
    let point = crate::application::SchedulerCursorPoint {
        created_at_nanos: "0".into(),
        id: "job-2".into(),
    };

    assert!(cursor.advance(point.clone(), Some(point.clone()), true).unwrap());
    let request = cursor.request();
    assert_eq!(request.limit, 2);
    assert_eq!(request.boundary, Some(point.clone()));
    assert_eq!(request.snapshot, Some(point));
}

#[test]
fn export_cursor_rejects_zero_limit_and_missing_snapshot() {
    assert_infrastructure(ExportCursor::new(0).unwrap_err(), "export page size is zero after validation");
    let mut cursor = ExportCursor::new(2).unwrap();
    let point = crate::application::SchedulerCursorPoint {
        created_at_nanos: "0".into(),
        id: "job-2".into(),
    };
    assert_infrastructure(cursor.advance(point, None, true).unwrap_err(), "scheduler export batch is missing its snapshot");
}

#[test]
fn job_log_export_is_summary_only() {
    assert_eq!(LOG_HEADER_KEYS.len(), 9);
    assert!(LOG_HEADER_KEYS.iter().all(|key| !key.contains("detail") && !key.contains("params")));
    let row = log_row(execution_summary(), Locale::En).unwrap();
    assert_eq!(row.len(), LOG_HEADER_KEYS.len());
    assert_eq!(row[0], "job");
    assert_eq!(row[3], "Misfire recovery");
    assert_eq!(row[4], "Skipped");

    let sensitive_detail = json!({"request": {"headers": [AUTHORIZATION_MARKER, COOKIE_MARKER]}, "response": {"body": RESPONSE_BODY_MARKER}});
    let headers = translated_headers(Locale::En, LOG_HEADER_KEYS);
    let mut writer = StreamingXlsxWriter::new("logs", &headers).unwrap();
    writer.append_rows(&[row]).unwrap();
    let artifact = writer.finish().unwrap();
    let cells = read_xlsx(&std::fs::read(artifact.path()).unwrap()).unwrap();
    let rendered = cells.into_iter().flatten().collect::<Vec<_>>().join("\n");

    for marker in [AUTHORIZATION_MARKER, COOKIE_MARKER, RESPONSE_BODY_MARKER] {
        assert!(sensitive_detail.to_string().contains(marker));
        assert!(!rendered.contains(marker));
    }
}

#[test]
fn job_export_columns_include_registry_and_runtime_state() {
    assert_eq!(JOB_HEADER_KEYS.len(), 10);
    assert_eq!(JOB_HEADER_KEYS[7], "excel.scheduler.job.headers.registry_status");
    assert_eq!(JOB_HEADER_KEYS[9], "excel.scheduler.job.headers.runtime_error");
}

fn execution_summary() -> ExecutionLogSummary {
    ExecutionLogSummary {
        id: "execution-1".into(),
        job_id: "job-1".into(),
        job_name: "job".into(),
        job_group: "system".into(),
        task_key: "task".into(),
        invoke_target: "task".into(),
        trigger: TriggerType::Misfire,
        scheduled_at: fixed_time("2026-07-10T08:30:00Z"),
        outcome: ExecutionOutcome::Skipped,
        message: LocalizedMessage::new("scheduler.execution.skipped_misfire"),
        error: None,
        start_time: None,
        end_time: fixed_time("2026-07-10T08:31:00Z"),
        create_time: fixed_time("2026-07-10T08:30:00Z"),
        has_detail: true,
    }
}

fn fixed_time(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value).unwrap().with_timezone(&Utc)
}

fn assert_infrastructure(error: SchedulerError, expected: &str) {
    match error {
        SchedulerError::Infrastructure(message) => assert_eq!(message, expected),
        other => panic!("expected infrastructure error, got {other:?}"),
    }
}
