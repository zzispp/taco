use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use audit_contract::{
    ActorSnapshot, AuditOutboxError, AuditOutboxEvent, AuditOutboxRecord, AuditOutboxRecorder, AuditOutboxResult, BusinessType, EndpointAccess, EndpointAudit,
    EndpointMethod, EndpointSpec, OperationAuditEvent, OperationEndpointAudit, RequestCapture,
};
use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{ConnectInfo, Request},
    http::{Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::{Value, json};
use tower::ServiceExt;
use types::http::RequestJson;

use super::{AuditActorContext, OperationAuditState, operation_audit_middleware};

#[derive(Clone, Default)]
struct MemoryRecorder(Arc<Mutex<Vec<AuditOutboxRecord>>>);

impl MemoryRecorder {
    fn events(&self) -> Vec<OperationAuditEvent> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .map(|record| match &record.event {
                AuditOutboxEvent::Operation(event) => event.clone(),
                AuditOutboxEvent::Security(_) => panic!("expected operation audit event"),
            })
            .collect()
    }
}

#[async_trait]
impl AuditOutboxRecorder for MemoryRecorder {
    async fn record(&self, record: AuditOutboxRecord) -> AuditOutboxResult<()> {
        self.0.lock().unwrap().push(record);
        Ok(())
    }
}

struct FailingRecorder;

#[async_trait]
impl AuditOutboxRecorder for FailingRecorder {
    async fn record(&self, _record: AuditOutboxRecord) -> AuditOutboxResult<()> {
        Err(AuditOutboxError::Infrastructure("database unavailable".into()))
    }
}

