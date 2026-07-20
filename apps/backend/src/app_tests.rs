#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header},
    };
    use http_body_util::BodyExt;
    use tower::util::ServiceExt;

    use crate::{composition, composition::tests::test_settings};

    #[tokio::test]
    async fn docs_and_metrics_routes_are_public() {
        let settings = test_settings();
        let metrics = taco_tracing::init_metrics(taco_tracing::MetricsConfig {
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
        let metrics = taco_tracing::init_metrics(taco_tracing::MetricsConfig {
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
        let metrics = taco_tracing::init_metrics(taco_tracing::MetricsConfig {
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

    #[tokio::test]
    async fn normal_public_routes_report_installed_and_ready() {
        let settings = test_settings();
        let metrics = None;
        let app = composition::build_public_router(&settings, metrics).unwrap();

        let status = app
            .clone()
            .oneshot(Request::builder().uri("/api/setup/status").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let ready = app.oneshot(Request::builder().uri("/ready").body(Body::empty()).unwrap()).await.unwrap();

        assert_eq!(status.status(), StatusCode::OK);
        assert_eq!(ready.status(), StatusCode::OK);
    }

    #[cfg(feature = "embedded-frontend")]
    #[tokio::test]
    async fn normal_public_routes_serve_embedded_frontend_without_serving_unknown_api_paths() {
        let app = composition::build_public_router(&test_settings(), None).unwrap();

        let document = app.clone().oneshot(Request::builder().uri("/").body(Body::empty()).unwrap()).await.unwrap();
        let api = app.oneshot(Request::builder().uri("/api/missing").body(Body::empty()).unwrap()).await.unwrap();

        assert_eq!(document.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(document.headers()[header::LOCATION], "/cn/");
        assert_eq!(document.headers()[header::CACHE_CONTROL], "no-cache");
        assert_eq!(api.status(), StatusCode::NOT_FOUND);
        assert_eq!(api.headers()[header::CACHE_CONTROL], "no-store");
        assert!(!api.headers().contains_key(header::CONTENT_TYPE));
    }
}
