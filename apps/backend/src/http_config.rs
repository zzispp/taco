use std::time::Duration;

use axum::http::{HeaderValue, Method, header};
use configuration::{Settings, ValidatedCorsList};
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
};

use crate::BackendResult;

pub fn cors_layer(settings: &Settings) -> BackendResult<CorsLayer> {
    let cors = settings.validated_cors()?;
    let mut layer = CorsLayer::new();
    layer = apply_origin_policy(layer, &cors.allowed_origins)?;
    layer = apply_method_policy(layer, &cors.allowed_methods)?;
    layer = apply_header_policy(layer, &cors.allowed_headers)?;
    layer = apply_exposed_header_policy(layer, &cors.exposed_headers)?;
    layer = layer.allow_credentials(cors.allow_credentials);

    if let Some(seconds) = cors.max_age_seconds {
        layer = layer.max_age(Duration::from_secs(seconds));
    }

    Ok(layer)
}

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

fn apply_origin_policy(layer: CorsLayer, policy: &ValidatedCorsList) -> BackendResult<CorsLayer> {
    match policy {
        ValidatedCorsList::Any => Ok(layer.allow_origin(Any)),
        ValidatedCorsList::Values(values) => Ok(layer.allow_origin(parse_header_values(values)?)),
    }
}

fn apply_method_policy(layer: CorsLayer, policy: &ValidatedCorsList) -> BackendResult<CorsLayer> {
    match policy {
        ValidatedCorsList::Any => Ok(layer.allow_methods(Any)),
        ValidatedCorsList::Values(values) => {
            let methods = values.iter().map(|value| Method::from_bytes(value.as_bytes())).collect::<Result<Vec<_>, _>>()?;
            Ok(layer.allow_methods(methods))
        }
    }
}

fn apply_header_policy(layer: CorsLayer, policy: &ValidatedCorsList) -> BackendResult<CorsLayer> {
    match policy {
        ValidatedCorsList::Any => Ok(layer.allow_headers(Any)),
        ValidatedCorsList::Values(values) => Ok(layer.allow_headers(parse_header_names(values)?)),
    }
}

fn apply_exposed_header_policy(layer: CorsLayer, policy: &ValidatedCorsList) -> BackendResult<CorsLayer> {
    match policy {
        ValidatedCorsList::Any => Ok(layer.expose_headers(Any)),
        ValidatedCorsList::Values(values) => Ok(layer.expose_headers(parse_header_names(values)?)),
    }
}

fn parse_header_values(values: &[String]) -> BackendResult<Vec<HeaderValue>> {
    values.iter().map(|value| HeaderValue::from_str(value).map_err(|error| error.into())).collect()
}

fn parse_header_names(values: &[String]) -> BackendResult<Vec<header::HeaderName>> {
    values
        .iter()
        .map(|value| header::HeaderName::from_bytes(value.as_bytes()).map_err(|error| error.into()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{cors_layer, parse_header_values};
    use axum::{
        Router,
        body::Body,
        http::{
            HeaderValue, Method, Request, Response, StatusCode,
            header::{
                self, ACCESS_CONTROL_ALLOW_CREDENTIALS, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
                ACCESS_CONTROL_EXPOSE_HEADERS,
            },
        },
        routing::get,
    };
    use configuration::{CorsSettings, Settings};
    use tower::{Layer, Service, ServiceExt, service_fn};

    #[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
    #[tokio::test]
    async fn cors_allows_wildcard_origin_for_normal_request() {
        let layer = cors_layer(&test_settings()).unwrap();
        let response = layer
            .layer(ok_service())
            .oneshot(
                Request::builder()
                    .uri("/api/test")
                    .header(header::ORIGIN, "http://localhost:8082")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN), Some(&HeaderValue::from_static("*")));
        assert_eq!(response.headers().get(ACCESS_CONTROL_EXPOSE_HEADERS), Some(&HeaderValue::from_static("*")));
        assert!(response.headers().get(ACCESS_CONTROL_ALLOW_CREDENTIALS).is_none());
    }

    #[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
    #[tokio::test]
    async fn cors_answers_preflight_with_wildcards() {
        let layer = cors_layer(&test_settings()).unwrap();
        let response = layer
            .layer(ok_service())
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/api/test")
                    .header(header::ORIGIN, "http://localhost:8082")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
                    .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "authorization,content-type")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN), Some(&HeaderValue::from_static("*")));
        assert_eq!(response.headers().get(ACCESS_CONTROL_ALLOW_METHODS), Some(&HeaderValue::from_static("*")));
        assert_eq!(response.headers().get(ACCESS_CONTROL_ALLOW_HEADERS), Some(&HeaderValue::from_static("*")));
        assert!(response.headers().get(ACCESS_CONTROL_EXPOSE_HEADERS).is_none());
    }

    #[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
    #[tokio::test]
    async fn cors_can_be_restricted_by_config() {
        let mut settings = test_settings();
        settings.cors = CorsSettings {
            allowed_origins: vec!["http://localhost:8082".into()],
            allowed_methods: vec!["GET".into()],
            allowed_headers: vec!["authorization".into()],
            exposed_headers: vec!["x-request-id".into()],
            allow_credentials: false,
            max_age_seconds: Some(600),
        };

        let response = cors_layer(&settings)
            .unwrap()
            .layer(ok_service())
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/api/test")
                    .header(header::ORIGIN, "http://localhost:8082")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "GET")
                    .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "authorization")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.headers().get(ACCESS_CONTROL_ALLOW_ORIGIN),
            Some(&HeaderValue::from_static("http://localhost:8082"))
        );
        assert_eq!(response.headers().get(ACCESS_CONTROL_ALLOW_METHODS), Some(&HeaderValue::from_static("GET")));
        assert_eq!(
            response.headers().get(ACCESS_CONTROL_ALLOW_HEADERS),
            Some(&HeaderValue::from_static("authorization"))
        );
        assert!(response.headers().get(ACCESS_CONTROL_EXPOSE_HEADERS).is_none());
    }

    #[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
    #[tokio::test]
    async fn cors_exposes_restricted_headers_on_actual_response() {
        let mut settings = test_settings();
        settings.cors = CorsSettings {
            allowed_origins: vec!["http://localhost:8082".into()],
            allowed_methods: vec!["GET".into()],
            allowed_headers: vec!["authorization".into()],
            exposed_headers: vec!["x-request-id".into()],
            allow_credentials: false,
            max_age_seconds: Some(600),
        };

        let response = cors_layer(&settings)
            .unwrap()
            .layer(ok_service())
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/api/test")
                    .header(header::ORIGIN, "http://localhost:8082")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(
            response.headers().get(ACCESS_CONTROL_EXPOSE_HEADERS),
            Some(&HeaderValue::from_static("x-request-id"))
        );
    }

    #[test]
    fn parse_header_values_accepts_valid_origins() {
        let values = parse_header_values(&["http://localhost:8082".into()]).unwrap();

        assert_eq!(values, vec![HeaderValue::from_static("http://localhost:8082")]);
    }

    #[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
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

    fn ok_service() -> impl Service<Request<Body>, Response = Response<Body>, Error = std::convert::Infallible> + Clone {
        service_fn(|_: Request<Body>| async { Ok::<_, std::convert::Infallible>(Response::new(Body::empty())) })
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
