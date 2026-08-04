use axum::{
    extract::{MatchedPath, Request, State},
    middleware::Next,
    response::Response,
};
use serde_json::{Map, Value};
use std::{
    fmt,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    HttpLogCaptureConfig, RuntimeTracingState,
    http_capture::{BodyCaptureOptions, SharedBodyCapture, body_value, content_type, query_parameters, request_headers, wrap_body},
};

const EXCLUDED_PREFIXES: &[&str] = &["/health", "/metrics", "/docs", "/openapi.json", "/uploads/"];

#[derive(Clone)]
pub struct HttpLogCaptureState {
    source: HttpLogCaptureConfigSource,
}

#[derive(Clone)]
enum HttpLogCaptureConfigSource {
    Fixed(Arc<HttpLogCaptureConfig>),
    Runtime(RuntimeTracingState),
}

impl HttpLogCaptureState {
    pub fn new(config: HttpLogCaptureConfig) -> Self {
        Self {
            source: HttpLogCaptureConfigSource::Fixed(Arc::new(config)),
        }
    }

    pub fn from_runtime_state(runtime: RuntimeTracingState) -> Self {
        Self {
            source: HttpLogCaptureConfigSource::Runtime(runtime),
        }
    }

    fn config(&self) -> HttpLogCaptureConfig {
        match &self.source {
            HttpLogCaptureConfigSource::Fixed(config) => config.as_ref().clone(),
            HttpLogCaptureConfigSource::Runtime(state) => state.current().http.clone(),
        }
    }

    #[cfg(test)]
    pub(crate) fn config_for_test(&self) -> HttpLogCaptureConfig {
        self.config()
    }
}

pub async fn http_log_middleware(State(state): State<HttpLogCaptureState>, request: Request, next: Next) -> Response {
    let config = state.config();
    if !config.access_enabled || excluded(request.uri().path()) {
        return next.run(request).await;
    }
    let context = AccessContext::new(&request, &config);
    let (request, request_capture) = capture_request(request, &config);
    let response = next.run(request).await;
    emit_response(ResponseCaptureInput {
        response,
        context,
        request_capture,
        config: &config,
    })
}

struct AccessContext {
    method: String,
    path: String,
    route: String,
    query: Option<Value>,
    headers: Option<Value>,
    started: Instant,
}

impl AccessContext {
    fn new(request: &Request, config: &HttpLogCaptureConfig) -> Self {
        Self {
            method: request.method().as_str().into(),
            path: request.uri().path().into(),
            route: request
                .extensions()
                .get::<MatchedPath>()
                .map(MatchedPath::as_str)
                .unwrap_or(request.uri().path())
                .into(),
            query: config.capture_query_parameters.then(|| query_parameters(request.uri())),
            headers: config.capture_request_headers.then(|| request_headers(request.headers())),
            started: Instant::now(),
        }
    }

    fn fields(&self, status: u16, request_body: Option<Value>) -> Result<Map<String, Value>, AccessLogBuildError> {
        let mut fields = Map::from_iter([
            ("event_type".into(), Value::String("http_access".into())),
            ("method".into(), Value::String(self.method.clone())),
            ("path".into(), Value::String(self.path.clone())),
            ("route".into(), Value::String(self.route.clone())),
            ("status".into(), Value::Number(status.into())),
            ("duration_ms".into(), Value::Number(duration_milliseconds(self.started.elapsed())?.into())),
        ]);
        insert_optional(&mut fields, "query_parameters", self.query.clone());
        insert_optional(&mut fields, "request_headers", self.headers.clone());
        insert_optional(&mut fields, "request_body", request_body);
        Ok(fields)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum AccessLogBuildError {
    DurationMillisecondsOverflow,
}

impl fmt::Display for AccessLogBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("HTTP access duration exceeds the supported millisecond range")
    }
}

enum RequestBodyCapture {
    Disabled,
    Enabled { content_type: Option<String>, capture: SharedBodyCapture },
}

fn capture_request(request: Request, config: &HttpLogCaptureConfig) -> (Request, RequestBodyCapture) {
    if !config.capture_request_body {
        return (request, RequestBodyCapture::Disabled);
    }
    let content_type = content_type(request.headers());
    let capture = SharedBodyCapture::new();
    let (parts, body) = request.into_parts();
    let body = wrap_body(
        body,
        capture.clone(),
        BodyCaptureOptions {
            limit: config.max_body_capture_bytes,
            on_complete: None,
        },
    );
    (Request::from_parts(parts, body), RequestBodyCapture::Enabled { content_type, capture })
}

struct ResponseCaptureInput<'a> {
    response: Response,
    context: AccessContext,
    request_capture: RequestBodyCapture,
    config: &'a HttpLogCaptureConfig,
}

fn emit_response(input: ResponseCaptureInput<'_>) -> Response {
    let ResponseCaptureInput {
        response,
        context,
        request_capture,
        config,
    } = input;
    let request_body = request_body_value(request_capture);
    let (parts, body) = response.into_parts();
    let status = parts.status.as_u16();
    if !config.capture_response_body {
        emit_access_event(context.fields(status, request_body));
        return Response::from_parts(parts, body);
    }
    let response_content_type = content_type(&parts.headers);
    let limit = config.max_body_capture_bytes;
    let callback = Box::new(move |capture| {
        let fields = context.fields(status, request_body).map(|mut fields| {
            fields.insert("response_body".into(), body_value(response_content_type.as_deref(), capture));
            fields
        });
        emit_access_event(fields);
    });
    Response::from_parts(
        parts,
        wrap_body(
            body,
            SharedBodyCapture::new(),
            BodyCaptureOptions {
                limit,
                on_complete: Some(callback),
            },
        ),
    )
}

