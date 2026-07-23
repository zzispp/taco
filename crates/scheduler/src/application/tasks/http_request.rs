use async_trait::async_trait;
use kernel::error::LocalizedError;
use scheduler_macros::scheduled_task;

use crate::application::task::{
    HttpFailureCode, OutboundHttpFailure, OutboundHttpRequest, OutboundHttpResponse, ScheduledTask, TaskExecutionContext, TaskExecutionFailure,
    TaskExecutionOutput, TaskInvocation,
};

use super::{HttpRequestParams, http_report::HttpExecutionReport, http_report::HttpRequestReport, is_body_method};

pub const HTTP_REQUEST_TASK_KEY: &str = "httpClient.request";
const HTTP_SUCCESS_STATUS_START: u16 = 200;
const HTTP_SUCCESS_STATUS_END: u16 = 300;

#[scheduled_task(
    task_key = HTTP_REQUEST_TASK_KEY,
    name_key = "scheduler.tasks.http.request.name",
    group = "SYSTEM",
    group_key = "scheduler.task_groups.system",
    description_key = "scheduler.tasks.http.request.description",
    repeatable = true,
    lifecycle = scheduler::application::task::TaskLifecyclePolicy::Administrable,
    params = HttpRequestParams,
)]
#[derive(Default)]
pub struct HttpRequestTask;

#[async_trait]
impl ScheduledTask for HttpRequestTask {
    async fn execute(&self, context: TaskExecutionContext, invocation: TaskInvocation) -> Result<TaskExecutionOutput, TaskExecutionFailure> {
        let params: HttpRequestParams = invocation.decode_params()?;
        let request = OutboundHttpRequest {
            method: params.method.clone(),
            url: params.url,
            headers: params.headers.into_iter().collect(),
            body: is_body_method(&params.method).then_some(params.body).flatten(),
        };
        let request_report = HttpRequestReport::from(&request);
        match context.http_client.send(request).await {
            Ok(response) => response_result(request_report, response),
            Err(failure) => Err(request_failure(request_report, failure)),
        }
    }
}

fn response_result(request: HttpRequestReport, response: OutboundHttpResponse) -> Result<TaskExecutionOutput, TaskExecutionFailure> {
    let status = response.head.status;
    if (HTTP_SUCCESS_STATUS_START..HTTP_SUCCESS_STATUS_END).contains(&status) {
        let report = HttpExecutionReport::from_response(request, response, None);
        return Ok(TaskExecutionOutput::with_detail(report));
    }
    let report = HttpExecutionReport::from_response(request, response, Some(HttpFailureCode::HttpStatus));
    Err(TaskExecutionFailure::new(
        LocalizedError::new("errors.scheduler.task_http_status").with_param("status", status.to_string()),
        format!("scheduled HTTP task returned non-success status {status}"),
    )
    .with_detail(report))
}

