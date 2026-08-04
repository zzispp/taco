use std::{future::Future, net::SocketAddr};

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
    let shutdown = shutdown_signal()?;

    taco_tracing::info_with_fields!("backend listening", addr = bind_addr);
    let result = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown)
        .await;
    system_logs.shutdown().await;
    result?;
    Ok(())
}

#[cfg(unix)]
fn shutdown_signal() -> std::io::Result<impl Future<Output = ()>> {
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    Ok(async move {
        tokio::select! {
            result = tokio::signal::ctrl_c() => record_signal_error(result),
            _ = terminate.recv() => {}
        }
    })
}

#[cfg(not(unix))]
fn shutdown_signal() -> std::io::Result<impl Future<Output = ()>> {
    Ok(async {
        record_signal_error(tokio::signal::ctrl_c().await);
    })
}

fn record_signal_error(result: std::io::Result<()>) {
    if let Err(error) = result {
        taco_tracing::error_with_fields!("backend shutdown signal handler failed", &error, component = "shutdown_signal");
    }
}
