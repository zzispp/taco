use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;
use utoipa::ToSchema;

use crate::composition::tracing_runtime::TracingConfigListenerHealth;

#[derive(Clone)]
pub(crate) struct HealthState {
    tracing_config_listener: TracingConfigListenerHealth,
    system_log_runtime: Option<Arc<taco_tracing::SystemLogRuntime>>,
}

impl HealthState {
    pub(crate) fn new(tracing_config_listener: TracingConfigListenerHealth, system_log_runtime: Arc<taco_tracing::SystemLogRuntime>) -> Self {
        Self {
            tracing_config_listener,
            system_log_runtime: Some(system_log_runtime),
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(tracing_config_listener: TracingConfigListenerHealth) -> Self {
        Self {
            tracing_config_listener,
            system_log_runtime: None,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    status: &'static str,
    tracing_config_listener_healthy: bool,
    tracing_config_listener_failures: u64,
    tracing_config_listener_last_failure: Option<String>,
    system_log_writer: Option<SystemLogWriterHealthResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReadyResponse {
    status: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
struct SystemLogWriterHealthResponse {
    healthy: bool,
    dropped_events: u64,
    latest_write_failure: Option<SystemLogWriteFailureResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
struct SystemLogWriteFailureResponse {
    failed_events: u64,
    reason: &'static str,
    occurred_at: String,
}

pub fn create_router(health_state: HealthState) -> Router {
    Router::new().route("/health", get(health)).route("/ready", get(ready)).with_state(health_state)
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "system",
    responses((status = OK, description = "Health check", body = HealthResponse))
)]
pub async fn health(State(health_state): State<HealthState>) -> Json<HealthResponse> {
    let listener = health_state.tracing_config_listener.snapshot();
    let system_log_writer = health_state.system_log_runtime.as_ref().map(system_log_writer_health);
    let writer_healthy = system_log_writer.as_ref().is_none_or(|status| status.healthy);
    Json(HealthResponse {
        status: if listener.healthy && writer_healthy { "ok" } else { "degraded" },
        tracing_config_listener_healthy: listener.healthy,
        tracing_config_listener_failures: listener.failures,
        tracing_config_listener_last_failure: listener.last_failure,
        system_log_writer,
    })
}

#[utoipa::path(
    get,
    path = "/ready",
    tag = "system",
    responses((status = OK, description = "Runtime dependencies are ready", body = ReadyResponse))
)]
pub async fn ready() -> (StatusCode, Json<ReadyResponse>) {
    (StatusCode::OK, Json(ReadyResponse { status: "ready" }))
}

fn system_log_writer_health(runtime: &Arc<taco_tracing::SystemLogRuntime>) -> SystemLogWriterHealthResponse {
    let status = runtime.status();
    let latest_write_failure = status.latest_write_failure.map(|failure| SystemLogWriteFailureResponse {
        failed_events: failure.failed_events,
        reason: failure.reason,
        occurred_at: failure
            .occurred_at
            .format(&time::format_description::well_known::Rfc3339)
            .expect("system log failure timestamp must format as RFC3339"),
    });
    SystemLogWriterHealthResponse {
        healthy: status.writer_healthy,
        dropped_events: status.dropped_events,
        latest_write_failure,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use serde_json::Map;

    use super::{HealthState, health, ready};

    #[tokio::test]
    async fn health_exposes_safe_writer_failure_state() {
        let runtime = Arc::new(taco_tracing::start_system_log_runtime(
            Arc::new(FailingSink),
            taco_tracing::SystemLogLevel::Trace,
        ));
        runtime.emitter().emit(taco_tracing::SystemLogEvent::new(taco_tracing::SystemLogEventInput {
            occurred_at: time::OffsetDateTime::now_utc(),
            level: taco_tracing::SystemLogLevel::Error,
            target: "test::health".into(),
            message: "failed".into(),
            fields: Map::new(),
        }));
        runtime.shutdown().await;

        let response = health(axum::extract::State(HealthState::new(
            crate::composition::tracing_runtime::TracingConfigListenerHealth::new(),
            runtime,
        )))
        .await
        .0;
        let writer = response.system_log_writer.expect("writer status must be present in production health");

        assert_eq!(response.status, "degraded");
        assert!(!writer.healthy);
        assert_eq!(writer.latest_write_failure.as_ref().map(|failure| failure.reason), Some("connection"));
    }

    #[tokio::test]
    async fn normal_runtime_reports_ready() {
        let (status, response) = ready().await;

        assert_eq!(status, axum::http::StatusCode::OK);
        assert_eq!(response.0.status, "ready");
    }

    struct FailingSink;

    #[async_trait]
    impl taco_tracing::SystemLogSink for FailingSink {
        async fn insert_batch(&self, _: Vec<taco_tracing::SystemLogEvent>) -> Result<(), String> {
            Err("connection unavailable".into())
        }
    }
}
