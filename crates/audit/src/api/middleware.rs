use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Instant,
};

use audit_contract::{
    AuditOutboxError, AuditOutboxRecorder, AuditStatus, EndpointSpec, OperationAuditContext, OperationAuditSeed, OperationEndpointAudit, OperationOutcome,
    OperationRequestSnapshot, OperatorType, RequestCapture as EndpointRequestCapture,
};
use axum::{
    Json,
    extract::{ConnectInfo, OriginalUri, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use client_info::ClientInfo;
use kernel::error::LocalizedError;
use time::OffsetDateTime;
use types::http::{ApiErrorKind, current_locale, localized_error_response};

use crate::application::{AuditError, AuditResult};

use super::{
    body_capture::{CaptureTrace, RequestCapture, capture_request},
    endpoint_catalog::EndpointAuditCatalog,
    sanitize::{normalized_request_id, sanitized_url},
};

pub use audit_contract::OperationAuditContext as AuditActorContext;
mod trace;
use trace::{request_id, trace_missing_peer};

#[derive(Clone)]
pub struct OperationAuditState {
    catalog: Arc<EndpointAuditCatalog>,
    recorder: Arc<dyn AuditOutboxRecorder>,
}

impl OperationAuditState {
    pub fn try_new(specs: Vec<EndpointSpec>, recorder: Arc<dyn AuditOutboxRecorder>) -> AuditResult<Self> {
        Ok(Self {
            catalog: Arc::new(EndpointAuditCatalog::new(specs)?),
            recorder,
        })
    }
}

/// Persists one durable operation-audit event before returning an authenticated
/// audited response. It deliberately never captures a response body.
pub async fn operation_audit_middleware(State(state): State<OperationAuditState>, request: Request, next: Next) -> Response {
    let uri = original_uri(&request);
    let Some(policy) = state.catalog.find(request.method().as_str(), uri.path()) else {
        return next.run(request).await;
    };
    let Some(peer) = request.extensions().get::<ConnectInfo<SocketAddr>>().map(|value| value.0) else {
        trace_missing_peer(&uri, request.headers());
        return audit_infrastructure_response("operation audit requires peer connection information");
    };
    audit_declared_request(DeclaredRequest { state, policy, uri, peer }, request, next).await
}

fn original_uri(request: &Request) -> axum::http::Uri {
    request
        .extensions()
        .get::<OriginalUri>()
        .map(|value| value.0.clone())
        .unwrap_or_else(|| request.uri().clone())
}

struct DeclaredRequest {
    state: OperationAuditState,
    policy: OperationEndpointAudit,
    uri: axum::http::Uri,
    peer: SocketAddr,
}

async fn audit_declared_request(input: DeclaredRequest, request: Request, next: Next) -> Response {
    let started = Instant::now();
    let occurred_at = OffsetDateTime::now_utc();
    let context = request_context(&request, &input.uri, input.peer);
    let (request, request_capture) = capture_request(request, input.policy.request_capture == EndpointRequestCapture::Sanitized);
    let operation_context = OperationAuditContext::new(
        operation_seed(input.policy, &context, occurred_at),
        Arc::new(CapturedRequest::new(request_capture, &context)),
    );
    let mut request = request;
    request.extensions_mut().insert(operation_context.clone());
    let response = next.run(request).await;
    if operation_context.is_persisted() {
        return response;
    }
    let outcome = match operation_outcome(response.status(), started) {
        Ok(outcome) => outcome,
        Err(error) => return audit_error_response(&context, error),
    };
    let Some(record) = operation_context.record(outcome) else {
        return response;
    };
    persist_or_error(PersistOperation {
        response,
        recorder: input.state.recorder.as_ref(),
        record,
        context: &context,
    })
    .await
}

#[derive(Clone)]
struct RequestContext {
    request_id: String,
    method: String,
    url: String,
    operation_ip: String,
}

fn request_context(request: &Request, uri: &axum::http::Uri, peer: SocketAddr) -> RequestContext {
    let client = ClientInfo::from_headers(request.headers(), peer);
    RequestContext {
        request_id: normalized_request_id(&request_id(request.headers())),
        method: request.method().as_str().into(),
        url: sanitized_url(uri),
        operation_ip: client.ipaddr(),
    }
}

fn operation_outcome(response_status: StatusCode, started: Instant) -> AuditResult<OperationOutcome> {
    let cost_time_ms = i64::try_from(started.elapsed().as_millis())
        .map_err(|error| AuditError::Infrastructure(format!("operation audit duration conversion failed: {error}")))?;
    Ok(OperationOutcome {
        status: audit_status(response_status),
        response_result: String::new(),
        error_message: stable_status_error(response_status),
        cost_time_ms,
    })
}

fn audit_status(value: StatusCode) -> AuditStatus {
    if value.is_success() { AuditStatus::Success } else { AuditStatus::Failure }
}

fn stable_status_error(status: StatusCode) -> String {
    if status.is_success() {
        String::new()
    } else {
        format!("http_status_{}", status.as_u16())
    }
}

fn operation_seed(policy: OperationEndpointAudit, context: &RequestContext, occurred_at: OffsetDateTime) -> OperationAuditSeed {
    OperationAuditSeed {
        id: uuid::Uuid::now_v7().to_string(),
        occurred_at,
        title_key: policy.title_key.into(),
        business_type: policy.business_type,
        handler: policy.handler.into(),
        request_method: context.method.clone(),
        operator_type: OperatorType::Manage,
        operation_url: context.url.clone(),
        operation_ip: context.operation_ip.clone(),
        request_id: context.request_id.clone(),
    }
}

struct CapturedRequest {
    capture: Mutex<RequestSnapshotState>,
    request_id: String,
    route: String,
}

enum RequestSnapshotState {
    Pending(RequestCapture),
    Ready(String),
}

impl CapturedRequest {
    fn new(capture: RequestCapture, context: &RequestContext) -> Self {
        Self {
            capture: Mutex::new(RequestSnapshotState::Pending(capture)),
            request_id: context.request_id.clone(),
            route: context.url.clone(),
        }
    }
}

impl OperationRequestSnapshot for CapturedRequest {
    fn request_params(&self) -> String {
        let mut state = self.capture.lock().expect("operation audit request snapshot state is valid");
        match &*state {
            RequestSnapshotState::Ready(value) => return value.clone(),
            RequestSnapshotState::Pending(_) => {}
        }
        let RequestSnapshotState::Pending(capture) = std::mem::replace(&mut *state, RequestSnapshotState::Ready(String::new())) else {
            unreachable!("request audit snapshot state changed while locked");
        };
        let value = capture.finish(CaptureTrace {
            request_id: &self.request_id,
            route: &self.route,
            phase: "request",
        });
        *state = RequestSnapshotState::Ready(value.clone());
        value
    }
}

struct PersistOperation<'a> {
    response: Response,
    recorder: &'a dyn AuditOutboxRecorder,
    record: audit_contract::AuditOutboxRecord,
    context: &'a RequestContext,
}

async fn persist_or_error(input: PersistOperation<'_>) -> Response {
    match input.recorder.record(input.record).await {
        Ok(()) => input.response,
        Err(error) => audit_recorder_error_response(input.context, error),
    }
}

fn audit_error_response(context: &RequestContext, error: AuditError) -> Response {
    taco_tracing::error_with_fields!(
        "operation audit construction failed",
        &error,
        request_id = context.request_id,
        route = context.url,
        event_type = "operation",
        reason = "construction_failed"
    );
    audit_infrastructure_response("operation audit construction failed")
}

fn audit_recorder_error_response(context: &RequestContext, error: AuditOutboxError) -> Response {
    taco_tracing::error_with_fields!(
        "operation audit outbox recording failed",
        &error,
        request_id = context.request_id,
        route = context.url,
        event_type = "operation",
        reason = "outbox_record_failed"
    );
    audit_infrastructure_response("operation audit outbox recording failed")
}

fn audit_infrastructure_response(reason: &'static str) -> Response {
    let body = localized_error_response(
        current_locale(),
        ApiErrorKind::Infrastructure,
        Some(&LocalizedError::new("errors.common.service_unavailable")),
    );
    let error = std::io::Error::other(reason);
    taco_tracing::error_with_fields!("operation audit infrastructure failure", &error, reason = reason, event_type = "operation");
    (StatusCode::INTERNAL_SERVER_ERROR, Json(body)).into_response()
}