fn request_failure(request: HttpRequestReport, failure: OutboundHttpFailure) -> TaskExecutionFailure {
    let code = failure.code;
    let report = HttpExecutionReport::from_failure(request, failure);
    TaskExecutionFailure::new(
        LocalizedError::new("errors.scheduler.task_http_request_failed"),
        format!("scheduled HTTP request failed with code {}", code.code()),
    )
    .with_detail(report)
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use async_trait::async_trait;
    use serde_json::{Value, json};

    use crate::application::task::{
        FileCleanupPort, FileTrashCleanupResult, FileUploadSessionCleanupResult, HttpTaskClient, OutboundHttpFailure, OutboundHttpHeader, OutboundHttpRequest,
        OutboundHttpResponse, OutboundHttpResponseHead, ScheduledTask, SystemCacheRefreshPort, SystemLogCleanupFilter, SystemLogCleanupPort,
        SystemLogCleanupResult, TaskExecutionContext, TaskExecutionFailure, TaskInvocation,
    };

    use super::{HttpFailureCode, HttpRequestTask};

    const ORIGINAL_URL: &str = "https://url-user:url-password@original.test/start?access_token=query-token#fragment";
    const SAFE_ORIGINAL_URL: &str = "https://original.test/start";
    const FINAL_URL: &str = "https://url-user:url-password@final.test/result?token=response-query-token#fragment";
    const SAFE_FINAL_URL: &str = "https://final.test/result";
    const REQUEST_HEADER_SECRET: &str = "request-header-token";
    const REQUEST_BODY_SECRET: &str = "request-password request-captcha request-file-content";
    const RESPONSE_HEADER_SECRET: &str = "response-header-token";
    const RESPONSE_BODY_SECRET: &str = "response-file-content";

    #[tokio::test]
    async fn success_report_persists_only_the_safe_http_exchange_summary() {
        let output = execute_with(Ok(success_response())).await.unwrap();
        let detail = output.detail.expect("HTTP success must include execution detail");

        assert_eq!(detail.kind(), "http_exchange");
        assert_eq!(detail.schema_version(), 1);
        assert_eq!(detail.payload(), &expected_success_payload());
        assert_no_secrets(detail.payload());
    }

    #[tokio::test]
    async fn non_success_status_keeps_status_without_response_content() {
        let mut response = success_response();
        response.head.status = 503;
        response.body = RESPONSE_BODY_SECRET.as_bytes().to_vec();

        let error = execute_with(Ok(response)).await.unwrap_err();
        assert_eq!(error.public.key(), "errors.scheduler.task_http_status");
        let detail = error.detail.expect("HTTP status failure must include execution detail");
        let payload = detail.payload();
        assert_eq!(payload["response"]["status"], json!(503));
        assert_eq!(payload["response"]["headers"], json!([]));
        assert_eq!(payload["response"]["body"], Value::Null);
        assert_eq!(payload["failure"], json!({"code": "http_status"}));
        assert_no_secrets(payload);
    }

    #[tokio::test]
    async fn transport_failure_keeps_safe_request_and_null_response() {
        let failure = OutboundHttpFailure {
            code: HttpFailureCode::Connect,
            duration: Duration::from_millis(9),
            response: None,
        };

        let error = execute_with(Err(failure)).await.unwrap_err();
        assert_eq!(error.public.key(), "errors.scheduler.task_http_request_failed");
        let detail = error.detail.expect("HTTP transport failure must include execution detail");
        let payload = detail.payload();
        assert_eq!(payload["duration_ms"], json!(9));
        assert_eq!(payload["request"]["url"], json!(SAFE_ORIGINAL_URL));
        assert_eq!(payload["request"]["headers"], json!([]));
        assert_eq!(payload["request"]["body"], Value::Null);
        assert_eq!(payload["response"], Value::Null);
        assert_eq!(payload["failure"], json!({"code": "connect"}));
        assert_no_secrets(payload);
    }

    #[tokio::test]
    async fn response_body_failure_keeps_safe_head_and_null_body() {
        let response = success_response().head;
        let failure = OutboundHttpFailure {
            code: HttpFailureCode::ResponseBody,
            duration: Duration::from_millis(12),
            response: Some(response),
        };

        let error = execute_with(Err(failure)).await.unwrap_err();
        let detail = error.detail.expect("HTTP body failure must include execution detail");
        let payload = detail.payload();
        assert_eq!(payload["response"]["status"], json!(204));
        assert_eq!(payload["response"]["final_url"], json!(SAFE_FINAL_URL));
        assert_eq!(payload["response"]["headers"], json!([]));
        assert_eq!(payload["response"]["body"], Value::Null);
        assert_eq!(payload["failure"], json!({"code": "response_body"}));
        assert_no_secrets(payload);
    }

    async fn execute_with(
        result: Result<OutboundHttpResponse, OutboundHttpFailure>,
    ) -> Result<crate::application::task::TaskExecutionOutput, TaskExecutionFailure> {
        HttpRequestTask
            .execute(
                TaskExecutionContext {
                    http_client: Arc::new(StubHttpClient { result }),
                    system_cache: Arc::new(UnexpectedCachePort),
                    system_log_cleanup: Arc::new(UnexpectedSystemLogCleanupPort),
                    file_cleanup: Arc::new(UnexpectedFileCleanupPort),
                },
                invocation(),
            )
            .await
    }

    fn invocation() -> TaskInvocation {
        TaskInvocation {
            execution_id: "execution-id".into(),
            job_id: "job-id".into(),
            task_key: "httpClient.request".into(),
            task_params: json!({
                "method": "POST",
                "url": ORIGINAL_URL,
                "headers": {"Authorization": REQUEST_HEADER_SECRET, "x-second": "two"},
                "body": REQUEST_BODY_SECRET
            }),
            invoke_target: "httpClient.request(POST, ...)".into(),
        }
    }

    fn success_response() -> OutboundHttpResponse {
        OutboundHttpResponse {
            head: OutboundHttpResponseHead {
                status: 204,
                headers: vec![
                    OutboundHttpHeader {
                        name: "Set-Cookie".into(),
                        value: RESPONSE_HEADER_SECRET.as_bytes().to_vec(),
                    },
                    OutboundHttpHeader {
                        name: "x-repeat".into(),
                        value: vec![255, 0],
                    },
                ],
                final_url: FINAL_URL.into(),
            },
            body: RESPONSE_BODY_SECRET.as_bytes().to_vec(),
            duration: Duration::from_millis(27),
        }
    }

    fn expected_success_payload() -> Value {
        json!({
            "duration_ms": 27,
            "request": {
                "method": "POST",
                "url": SAFE_ORIGINAL_URL,
                "headers": [],
                "body": null
            },
            "response": {
                "status": 204,
                "final_url": SAFE_FINAL_URL,
                "headers": [],
                "body": null
            },
            "failure": null
        })
    }

    fn assert_no_secrets(payload: &Value) {
        let rendered = payload.to_string();
        for marker in [
            "url-user",
            "url-password",
            "query-token",
            "response-query-token",
            REQUEST_HEADER_SECRET,
            REQUEST_BODY_SECRET,
            RESPONSE_HEADER_SECRET,
            RESPONSE_BODY_SECRET,
        ] {
            assert!(!rendered.contains(marker), "execution detail leaked {marker}");
        }
    }

    #[derive(Clone)]
    struct StubHttpClient {
        result: Result<OutboundHttpResponse, OutboundHttpFailure>,
    }

    #[async_trait]
    impl HttpTaskClient for StubHttpClient {
        async fn send(&self, _request: OutboundHttpRequest) -> Result<OutboundHttpResponse, OutboundHttpFailure> {
            self.result.clone()
        }
    }

    struct UnexpectedCachePort;
    struct UnexpectedSystemLogCleanupPort;
    struct UnexpectedFileCleanupPort;

    #[async_trait]
    impl SystemCacheRefreshPort for UnexpectedCachePort {
        async fn refresh_config_cache(&self) -> Result<(), TaskExecutionFailure> {
            panic!("HTTP task test invoked config cache refresh")
        }

        async fn refresh_dict_cache(&self) -> Result<(), TaskExecutionFailure> {
            panic!("HTTP task test invoked dictionary cache refresh")
        }
    }

    #[async_trait]
    impl SystemLogCleanupPort for UnexpectedSystemLogCleanupPort {
        async fn cleanup_expired(&self, _: u64, _: u64) -> Result<SystemLogCleanupResult, TaskExecutionFailure> {
            panic!("HTTP task test invoked system log retention")
        }

        async fn cleanup_filtered(&self, _: SystemLogCleanupFilter, _: u64) -> Result<SystemLogCleanupResult, TaskExecutionFailure> {
            panic!("HTTP task test invoked system log cleanup")
        }
    }

    #[async_trait]
    impl FileCleanupPort for UnexpectedFileCleanupPort {
        async fn purge_trash(&self, _: u64, _: u64) -> Result<FileTrashCleanupResult, TaskExecutionFailure> {
            panic!("HTTP task test invoked file trash cleanup")
        }

        async fn cleanup_upload_sessions(&self, _: u64) -> Result<FileUploadSessionCleanupResult, TaskExecutionFailure> {
            panic!("HTTP task test invoked upload session cleanup")
        }
    }
}
