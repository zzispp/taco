use std::{
    future::Future,
    sync::{Mutex, OnceLock},
    time::Instant,
};

use axum::{
    extract::{MatchedPath, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use thiserror::Error;

const METRICS_ROUTE: &str = "/metrics";
const UNMATCHED_ROUTE: &str = "unmatched";
const HTTP_REQUESTS_TOTAL: &str = "http_requests_total";
const HTTP_REQUEST_DURATION_SECONDS: &str = "http_request_duration_seconds";
const HTTP_REQUESTS_IN_FLIGHT: &str = "http_requests_in_flight";
const DB_QUERIES_TOTAL: &str = "db_queries_total";
const DB_QUERY_DURATION_SECONDS: &str = "db_query_duration_seconds";
const HTTP_HISTOGRAM_BUCKETS: [f64; 11] = [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0];
const DB_HISTOGRAM_BUCKETS: [f64; 9] = [0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0];

static METRICS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();
static METRICS_INIT_LOCK: Mutex<()> = Mutex::new(());

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MetricsConfig {
    pub enabled: bool,
}

pub type MetricsHandle = Option<PrometheusHandle>;

#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("prometheus setup failed: {0}")]
    Install(String),
}

pub fn init_metrics(config: MetricsConfig) -> Result<MetricsHandle, MetricsError> {
    if !config.enabled {
        return Ok(None);
    }

    if let Some(handle) = METRICS_HANDLE.get() {
        return Ok(Some(handle.clone()));
    }

    let _guard = METRICS_INIT_LOCK
        .lock()
        .map_err(|_| MetricsError::Install("metrics init lock poisoned".into()))?;

    if let Some(handle) = METRICS_HANDLE.get() {
        return Ok(Some(handle.clone()));
    }

    let handle = PrometheusBuilder::new()
        .set_buckets_for_metric(Matcher::Full(HTTP_REQUEST_DURATION_SECONDS.into()), &HTTP_HISTOGRAM_BUCKETS)
        .map_err(|error| MetricsError::Install(error.to_string()))?
        .set_buckets_for_metric(Matcher::Full(DB_QUERY_DURATION_SECONDS.into()), &DB_HISTOGRAM_BUCKETS)
        .map_err(|error| MetricsError::Install(error.to_string()))?
        .install_recorder()
        .map_err(|error| MetricsError::Install(error.to_string()))?;

    let _ = METRICS_HANDLE.set(handle.clone());
    Ok(Some(handle))
}

pub async fn metrics_handler(handle: PrometheusHandle) -> impl IntoResponse {
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        handle.render(),
    )
}

pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    let method = request.method().as_str().to_owned();
    let route = matched_route(request.extensions().get::<MatchedPath>());
    let should_record = route != METRICS_ROUTE;

    let started_at = Instant::now();
    if should_record {
        gauge!(HTTP_REQUESTS_IN_FLIGHT, "method" => method.clone(), "route" => route.clone()).increment(1.0);
    }

    let response = next.run(request).await;
    if !should_record {
        return response;
    }

    let status = response.status().as_u16().to_string();
    let elapsed = started_at.elapsed().as_secs_f64();
    counter!(HTTP_REQUESTS_TOTAL, "method" => method.clone(), "route" => route.clone(), "status" => status.clone()).increment(1);
    histogram!(HTTP_REQUEST_DURATION_SECONDS, "method" => method.clone(), "route" => route.clone(), "status" => status.clone()).record(elapsed);
    gauge!(HTTP_REQUESTS_IN_FLIGHT, "method" => method, "route" => route).decrement(1.0);

    response
}

pub async fn db_query_metric<F, T, E>(component: &'static str, operation: &'static str, action: F) -> Result<T, E>
where
    F: Future<Output = Result<T, E>>,
{
    let started_at = Instant::now();
    let result = action.await;
    let status = if result.is_ok() { "ok" } else { "error" };

    counter!(DB_QUERIES_TOTAL, "component" => component, "operation" => operation, "status" => status).increment(1);
    histogram!(DB_QUERY_DURATION_SECONDS, "component" => component, "operation" => operation, "status" => status)
        .record(started_at.elapsed().as_secs_f64());

    result
}

fn matched_route(matched_path: Option<&MatchedPath>) -> String {
    matched_path.map(MatchedPath::as_str).unwrap_or(UNMATCHED_ROUTE).to_owned()
}
