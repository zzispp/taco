use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use audit::api::OperationAuditState;
use audit_contract::{
    ActorSnapshot, AuditOutboxEvent, AuditOutboxRecord, AuditOutboxRecorder, AuditOutboxResult, AuditStatus, BusinessType, EndpointAccess, EndpointAudit,
    EndpointMethod, EndpointSpec, OperationAuditContext, OperationEndpointAudit, RequestCapture,
};
use axum::{
    Router,
    body::Body,
    extract::{ConnectInfo, Request},
    http::{HeaderValue, Method, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
};
use tower::ServiceExt;

use super::super::http_pipeline::{RuntimeLayerParts, apply_runtime_layers};

const TEST_TIMEOUT_MS: u64 = 5;
const SLOW_DELAY_MS: u64 = 30;
const SLOW_AUTH_PATH: &str = "/api/__composition-slow-auth";
const AUTHORIZED_TIMEOUT_PATH: &str = "/api/__composition-authorized-timeout";
const SUCCESS_PATH: &str = "/api/__composition-metrics-success";
const TIMEOUT_PATH: &str = "/api/__composition-metrics-timeout";
const STATIC_PATH: &str = "/uploads/avatars/avatar.png";
const DOCS_PATH: &str = "/docs";
const AVATAR_PROJECTION_PATH: &str = "/api/avatars/user-1/1";

#[derive(Clone, Default)]
struct MemoryRecorder(Arc<Mutex<Vec<AuditOutboxRecord>>>);

impl MemoryRecorder {
    fn records(&self) -> Vec<AuditOutboxRecord> {
        self.0.lock().unwrap().clone()
    }
}

#[async_trait]
impl AuditOutboxRecorder for MemoryRecorder {
    async fn record(&self, record: AuditOutboxRecord) -> AuditOutboxResult<()> {
        self.0.lock().unwrap().push(record);
        Ok(())
    }
}

#[tokio::test]
async fn global_timeout_cancels_slow_authentication_without_auditing() {
    let mut settings = super::test_settings();
    settings.http.request_timeout_ms = TEST_TIMEOUT_MS;
    let recorder = MemoryRecorder::default();
    let routes = Router::new().route(SLOW_AUTH_PATH, post(ok_handler)).layer(middleware::from_fn(slow_auth));
    let metrics = None;
    let app = apply_runtime_layers(
        routes,
        RuntimeLayerParts {
            settings: &settings,
            audit: audit_state(Arc::new(recorder.clone()), vec![operation_endpoint(SLOW_AUTH_PATH)]),
            metrics: &metrics,
            http_logs: None,
        },
    )
    .unwrap();

    let response = app.oneshot(request(SLOW_AUTH_PATH)).await.unwrap();

    assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    assert_eq!(recorder.records(), Vec::new());
}

#[tokio::test]
async fn authorized_handler_timeout_is_written_to_the_operation_outbox() {
    let mut settings = super::test_settings();
    settings.http.request_timeout_ms = TEST_TIMEOUT_MS;
    let recorder = MemoryRecorder::default();
    let routes = Router::new()
        .route(AUTHORIZED_TIMEOUT_PATH, post(slow_handler))
        .layer(middleware::from_fn(authorize));
    let metrics = None;
    let app = apply_runtime_layers(
        routes,
        RuntimeLayerParts {
            settings: &settings,
            audit: audit_state(Arc::new(recorder.clone()), vec![operation_endpoint(AUTHORIZED_TIMEOUT_PATH)]),
            metrics: &metrics,
            http_logs: None,
        },
    )
    .unwrap();

    let response = app.oneshot(request(AUTHORIZED_TIMEOUT_PATH)).await.unwrap();

    assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    let records = recorder.records();
    let [record] = records.as_slice() else {
        panic!("expected one timeout audit record, got {}", records.len());
    };
    let AuditOutboxEvent::Operation(event) = &record.event else {
        panic!("expected an operation audit record");
    };
    assert_eq!(event.actor.username, "alice");
    assert_eq!(event.status, AuditStatus::Failure);
    assert_eq!(event.error_message, "http_status_408");
}

#[tokio::test]
async fn metrics_cover_success_and_timeout_responses() {
    let mut settings = super::test_settings();
    settings.http.request_timeout_ms = TEST_TIMEOUT_MS;
    let metrics = taco_tracing::init_metrics(taco_tracing::MetricsConfig { enabled: true }).unwrap();
    let routes = Router::new().route(SUCCESS_PATH, post(ok_handler)).route(TIMEOUT_PATH, post(slow_handler));
    let app = apply_runtime_layers(
        routes,
        RuntimeLayerParts {
            settings: &settings,
            audit: audit_state(Arc::new(MemoryRecorder::default()), Vec::new()),
            metrics: &metrics,
            http_logs: None,
        },
    )
    .unwrap();

    let success = app.clone().oneshot(request(SUCCESS_PATH)).await.unwrap();
    let timeout = app.oneshot(request(TIMEOUT_PATH)).await.unwrap();

    assert_eq!(success.status(), StatusCode::OK);
    assert_eq!(timeout.status(), StatusCode::REQUEST_TIMEOUT);
    let rendered = metrics.as_ref().unwrap().render();
    assert_metric_count(
        &rendered,
        MetricExpectation {
            route: SUCCESS_PATH,
            status: "200",
            count: "1",
        },
    );
    assert_metric_count(
        &rendered,
        MetricExpectation {
            route: TIMEOUT_PATH,
            status: "408",
            count: "1",
        },
    );
}

#[tokio::test]
async fn browser_security_headers_are_global_with_route_specific_cache_and_csp() {
    let settings = super::test_settings();
    let metrics = None;
    let routes = Router::new()
        .route(SUCCESS_PATH, post(ok_handler))
        .route(STATIC_PATH, post(ok_handler))
        .route(DOCS_PATH, post(ok_handler))
        .route(AVATAR_PROJECTION_PATH, post(avatar_handler));
    let app = apply_runtime_layers(
        routes,
        RuntimeLayerParts {
            settings: &settings,
            audit: audit_state(Arc::new(MemoryRecorder::default()), Vec::new()),
            metrics: &metrics,
            http_logs: None,
        },
    )
    .unwrap();

    let api = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(SUCCESS_PATH)
                .header(header::ORIGIN, "https://other.example.test")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let static_asset = app.clone().oneshot(request(STATIC_PATH)).await.unwrap();
    let docs = app.clone().oneshot(request(DOCS_PATH)).await.unwrap();
    let avatar = app.oneshot(request(AVATAR_PROJECTION_PATH)).await.unwrap();

    for response in [&api, &static_asset, &docs, &avatar] {
        assert_eq!(
            response.headers().get(header::X_CONTENT_TYPE_OPTIONS),
            Some(&HeaderValue::from_static("nosniff"))
        );
        assert_eq!(response.headers().get(header::X_FRAME_OPTIONS), Some(&HeaderValue::from_static("DENY")));
        assert_eq!(response.headers().get(header::REFERRER_POLICY), Some(&HeaderValue::from_static("no-referrer")));
        assert_eq!(
            response.headers().get("permissions-policy"),
            Some(&HeaderValue::from_static("camera=(), microphone=(), geolocation=()"))
        );
    }
    assert_eq!(api.headers().get(header::CACHE_CONTROL), Some(&HeaderValue::from_static("no-store")));
    assert_eq!(api.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN), None);
    assert_eq!(static_asset.headers().get(header::CACHE_CONTROL), None);
    assert_eq!(docs.headers().get(header::CACHE_CONTROL), None);
    assert_eq!(
        avatar.headers().get(header::CACHE_CONTROL),
        Some(&HeaderValue::from_static("public, max-age=31536000, immutable"))
    );
    assert_eq!(api.headers().get(header::CONTENT_SECURITY_POLICY), None);
    assert_eq!(static_asset.headers().get(header::CONTENT_SECURITY_POLICY), None);
    assert_eq!(
        docs.headers().get(header::CONTENT_SECURITY_POLICY),
        Some(&HeaderValue::from_static("frame-ancestors 'none'; object-src 'none'; base-uri 'none'"))
    );
}

