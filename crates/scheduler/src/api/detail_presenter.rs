use types::http::Locale;

use crate::{
    application::{
        ExecutionLogDetail, SchedulerResult,
        tasks::{HTTP_EXECUTION_DETAIL_KIND, sanitize_execution_task_params, sanitize_http_execution_payload},
    },
    domain::ExecutionDetail,
};

use super::{
    dto::{ExecutionDetailResponse, ExecutionLogDetailResponse},
    presenter::execution_response,
};

pub fn execution_detail_response(detail: ExecutionLogDetail, locale: Locale) -> SchedulerResult<ExecutionLogDetailResponse> {
    let task_key = detail.summary.task_key.clone();
    Ok(ExecutionLogDetailResponse {
        summary: execution_response(detail.summary, locale)?,
        job_revision: detail.job_revision,
        requested_by: detail.requested_by,
        task_params: sanitize_execution_task_params(&task_key, detail.task_params),
        detail: detail.detail.map(detail_response),
    })
}

fn detail_response(detail: ExecutionDetail) -> ExecutionDetailResponse {
    let (kind, schema_version, payload) = detail.into_parts();
    let payload = if kind == HTTP_EXECUTION_DETAIL_KIND {
        sanitize_http_execution_payload(payload)
    } else {
        payload
    };
    ExecutionDetailResponse { kind, schema_version, payload }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use serde_json::json;
    use types::http::Locale;

    use crate::{
        application::{ExecutionLogDetail, ExecutionLogSummary, tasks::HTTP_REQUEST_TASK_KEY},
        domain::{ExecutionDetail, ExecutionOutcome, LocalizedMessage, TriggerType},
    };

    use super::execution_detail_response;

    #[test]
    fn detail_response_flattens_summary_and_preserves_structured_payload() {
        let response = execution_detail_response(execution_detail(), Locale::En).unwrap();
        let value = serde_json::to_value(response).unwrap();

        assert_eq!(value["execution_id"], "execution-1");
        assert_eq!(value["job_revision"], 7);
        assert_eq!(value["requested_by"], "operator");
        assert_eq!(value["task_params"], json!({"url": "https://service.test"}));
        assert_eq!(value["detail"]["kind"], "http_exchange");
        assert_eq!(value["detail"]["schema_version"], 1);
        assert_eq!(value["detail"]["payload"]["failure"], json!({"code": "http_status"}));
    }

    #[test]
    fn legacy_detail_response_keeps_snapshot_with_null_detail() {
        let mut detail = execution_detail();
        detail.summary.has_detail = false;
        detail.requested_by = None;
        detail.detail = None;

        let value = serde_json::to_value(execution_detail_response(detail, Locale::En).unwrap()).unwrap();

        assert_eq!(value["has_detail"], false);
        assert_eq!(value["job_revision"], 7);
        assert_eq!(value["requested_by"], serde_json::Value::Null);
        assert_eq!(value["task_params"], json!({"url": "https://service.test"}));
        assert_eq!(value["detail"], serde_json::Value::Null);
    }

    #[test]
    fn http_execution_log_response_drops_task_parameters_and_historical_exchange_content() {
        let value = serde_json::to_value(execution_detail_response(sensitive_http_execution_detail(), Locale::En).unwrap()).unwrap();
        let rendered = value.to_string();

        assert_eq!(value["invoke_target"], "httpClient.request(...)");
        assert_eq!(value["task_params"], json!({"method": "POST", "url": "https://example.test/request"}));
        assert_eq!(value["detail"]["payload"]["request"]["headers"], json!([]));
        assert_eq!(value["detail"]["payload"]["request"]["body"], serde_json::Value::Null);
        assert_eq!(value["detail"]["payload"]["response"]["headers"], json!([]));
        assert_eq!(value["detail"]["payload"]["response"]["body"], serde_json::Value::Null);
        for marker in [
            "url-user",
            "url-password",
            "query-token",
            "request-header-token",
            "request-password",
            "request-captcha",
            "request-file-content",
            "response-token",
            "response-header-token",
            "response-file-content",
        ] {
            assert!(!rendered.contains(marker), "execution log response leaked {marker}");
        }
    }

    fn sensitive_http_execution_detail() -> ExecutionLogDetail {
        let mut detail = execution_detail();
        detail.summary.task_key = HTTP_REQUEST_TASK_KEY.into();
        detail.summary.invoke_target = "httpClient.request(POST, https://url-user:url-password@example.test?token=query-token)".into();
        detail.task_params = json!({
            "method": "POST",
            "url": "https://url-user:url-password@example.test/request?token=query-token",
            "headers": {"Authorization": "request-header-token"},
            "body": "request-password request-captcha request-file-content",
        });
        detail.detail = Some(ExecutionDetail::new(
            "http_exchange",
            1,
            json!({
                "duration_ms": 8,
                "request": {
                    "method": "POST",
                    "url": "https://url-user:url-password@example.test/request?token=query-token",
                    "headers": [{"name": "Authorization", "value": "request-header-token"}],
                    "body": "request-file-content",
                },
                "response": {
                    "status": 503,
                    "final_url": "https://url-user:url-password@example.test/response?token=response-token",
                    "headers": [{"name": "Set-Cookie", "value": "response-header-token"}],
                    "body": "response-file-content",
                },
                "failure": {"code": "http_status"},
            })
            .as_object()
            .unwrap()
            .clone(),
        ));
        detail
    }

    fn execution_detail() -> ExecutionLogDetail {
        ExecutionLogDetail {
            summary: execution_summary(),
            job_revision: 7,
            requested_by: Some("operator".into()),
            task_params: json!({"url": "https://service.test"}),
            detail: Some(ExecutionDetail::new(
                "http_exchange",
                1,
                json!({"duration_ms": 4, "request": {}, "response": null, "failure": {"code": "http_status"}})
                    .as_object()
                    .unwrap()
                    .clone(),
            )),
        }
    }

    fn execution_summary() -> ExecutionLogSummary {
        ExecutionLogSummary {
            id: "execution-1".into(),
            job_id: "job-1".into(),
            job_name: "job".into(),
            job_group: "system".into(),
            task_key: "task".into(),
            invoke_target: "task".into(),
            trigger: TriggerType::Manual,
            scheduled_at: fixed_time("2026-07-10T08:30:00Z"),
            outcome: ExecutionOutcome::Failed,
            message: LocalizedMessage::new("scheduler.execution.failed"),
            error: Some(LocalizedMessage::new("errors.scheduler.task_http_request_failed")),
            start_time: Some(fixed_time("2026-07-10T08:30:01Z")),
            end_time: fixed_time("2026-07-10T08:31:00Z"),
            create_time: fixed_time("2026-07-10T08:30:00Z"),
            has_detail: true,
        }
    }

    fn fixed_time(value: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(value).unwrap().with_timezone(&Utc)
    }
}
