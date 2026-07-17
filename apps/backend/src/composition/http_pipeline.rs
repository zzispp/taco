use audit::api::{OperationAuditState, operation_audit_middleware};
use axum::{
    Router,
    extract::Request,
    http::{HeaderValue, header},
    middleware::{self, Next},
    response::Response,
};
use configuration::Settings;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};

use crate::{BackendResult, http_config};

const API_PATH_PREFIX: &str = "/api/";
const DOCS_PATH: &str = "/docs";
const PERMISSIONS_POLICY: &str = "permissions-policy";
const PERMISSIONS_POLICY_VALUE: &str = "camera=(), microphone=(), geolocation=()";
const DOCS_CONTENT_SECURITY_POLICY: &str = "frame-ancestors 'none'; object-src 'none'; base-uri 'none'";

pub(super) struct RuntimeLayerParts<'a> {
    pub settings: &'a Settings,
    pub audit: OperationAuditState,
    pub metrics: &'a taco_tracing::MetricsHandle,
    pub http_logs: Option<taco_tracing::HttpLogCaptureState>,
}

pub(super) fn apply_runtime_layers(app: Router, parts: RuntimeLayerParts<'_>) -> BackendResult<Router> {
    let app = app.layer(middleware::from_fn(types::http::locale_middleware));
    let app = with_timeout(app, parts.settings)?;
    let app = app.layer(middleware::from_fn_with_state(parts.audit, operation_audit_middleware));
    let app = match parts.http_logs {
        Some(state) => app.layer(middleware::from_fn_with_state(state, taco_tracing::http_log_middleware)),
        None => app,
    };
    let app = apply_metrics_layer(app, parts.metrics);
    apply_http_layers(app, parts.settings)
}

pub(super) fn add_metrics_route(mut app: Router, handle: &taco_tracing::MetricsHandle) -> Router {
    if let Some(handle) = handle {
        let handle = handle.clone();
        app = app.route("/metrics", axum::routing::get(move || taco_tracing::metrics_handler(handle.clone())));
    }
    app
}

pub(super) fn apply_metrics_layer(app: Router, handle: &taco_tracing::MetricsHandle) -> Router {
    if handle.is_none() {
        return app;
    }
    app.layer(middleware::from_fn(taco_tracing::metrics_middleware))
}

pub(super) fn with_timeout(app: Router, settings: &Settings) -> BackendResult<Router> {
    Ok(app.layer(http_config::timeout_layer(settings)?))
}

pub(crate) fn apply_http_layers(app: Router, settings: &Settings) -> BackendResult<Router> {
    Ok(app
        .layer(http_config::compression_layer(settings)?)
        .layer(http_config::cors_layer(settings)?)
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(middleware::from_fn(browser_security_headers)))
}

async fn browser_security_headers(request: Request, next: Next) -> Response {
    let path = request.uri().path();
    let is_api = path.starts_with(API_PATH_PREFIX);
    let is_docs = path == DOCS_PATH || path.starts_with("/docs/");
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(header::REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
    headers.insert(PERMISSIONS_POLICY, HeaderValue::from_static(PERMISSIONS_POLICY_VALUE));
    if is_api {
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    }
    if is_docs {
        headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(DOCS_CONTENT_SECURITY_POLICY));
    }
    response
}
