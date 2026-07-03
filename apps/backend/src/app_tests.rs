#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use http_body_util::BodyExt;
    use tower::util::ServiceExt;

    use crate::composition;
    use configuration::Settings;

    #[tokio::test]
    async fn docs_and_metrics_routes_are_public() {
        let settings = test_settings();
        let metrics = hook_tracing::init_metrics(hook_tracing::MetricsConfig {
            enabled: settings.metrics.enabled,
        })
        .unwrap();
        let app = composition::build_public_router(&settings, metrics).unwrap();

        let openapi = app
            .clone()
            .oneshot(Request::builder().uri("/openapi.json").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(openapi.status(), StatusCode::OK);

        let docs = app.clone().oneshot(Request::builder().uri("/docs").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(docs.status(), StatusCode::OK);

        let metrics = app.oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(metrics.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn docs_support_gzip_and_request_id() {
        let settings = test_settings();
        let metrics = hook_tracing::init_metrics(hook_tracing::MetricsConfig {
            enabled: settings.metrics.enabled,
        })
        .unwrap();
        let app = composition::build_public_router(&settings, metrics).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/openapi.json")
                    .header(header::ACCEPT_ENCODING, "gzip")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get(header::CONTENT_ENCODING).unwrap(), "gzip");
        assert!(response.headers().contains_key("x-request-id"));
    }

    #[tokio::test]
    async fn metrics_output_exposes_http_series() {
        let settings = test_settings();
        let metrics = hook_tracing::init_metrics(hook_tracing::MetricsConfig {
            enabled: settings.metrics.enabled,
        })
        .unwrap();
        let app = composition::build_public_router(&settings, metrics).unwrap();

        let _ = app
            .clone()
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let response = app.oneshot(Request::builder().uri("/metrics").body(Body::empty()).unwrap()).await.unwrap();

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("http_requests_total"));
    }

    fn test_settings() -> Settings {
        let config_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../config/config.yaml")
            .canonicalize()
            .unwrap();

        Settings::load_from_args([
            std::ffi::OsString::from("backend"),
            std::ffi::OsString::from("--config"),
            config_path.into_os_string(),
        ])
        .unwrap()
    }
}