#[tokio::test]
async fn successful_operation_uses_immutable_actor_and_never_saves_response_body() {
    let recorder = MemoryRecorder::default();
    let app = app(Arc::new(recorder.clone()), RequestCapture::Sanitized);

    let response = app
        .oneshot(json_request(
            br#"{"password":"raw","body":{"secret":"nested"},"endpoint":"https://user:pass@example.com/run?token=query#fragment"}"#,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response_body(response).await, json!({"access_token": "response-secret", "ok": true}));
    let events = recorder.events();
    assert_eq!(events.len(), 1);
    let event = &events[0];
    assert_eq!(event.status, audit_contract::AuditStatus::Success);
    assert_eq!(event.actor.username, "alice");
    assert_eq!(event.actor.department_name, "Engineering");
    assert_eq!(event.operation_ip, "198.51.100.8");
    assert_eq!(event.request_id, "request-1");
    assert_eq!(
        event.request_params,
        r#"{"body":"[body omitted]","endpoint":"https://example.com/run?token=***","password":"***"}"#
    );
    assert_eq!(event.response_result, "");
    assert_eq!(event.error_message, "");
    assert!(!event.response_result.contains("response-secret"));
}

#[tokio::test]
async fn authenticated_forbidden_operation_is_persisted_before_returning_403() {
    let recorder = MemoryRecorder::default();
    let state = state(Arc::new(recorder.clone()), RequestCapture::Sanitized);
    let app = Router::new()
        .route("/api/test", post(test_handler))
        .layer(middleware::from_fn(authorize_then_forbid))
        .layer(middleware::from_fn_with_state(state, operation_audit_middleware));

    let response = app.oneshot(json_request(br#"{"password":"raw"}"#)).await.unwrap();

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let events = recorder.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].status, audit_contract::AuditStatus::Failure);
    assert_eq!(events[0].error_message, "http_status_403");
}

#[tokio::test]
async fn unauthenticated_rejection_does_not_invent_an_operator() {
    let recorder = MemoryRecorder::default();
    let state = state(Arc::new(recorder.clone()), RequestCapture::Sanitized);
    let app = Router::new()
        .route("/api/test", post(test_handler))
        .layer(middleware::from_fn(reject_unauthenticated))
        .layer(middleware::from_fn_with_state(state, operation_audit_middleware));

    let response = app.oneshot(json_request(br#"{}"#)).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(recorder.events().is_empty());
}

#[tokio::test]
async fn outbox_record_failure_is_exposed_instead_of_returning_a_successful_operation() {
    let app = app(Arc::new(FailingRecorder), RequestCapture::Sanitized);

    let response = app.oneshot(json_request(br#"{}"#)).await.unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn request_capture_none_never_reads_a_config_payload() {
    let recorder = MemoryRecorder::default();
    let app = app(Arc::new(recorder.clone()), RequestCapture::None);

    let response = app.oneshot(json_request(br#"{"config_value":"secret"}"#)).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let events = recorder.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].request_params, "");
}

#[tokio::test]
async fn explicit_download_audit_persists_a_get_event_without_response_capture() {
    let recorder = MemoryRecorder::default();
    let app = Router::new()
        .route("/api/download", get(empty_handler))
        .layer(middleware::from_fn(authorize))
        .layer(middleware::from_fn_with_state(
            download_state(Arc::new(recorder.clone())),
            operation_audit_middleware,
        ));

    let response = app.oneshot(download_request()).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let events = recorder.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].request_method, "GET");
    assert_eq!(events[0].business_type, BusinessType::Export);
    assert_eq!(events[0].handler, "test::download");
    assert_eq!(events[0].request_params, "");
}

fn app(recorder: Arc<dyn AuditOutboxRecorder>, request_capture: RequestCapture) -> Router {
    Router::new()
        .route("/api/test", post(test_handler))
        .layer(middleware::from_fn(authorize))
        .layer(middleware::from_fn_with_state(state(recorder, request_capture), operation_audit_middleware))
}

async fn test_handler(RequestJson(_input): RequestJson<Value>) -> Json<Value> {
    Json(json!({"ok": true, "access_token": "response-secret"}))
}

async fn empty_handler() -> StatusCode {
    StatusCode::OK
}

fn state(recorder: Arc<dyn AuditOutboxRecorder>, request_capture: RequestCapture) -> OperationAuditState {
    OperationAuditState::try_new(vec![operation_endpoint(request_capture)], recorder).unwrap()
}

fn download_state(recorder: Arc<dyn AuditOutboxRecorder>) -> OperationAuditState {
    OperationAuditState::try_new(
        vec![EndpointSpec {
            method: EndpointMethod::Get,
            path: "/api/download",
            access: EndpointAccess::Authenticated,
            audit: EndpointAudit::Download(OperationEndpointAudit {
                title_key: "audit.module.file",
                business_type: BusinessType::Export,
                handler: "test::download",
                request_capture: RequestCapture::None,
            }),
        }],
        recorder,
    )
    .unwrap()
}

fn operation_endpoint(request_capture: RequestCapture) -> EndpointSpec {
    EndpointSpec {
        method: EndpointMethod::Post,
        path: "/api/test",
        access: EndpointAccess::Authenticated,
        audit: EndpointAudit::Operation(OperationEndpointAudit {
            title_key: "audit.module.user",
            business_type: BusinessType::Insert,
            handler: "test::handler",
            request_capture,
        }),
    }
}

async fn authorize(request: Request, next: Next) -> Response {
    request
        .extensions()
        .get::<AuditActorContext>()
        .expect("audit envelope inserted actor context")
        .set_actor(actor())
        .unwrap();
    next.run(request).await
}

async fn authorize_then_forbid(request: Request, _next: Next) -> Response {
    request
        .extensions()
        .get::<AuditActorContext>()
        .expect("audit envelope inserted actor context")
        .set_actor(actor())
        .unwrap();
    StatusCode::FORBIDDEN.into_response()
}

async fn reject_unauthenticated(_request: Request, _next: Next) -> Response {
    StatusCode::UNAUTHORIZED.into_response()
}

fn actor() -> ActorSnapshot {
    ActorSnapshot {
        user_id: Some("user-1".into()),
        username: "alice".into(),
        department_id: Some("dept-1".into()),
        department_name: "Engineering".into(),
    }
}

fn json_request(body: &'static [u8]) -> Request {
    let mut request = Request::builder()
        .method(Method::POST)
        .uri("/api/test")
        .header("content-type", "application/json")
        .header("x-forwarded-for", "198.51.100.8, 10.0.0.2")
        .header("x-request-id", "request-1")
        .body(Body::from(body))
        .unwrap();
    request.extensions_mut().insert(ConnectInfo("10.0.0.2:8080".parse::<SocketAddr>().unwrap()));
    request
}

fn download_request() -> Request {
    let mut request = Request::builder().method(Method::GET).uri("/api/download").body(Body::empty()).unwrap();
    request.extensions_mut().insert(ConnectInfo("10.0.0.2:8080".parse::<SocketAddr>().unwrap()));
    request
}

async fn response_body(response: Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}
