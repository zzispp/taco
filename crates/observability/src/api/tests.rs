use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use audit_contract::{ActorSnapshot, AuditOutboxRecord, BusinessType, OperationAuditContext, OperationAuditSeed, OperationRequestSnapshot, OperatorType};
use axum::{
    body::{Body, to_bytes},
    extract::Extension,
    http::{Request, StatusCode, header},
    middleware,
};
use kernel::{
    excel::{StreamingXlsxWriter, TemporaryXlsxFile, read_xlsx},
    pagination::{CursorPage, CursorPageRequest},
    runtime_config::{ExportBatchConfig, ExportConfigProvider},
};
use tower::ServiceExt;

use crate::{
    api::{SystemLogApiState, SystemLogApiStateParts, create_router, export::system_log_export_layout},
    application::{
        ManualSystemLogCleanupRequest, ObservabilityError, ObservabilityResult, SystemLogCleanupExecution, SystemLogCleanupExecutionPort,
        SystemLogExportRequest, SystemLogExportUseCase, SystemLogRetentionReport, SystemLogUseCase,
    },
    domain::{SystemLogDetail, SystemLogFilter, SystemLogSummary},
};

#[tokio::test]
async fn export_route_delegates_to_application_and_streams_the_workbook() {
    let captured = Arc::new(Mutex::new(None));
    let app = create_router(state(captured.clone())).layer(middleware::from_fn(types::http::locale_middleware));
    let response = app
        .oneshot(
            Request::post("/system/system-logs/export?begin_time=2026-07-20T00%3A00%3A00Z&end_time=2026-07-20T23%3A59%3A59Z")
                .header(header::ACCEPT_LANGUAGE, "en")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_TYPE],
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    );
    assert_eq!(response.headers()[header::CONTENT_DISPOSITION], "attachment; filename=\"system_logs.xlsx\"");
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    assert_eq!(read_xlsx(&bytes).unwrap(), vec![vec!["delegated"], vec!["yes"]]);

    let request = captured.lock().unwrap().take().expect("export use case was not called");
    assert_eq!(request.batch, ExportBatchConfig { page_size: 7 });
    assert_eq!(request.layout, system_log_export_layout(types::http::Locale::En));
    assert!(request.filter.begin_time.is_some());
    assert!(request.filter.end_time.is_some());
}

