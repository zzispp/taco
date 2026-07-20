use std::{net::SocketAddr, sync::Arc};

use axum::{Json, Router, http::StatusCode, middleware, routing::get};
use configuration::{BootstrapInputs, Settings};
use installation::application::SetupUseCase;
use serde::Serialize;
use tokio::net::TcpListener;

use crate::{BackendResult, composition};

pub async fn serve(settings: Settings) -> BackendResult<()> {
    let bind_addr = settings.bind_addr();
    let metrics = taco_tracing::init_metrics(taco_tracing::MetricsConfig {
        enabled: settings.metrics.enabled,
    })?;
    let state = composition::build_app_state(&settings).await?;
    let system_logs = state.system_log_runtime.clone();
    let app = composition::create_app(state, &settings, metrics)?;
    taco_tracing::info_with_fields!("backend starting", addr = bind_addr);
    let listener = TcpListener::bind(&bind_addr).await?;

    taco_tracing::info_with_fields!("backend listening", addr = bind_addr);
    let result = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await;
    system_logs.shutdown().await;
    result?;
    Ok(())
}

pub async fn serve_setup(bootstrap: BootstrapInputs) -> BackendResult<()> {
    let bind_addr = bootstrap.listen_addr;
    let shutdown = composition::setup_wiring::SetupShutdown::new();
    let setup = composition::setup_wiring::build_setup_use_case(&bootstrap, shutdown.clone());
    let app = setup_app(setup);
    taco_tracing::info_with_fields!("setup backend starting", addr = bind_addr);
    let listener = TcpListener::bind(bind_addr).await?;

    taco_tracing::info_with_fields!("setup backend listening", addr = bind_addr);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(setup_shutdown_signal(shutdown))
        .await?;
    Ok(())
}

fn setup_app(setup: Arc<dyn SetupUseCase>) -> Router {
    let app = Router::new()
        .route("/health", get(setup_health))
        .route("/ready", get(setup_ready))
        .merge(installation::api::setup_router_with_state(setup));
    let app = crate::embedded_frontend::with_embedded_frontend(app).layer(middleware::from_fn(types::http::locale_middleware));
    composition::http_pipeline::apply_setup_layers(app)
}

#[derive(Serialize)]
struct SetupStatusResponse {
    status: &'static str,
}

async fn setup_health() -> Json<SetupStatusResponse> {
    Json(SetupStatusResponse { status: "setup" })
}

async fn setup_ready() -> (StatusCode, Json<SetupStatusResponse>) {
    (StatusCode::SERVICE_UNAVAILABLE, Json(SetupStatusResponse { status: "setup" }))
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("SIGTERM handler must initialize");
        tokio::select! {
            result = tokio::signal::ctrl_c() => result.expect("Ctrl-C handler must initialize"),
            _ = terminate.recv() => {}
        }
    }
    #[cfg(not(unix))]
    tokio::signal::ctrl_c().await.expect("Ctrl-C handler must initialize");
}

async fn setup_shutdown_signal(shutdown: composition::setup_wiring::SetupShutdown) {
    tokio::select! {
        _ = shutdown.wait_for_request() => {}
        _ = shutdown_signal() => {}
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{HeaderValue, Request, StatusCode, header},
    };
    use tower::ServiceExt;

    use async_trait::async_trait;
    use installation::{
        application::{InstallationCompleted, SetupError, SetupInstallationInput, SetupUseCase},
        domain::{PostgresConnection, RedisConnection},
    };

    use super::setup_app;

    #[tokio::test]
    async fn setup_mode_is_live_but_not_ready() {
        let app = setup_app(test_setup());
        let health = app.clone().oneshot(Request::get("/health").body(Body::empty()).unwrap()).await.unwrap();
        let ready = app.oneshot(Request::get("/ready").body(Body::empty()).unwrap()).await.unwrap();

        assert_eq!(health.status(), StatusCode::OK);
        assert_eq!(to_bytes(health.into_body(), usize::MAX).await.unwrap().as_ref(), br#"{"status":"setup"}"#);
        assert_eq!(ready.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(to_bytes(ready.into_body(), usize::MAX).await.unwrap().as_ref(), br#"{"status":"setup"}"#);
    }

    #[tokio::test]
    async fn setup_mode_exposes_the_public_setup_status() {
        let response = setup_app(test_setup())
            .oneshot(Request::get(installation::api::SETUP_STATUS_PATH).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(to_bytes(response.into_body(), usize::MAX).await.unwrap().as_ref(), br#"{"state":"setup"}"#);
    }

    #[tokio::test]
    async fn setup_mode_exposes_backend_owned_setup_defaults() {
        let response = setup_app(test_setup())
            .oneshot(Request::get(installation::api::SETUP_DEFAULTS_PATH).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[axum::http::header::CACHE_CONTROL], "no-store");
    }

    #[tokio::test]
    async fn setup_mode_applies_normal_response_security_and_request_id_layers() {
        let response = setup_app(test_setup())
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .header("x-request-id", "setup-request-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.headers().get("x-request-id"), Some(&HeaderValue::from_static("setup-request-1")));
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

    #[cfg(feature = "embedded-frontend")]
    #[tokio::test]
    async fn setup_mode_serves_the_embedded_frontend_without_claiming_unknown_api_paths() {
        let app = setup_app(test_setup());
        let document = app.clone().oneshot(Request::get("/").body(Body::empty()).unwrap()).await.unwrap();
        let api = app.oneshot(Request::get("/api/missing").body(Body::empty()).unwrap()).await.unwrap();

        assert_eq!(document.status(), StatusCode::OK);
        assert_eq!(document.headers()[axum::http::header::CONTENT_TYPE], "text/html; charset=utf-8");
        assert_eq!(api.status(), StatusCode::NOT_FOUND);
        assert!(!api.headers().contains_key(axum::http::header::CONTENT_TYPE));
    }

    fn test_setup() -> std::sync::Arc<dyn SetupUseCase> {
        std::sync::Arc::new(TestSetup)
    }

    struct TestSetup;

    #[async_trait]
    impl SetupUseCase for TestSetup {
        async fn test_postgres(&self, _: PostgresConnection) -> Result<(), SetupError> {
            Ok(())
        }

        async fn test_redis(&self, _: RedisConnection) -> Result<(), SetupError> {
            Ok(())
        }

        async fn install(&self, _: SetupInstallationInput) -> Result<InstallationCompleted, SetupError> {
            Ok(InstallationCompleted)
        }
    }
}