fn audit_state(recorder: Arc<dyn AuditOutboxRecorder>, specs: Vec<EndpointSpec>) -> OperationAuditState {
    OperationAuditState::try_new(specs, recorder).unwrap()
}

fn operation_endpoint(path: &'static str) -> EndpointSpec {
    EndpointSpec {
        method: EndpointMethod::Post,
        path,
        access: EndpointAccess::Authenticated,
        audit: EndpointAudit::Operation(OperationEndpointAudit {
            title_key: "audit.module.user",
            business_type: BusinessType::Other,
            handler: "composition::slow_handler",
            request_capture: RequestCapture::None,
        }),
    }
}

fn request(path: &str) -> Request {
    let mut request = Request::builder().method(Method::POST).uri(path).body(Body::empty()).unwrap();
    request.extensions_mut().insert(ConnectInfo("127.0.0.1:3000".parse::<SocketAddr>().unwrap()));
    request
}

async fn authorize(request: Request, next: Next) -> Response {
    request
        .extensions()
        .get::<OperationAuditContext>()
        .expect("operation audit middleware must insert an audit context")
        .set_actor(ActorSnapshot {
            user_id: Some("user-1".into()),
            username: "alice".into(),
            department_id: None,
            department_name: String::new(),
        })
        .unwrap();
    next.run(request).await
}

async fn slow_auth(request: Request, next: Next) -> Response {
    tokio::time::sleep(Duration::from_millis(SLOW_DELAY_MS)).await;
    next.run(request).await
}

async fn slow_handler() -> StatusCode {
    tokio::time::sleep(Duration::from_millis(SLOW_DELAY_MS)).await;
    StatusCode::OK
}

async fn ok_handler() -> StatusCode {
    StatusCode::OK
}

async fn avatar_handler() -> Response {
    let mut response = StatusCode::OK.into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("public, max-age=31536000, immutable"));
    response
}

struct MetricExpectation<'a> {
    route: &'a str,
    status: &'a str,
    count: &'a str,
}

fn assert_metric_count(rendered: &str, expected: MetricExpectation<'_>) {
    let line = rendered
        .lines()
        .find(|line| {
            line.starts_with("http_requests_total{")
                && line.contains(&format!("route=\"{}\"", expected.route))
                && line.contains(&format!("status=\"{}\"", expected.status))
        })
        .expect("expected HTTP counter series");
    assert_eq!(line.split_whitespace().last(), Some(expected.count));
}