fn request_body_value(capture: RequestBodyCapture) -> Option<Value> {
    match capture {
        RequestBodyCapture::Disabled => None,
        RequestBodyCapture::Enabled { content_type, capture } => Some(body_value(content_type.as_deref(), capture.snapshot())),
    }
}

fn emit_access_event(fields: Result<Map<String, Value>, AccessLogBuildError>) {
    let fields = match fields {
        Ok(fields) => fields,
        Err(error) => return tracing::error!(target: "taco.internal", error = %error, "HTTP access log construction failed"),
    };
    let Ok(fields_json) = serde_json::to_string(&fields) else {
        return tracing::error!(target: "taco.internal", "HTTP access log serialization failed");
    };
    tracing::info!(target: module_path!(), __taco_system_log = true, message = "HTTP access", fields_json = %fields_json);
}

fn duration_milliseconds(duration: Duration) -> Result<u64, AccessLogBuildError> {
    u64::try_from(duration.as_millis()).map_err(|_| AccessLogBuildError::DurationMillisecondsOverflow)
}

fn insert_optional(fields: &mut Map<String, Value>, key: &'static str, value: Option<Value>) {
    if let Some(value) = value {
        fields.insert(key.into(), value);
    }
}

fn excluded(path: &str) -> bool {
    EXCLUDED_PREFIXES.iter().any(|prefix| path == *prefix || path.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::{Body, to_bytes},
        http::{Request, StatusCode},
        middleware,
        response::IntoResponse,
        routing::post,
    };
    use serde_json::json;
    use tower::ServiceExt;
    use tracing_subscriber::{Registry, layer::SubscriberExt};

    use crate::{HttpLogCaptureConfig, SystemLogLayer, SystemLogLevel, start_system_log_runtime};

    use super::{AccessLogBuildError, HttpLogCaptureState, duration_milliseconds, excluded, http_log_middleware};

    #[tokio::test(flavor = "current_thread")]
    async fn middleware_captures_selected_http_data_without_changing_the_body() {
        let sink = std::sync::Arc::new(CollectingSink::default());
        let runtime = start_system_log_runtime(sink.clone(), SystemLogLevel::Info);
        let subscriber = Registry::default().with(SystemLogLayer::new(runtime.emitter()));
        let config = HttpLogCaptureConfig {
            access_enabled: true,
            capture_request_body: true,
            capture_response_body: true,
            capture_query_parameters: true,
            capture_request_headers: true,
            max_body_capture_bytes: 1024,
        };
        let app = Router::new()
            .route("/api/test", post(echo))
            .layer(middleware::from_fn_with_state(HttpLogCaptureState::new(config), http_log_middleware));

        let guard = tracing::subscriber::set_default(subscriber);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/test?token=raw&page=1")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer raw")
                    .body(Body::from(r#"{"password":"raw"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body).unwrap(),
            json!({"ok": true, "access_token": "raw"})
        );
        wait_for_events(&sink).await;

        let fields = &sink.events()[0].fields;
        assert_eq!(fields["query_parameters"]["token"], kernel::redaction::REDACTED);
        assert_eq!(fields["request_headers"]["authorization"], kernel::redaction::REDACTED);
        assert_eq!(fields["request_body"]["content"]["password"], kernel::redaction::REDACTED);
        assert_eq!(fields["response_body"]["content"]["access_token"], kernel::redaction::REDACTED);
        drop(guard);
    }

    #[test]
    fn excluded_routes_do_not_produce_access_events() {
        for path in ["/health", "/metrics", "/docs", "/docs/", "/openapi.json", "/uploads/avatars/a.png"] {
            assert!(excluded(path), "{path} should be excluded");
        }
        assert!(!excluded("/api/system/system-logs"));
    }

    #[test]
    fn rejects_an_access_duration_outside_the_wire_range() {
        let error = duration_milliseconds(std::time::Duration::from_secs(u64::MAX)).unwrap_err();

        assert_eq!(error, AccessLogBuildError::DurationMillisecondsOverflow);
    }

    async fn echo(body: String) -> impl IntoResponse {
        assert_eq!(body, r#"{"password":"raw"}"#);
        (
            StatusCode::OK,
            [("content-type", "application/json")],
            json!({"ok": true, "access_token": "raw"}).to_string(),
        )
    }

    #[derive(Default)]
    struct CollectingSink(std::sync::Mutex<Vec<crate::SystemLogEvent>>);

    impl CollectingSink {
        fn events(&self) -> Vec<crate::SystemLogEvent> {
            self.0.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl crate::SystemLogSink for CollectingSink {
        async fn insert_batch(&self, events: Vec<crate::SystemLogEvent>) -> Result<(), String> {
            self.0.lock().unwrap().extend(events);
            Ok(())
        }
    }

    async fn wait_for_events(sink: &CollectingSink) {
        tokio::time::timeout(std::time::Duration::from_secs(1), async {
            while sink.events().is_empty() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("HTTP event was not persisted");
    }
}
