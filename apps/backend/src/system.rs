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
    system_log_ingestion: Option<SystemLogIngestionHealthResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ReadyResponse {
    status: &'static str,
}

#[derive(Debug, Serialize, ToSchema)]
struct SystemLogIngestionHealthResponse {
    delivery_guarantee: &'static str,
    queue_depth: usize,
    queue_capacity: usize,
    pending_events: u64,
    persisted_events: u64,
    dropped_events: u64,
    writer_running: bool,
    writer_healthy: bool,
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
    let system_log_ingestion = health_state.system_log_runtime.as_ref().map(system_log_ingestion_health);
    let ingestion_healthy = system_log_ingestion
        .as_ref()
        .is_none_or(|status| status.writer_running && status.writer_healthy);
    Json(HealthResponse {
        status: if listener.healthy && ingestion_healthy { "ok" } else { "degraded" },
        tracing_config_listener_healthy: listener.healthy,
        tracing_config_listener_failures: listener.failures,
        tracing_config_listener_last_failure: listener.last_failure,
        system_log_ingestion,
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

fn system_log_ingestion_health(runtime: &Arc<taco_tracing::SystemLogRuntime>) -> SystemLogIngestionHealthResponse {
    let status = runtime.status();
    let latest_write_failure = status.latest_write_failure.map(|failure| SystemLogWriteFailureResponse {
        failed_events: failure.failed_events,
        reason: failure.reason,
        occurred_at: failure
            .occurred_at
            .format(&time::format_description::well_known::Rfc3339)
            .expect("system log failure timestamp must format as RFC3339"),
    });
    SystemLogIngestionHealthResponse {
        delivery_guarantee: status.delivery_guarantee.as_str(),
        queue_depth: status.queue_depth,
        queue_capacity: status.queue_capacity,
        pending_events: status.pending_events,
        persisted_events: status.persisted_events,
        dropped_events: status.dropped_events,
        writer_running: status.writer_running,
        writer_healthy: status.writer_healthy,
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
        let ingestion = response.system_log_ingestion.expect("ingestion status must be present in production health");

        assert_eq!(response.status, "degraded");
        assert!(!ingestion.writer_running);
        assert!(!ingestion.writer_healthy);
        assert_eq!(ingestion.dropped_events, 1);
        assert_eq!(ingestion.latest_write_failure.as_ref().map(|failure| failure.reason), Some("connection"));
    }

    #[tokio::test]
    async fn health_exposes_current_best_effort_ingestion_state() {
        let runtime = Arc::new(taco_tracing::start_system_log_runtime(
            Arc::new(CollectingSink),
            taco_tracing::SystemLogLevel::Trace,
        ));
        runtime.emitter().emit(taco_tracing::SystemLogEvent::new(taco_tracing::SystemLogEventInput {
            occurred_at: time::OffsetDateTime::now_utc(),
            level: taco_tracing::SystemLogLevel::Info,
            target: "test::health".into(),
            message: "queued".into(),
            fields: Map::new(),
        }));

        let response = health(axum::extract::State(HealthState::new(
            crate::composition::tracing_runtime::TracingConfigListenerHealth::new(),
            runtime.clone(),
        )))
        .await
        .0;
        let serialized = serde_json::to_value(&response).unwrap();
        let ingestion = response.system_log_ingestion.expect("ingestion status must be present in production health");

        assert_eq!(response.status, "ok");
        assert_eq!(serialized["system_log_ingestion"]["delivery_guarantee"], "best_effort");
        assert_eq!(serialized.get("system_log_writer"), None);
        assert_eq!(ingestion.queue_depth, 1);
        assert_eq!(ingestion.queue_capacity, taco_tracing::SYSTEM_LOG_CHANNEL_CAPACITY);
        assert_eq!(ingestion.pending_events, 1);
        assert_eq!(ingestion.persisted_events, 0);
        assert_eq!(ingestion.dropped_events, 0);
        assert!(ingestion.writer_running);
        assert!(ingestion.writer_healthy);

        runtime.shutdown().await;
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

    struct CollectingSink;

    #[async_trait]
    impl taco_tracing::SystemLogSink for CollectingSink {
        async fn insert_batch(&self, _: Vec<taco_tracing::SystemLogEvent>) -> Result<(), String> {
            Ok(())
        }
    }
}