#[tokio::test]
async fn single_delete_marks_operation_audit_persisted_after_repository_success() {
    let context = operation_context();
    let logs = Arc::new(DeleteLogs::new(false));
    let response = create_router(delete_state(logs.clone()))
        .layer(Extension(context.clone()))
        .oneshot(Request::delete("/system/system-logs/log-1").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(context.is_persisted());
    assert_eq!(logs.ids(), vec![vec!["log-1".to_owned()]]);
}

#[tokio::test]
async fn batch_delete_leaves_operation_audit_unpersisted_when_repository_fails() {
    let context = operation_context();
    let logs = Arc::new(DeleteLogs::new(true));
    let response = create_router(delete_state(logs.clone()))
        .layer(Extension(context.clone()))
        .oneshot(
            Request::delete("/system/system-logs/batch")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"ids":["log-1","log-2"]}"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert!(!context.is_persisted());
    assert_eq!(logs.ids(), vec![vec!["log-1".to_owned(), "log-2".to_owned()]]);
}

fn state(captured: Arc<Mutex<Option<SystemLogExportRequest>>>) -> SystemLogApiState {
    SystemLogApiState::new(SystemLogApiStateParts {
        logs: Arc::new(UnusedLogs),
        exporter: Arc::new(CapturingExporter { captured }),
        cleanup_executions: Arc::new(UnusedCleanupExecutions),
        export_config: Arc::new(TestExportConfig),
    })
}

fn delete_state(logs: Arc<dyn SystemLogUseCase>) -> SystemLogApiState {
    SystemLogApiState::new(SystemLogApiStateParts {
        logs,
        exporter: Arc::new(UnusedExporter),
        cleanup_executions: Arc::new(UnusedCleanupExecutions),
        export_config: Arc::new(TestExportConfig),
    })
}

fn operation_context() -> OperationAuditContext {
    let context = OperationAuditContext::new(
        OperationAuditSeed {
            id: "audit-delete-request".into(),
            occurred_at: time::OffsetDateTime::UNIX_EPOCH,
            title_key: "audit.module.system_log".into(),
            business_type: BusinessType::Delete,
            handler: "observability::delete_system_logs".into(),
            request_method: "DELETE".into(),
            operator_type: OperatorType::Manage,
            operation_url: "/api/system/system-logs".into(),
            operation_ip: "127.0.0.1".into(),
            request_id: "request-delete".into(),
        },
        Arc::new(EmptySnapshot),
    );
    context
        .set_actor(ActorSnapshot {
            user_id: Some("user-1".into()),
            username: "alice".into(),
            department_id: None,
            department_name: String::new(),
        })
        .unwrap();
    context
}

struct CapturingExporter {
    captured: Arc<Mutex<Option<SystemLogExportRequest>>>,
}

#[async_trait]
impl SystemLogExportUseCase for CapturingExporter {
    async fn export_xlsx(&self, request: SystemLogExportRequest) -> ObservabilityResult<TemporaryXlsxFile> {
        *self.captured.lock().unwrap() = Some(request);
        StreamingXlsxWriter::new("delegated", &["delegated"])
            .and_then(|mut writer| {
                writer.append_rows(&[vec!["yes".into()]])?;
                writer.finish()
            })
            .map_err(crate::application::ObservabilityError::Infrastructure)
    }
}

struct TestExportConfig;

#[async_trait]
impl ExportConfigProvider for TestExportConfig {
    type Error = crate::application::ObservabilityError;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error> {
        Ok(ExportBatchConfig { page_size: 7 })
    }
}

struct UnusedExporter;

#[async_trait]
impl SystemLogExportUseCase for UnusedExporter {
    async fn export_xlsx(&self, _: SystemLogExportRequest) -> ObservabilityResult<TemporaryXlsxFile> {
        unreachable!()
    }
}

struct EmptySnapshot;

impl OperationRequestSnapshot for EmptySnapshot {
    fn request_params(&self) -> String {
        String::new()
    }
}

struct DeleteLogs {
    fail: bool,
    calls: Mutex<Vec<(Vec<String>, AuditOutboxRecord)>>,
}

impl DeleteLogs {
    fn new(fail: bool) -> Self {
        Self {
            fail,
            calls: Mutex::new(Vec::new()),
        }
    }

    fn ids(&self) -> Vec<Vec<String>> {
        self.calls.lock().unwrap().iter().map(|(ids, _)| ids.clone()).collect()
    }
}

#[async_trait]
impl SystemLogUseCase for DeleteLogs {
    async fn page(&self, _: SystemLogFilter, _: CursorPageRequest) -> ObservabilityResult<CursorPage<SystemLogSummary>> {
        unreachable!()
    }

    async fn detail(&self, _: &str) -> ObservabilityResult<SystemLogDetail> {
        unreachable!()
    }

    async fn delete_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> ObservabilityResult<()> {
        self.calls.lock().unwrap().push((ids, audit));
        if self.fail { Err(ObservabilityError::NotFound) } else { Ok(()) }
    }

    async fn count(&self, _: SystemLogFilter) -> ObservabilityResult<u64> {
        unreachable!()
    }

    async fn delete_filtered(&self, _: SystemLogFilter, _: u64) -> ObservabilityResult<SystemLogRetentionReport> {
        unreachable!()
    }
}

struct UnusedLogs;

#[async_trait]
impl SystemLogUseCase for UnusedLogs {
    async fn page(&self, _: SystemLogFilter, _: CursorPageRequest) -> ObservabilityResult<CursorPage<SystemLogSummary>> {
        unreachable!()
    }

    async fn detail(&self, _: &str) -> ObservabilityResult<SystemLogDetail> {
        unreachable!()
    }

    async fn delete_with_audit(&self, _: Vec<String>, _: AuditOutboxRecord) -> ObservabilityResult<()> {
        unreachable!()
    }

    async fn count(&self, _: SystemLogFilter) -> ObservabilityResult<u64> {
        unreachable!()
    }

    async fn delete_filtered(&self, _: SystemLogFilter, _: u64) -> ObservabilityResult<SystemLogRetentionReport> {
        unreachable!()
    }
}

struct UnusedCleanupExecutions;

#[async_trait]
impl SystemLogCleanupExecutionPort for UnusedCleanupExecutions {
    async fn enqueue_manual_cleanup(&self, _: ManualSystemLogCleanupRequest) -> ObservabilityResult<String> {
        unreachable!()
    }

    async fn cleanup_execution(&self, _: &str) -> ObservabilityResult<SystemLogCleanupExecution> {
        unreachable!()
    }
}
