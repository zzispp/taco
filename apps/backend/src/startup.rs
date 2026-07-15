use std::net::SocketAddr;

use configuration::Settings;
use tokio::net::TcpListener;

use crate::{BackendResult, composition};

pub async fn serve(settings: Settings) -> BackendResult<()> {
    let bind_addr = settings.bind_addr();
    hook_tracing::info_with_fields!("backend starting", addr = bind_addr);

    let metrics = hook_tracing::init_metrics(hook_tracing::MetricsConfig {
        enabled: settings.metrics.enabled,
    })?;
    let app = composition::build_router(&settings, metrics).await?;
    let listener = TcpListener::bind(&bind_addr).await?;

    hook_tracing::info_with_fields!("backend listening", addr = bind_addr);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}
