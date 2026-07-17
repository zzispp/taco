use std::net::SocketAddr;

use configuration::Settings;
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
