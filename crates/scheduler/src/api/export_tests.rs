use chrono::{DateTime, Utc};
use kernel::{
    excel::{StreamingXlsxWriter, read_xlsx},
    pagination::PageSliceRequest,
};
use serde_json::json;
use types::http::Locale;

use crate::{
    application::{ExecutionLogSummary, SchedulerError},
    domain::{ExecutionOutcome, LocalizedMessage, TriggerType},
};

use super::{ExportPage, JOB_HEADER_KEYS, LOG_HEADER_KEYS, log_row, translated_headers};

const AUTHORIZATION_MARKER: &str = "Authorization: Bearer execution-detail-secret";
const COOKIE_MARKER: &str = "Cookie: session=execution-detail-secret";
const RESPONSE_BODY_MARKER: &str = "execution-detail-response-body";

#[test]
fn export_page_advances_with_checked_offsets() {
    let mut page = ExportPage::new(2).unwrap();
    page.observe_total(3).unwrap();

    assert_eq!(
        page.request().unwrap(),
        PageSliceRequest {
            offset: 0,
            limit: 2,
            page: 1,
            page_size: 2,
        }
    );
    assert!(page.advance(2).unwrap());
    assert_eq!(page.request().unwrap().offset, 2);
    assert_eq!(page.request().unwrap().page, 2);
    assert!(!page.advance(1).unwrap());
}

#[test]
fn export_page_rejects_invalid_or_inconsistent_progress() {
    assert_infrastructure(ExportPage::new(0).unwrap_err(), "export page size is zero after validation");

    let mut page = ExportPage::new(2).unwrap();
    page.observe_total(1).unwrap();
    assert_infrastructure(page.ensure_complete().unwrap_err(), "export page was empty before all reported rows were read");
    assert_infrastructure(page.observe_total(2).unwrap_err(), "export total changed while paging");

    let mut page = ExportPage::new(2).unwrap();
    page.observe_total(1).unwrap();
    assert_infrastructure(page.advance(2).unwrap_err(), "export page returned more rows than the reported total");

    let mut page = ExportPage {
        page: 1,
        page_size: 1,
        offset: u64::MAX,
        total: Some(u64::MAX),
    };
    assert_infrastructure(page.advance(1).unwrap_err(), "export offset overflow");
}

#[test]
#[cfg_attr(miri, ignore = "Miri isolation blocks rust_xlsxwriter SystemTime usage")]
fn job_log_export_is_summary_only() {
    assert_eq!(LOG_HEADER_KEYS.len(), 9);
    assert!(LOG_HEADER_KEYS.iter().all(|key| !key.contains("detail") && !key.contains("params")));
    let row = log_row(execution_summary(), Locale::En);
    assert_eq!(row.len(), LOG_HEADER_KEYS.len());
    assert_eq!(row[0], "job");
    assert_eq!(row[3], "Misfire recovery");
    assert_eq!(row[4], "Skipped");

    let sensitive_detail = json!({"request": {"headers": [AUTHORIZATION_MARKER, COOKIE_MARKER]}, "response": {"body": RESPONSE_BODY_MARKER}});
    let headers = translated_headers(Locale::En, LOG_HEADER_KEYS);
    let mut writer = StreamingXlsxWriter::new("logs", &headers).unwrap();
    writer.append_rows(&[row]).unwrap();
    let cells = read_xlsx(&writer.finish().unwrap()).unwrap();
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
