use std::time::Duration;

use configuration::Settings;
use tower_http::{compression::CompressionLayer, timeout::TimeoutLayer};

use crate::BackendResult;

pub fn timeout_layer(settings: &Settings) -> BackendResult<TimeoutLayer> {
    let http = settings.http_config()?;
    Ok(TimeoutLayer::with_status_code(
        axum::http::StatusCode::REQUEST_TIMEOUT,
        Duration::from_millis(http.request_timeout_ms),
    ))
}

pub fn compression_layer(settings: &Settings) -> BackendResult<CompressionLayer> {
    let http = settings.http_config()?;
    let layer = if http.compression_enabled {
        CompressionLayer::new()
    } else {
        CompressionLayer::new().no_gzip()
    };
    Ok(layer)
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use tower::ServiceExt;

    use crate::composition::tests::test_settings;

    #[tokio::test]
    async fn timeout_layer_returns_request_timeout() {
        let mut settings = test_settings();
        settings.http.request_timeout_ms = 1;

        let app = Router::new()
            .route(
                "/slow",
                get(|| async {
                    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
                    StatusCode::OK
                }),
            )
            .layer(super::timeout_layer(&settings).unwrap());

        let response = app.oneshot(Request::builder().uri("/slow").body(Body::empty()).unwrap()).await.unwrap();

        assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    }
}
