use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use tower::ServiceExt;

use crate::{
    application::{InstallationCompleted, SetupError, SetupInstallationInput, SetupUseCase},
    domain::{PostgresConnection, RedisConnection},
};

use super::{
    SETUP_DEFAULTS_PATH, SETUP_INSTALL_PATH, SETUP_POSTGRES_TEST_PATH, SETUP_REDIS_TEST_PATH, SETUP_STATUS_PATH, installed_router, setup_router,
    setup_router_with_state,
};

#[tokio::test]
async fn setup_status_is_public_and_uncached_without_any_setup_token() {
    assert_status(setup_router(), SETUP_STATUS_PATH, StatusCode::OK, json!({"state": "setup"})).await;
}

#[tokio::test]
async fn installed_router_retains_only_public_status() {
    assert_status(installed_router(), SETUP_STATUS_PATH, StatusCode::OK, json!({"state": "installed"})).await;
    for path in [SETUP_DEFAULTS_PATH, SETUP_POSTGRES_TEST_PATH, SETUP_REDIS_TEST_PATH, SETUP_INSTALL_PATH] {
        let response = installed_router().oneshot(Request::post(path).body(Body::empty()).unwrap()).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn installed_setup_paths_return_not_found_before_a_nested_api_fallback() {
    let nested_api = Router::new().fallback(|| async { StatusCode::UNAUTHORIZED });
    let app = installed_router().nest("/api", nested_api);

    for path in [SETUP_DEFAULTS_PATH, SETUP_POSTGRES_TEST_PATH, SETUP_REDIS_TEST_PATH, SETUP_INSTALL_PATH] {
        let response = app.clone().oneshot(Request::post(path).body(Body::empty()).unwrap()).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}

#[tokio::test]
async fn configured_setup_router_exposes_connection_tests_without_authentication() {
    let setup = Arc::new(RecordingSetup::default());
    let response = setup_router_with_state(setup.clone())
        .oneshot(json_request(SETUP_POSTGRES_TEST_PATH, postgres_payload()))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(response_json(response).await, json!({"status": "ok"}));
    assert_eq!(setup.calls(), vec![SetupCall::Postgres { port: 5_432, use_tls: true }]);
}

#[tokio::test]
async fn configured_setup_router_exposes_backend_owned_setup_defaults() {
    let response = setup_router_with_state(Arc::new(RecordingSetup::default()))
        .oneshot(Request::get(SETUP_DEFAULTS_PATH).body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    let body = response_json(response).await;
    assert_eq!(body["postgres"], json!({"port": 5432, "use_tls": true}));
    assert_eq!(body["redis"], json!({"port": 6379, "use_tls": true}));
    assert_eq!(body["advanced"]["http_request_timeout_ms"], json!(30000));
    assert_eq!(body["advanced"]["redis_key_prefix"], json!("taco:"));
}

#[tokio::test]
async fn redis_test_keeps_an_explicit_database_zero() {
    let setup = Arc::new(RecordingSetup::default());
    let payload = json!({
        "host": "redis.internal",
        "database": 0,
    });
    let response = setup_router_with_state(setup.clone())
        .oneshot(json_request(SETUP_REDIS_TEST_PATH, payload))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        setup.calls(),
        vec![SetupCall::Redis {
            port: 6_379,
            database: Some(0),
            use_tls: true
        }]
    );
}

#[tokio::test]
async fn installation_submission_returns_restart_contract_without_disclosing_secrets() {
    let setup = Arc::new(RecordingSetup::default());
    let response = setup_router_with_state(setup.clone())
        .oneshot(json_request(SETUP_INSTALL_PATH, installation_payload()))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response_json(response).await, json!({"state": "installed", "restart_required": true}));
    assert_eq!(setup.calls(), vec![SetupCall::Install]);
}

#[tokio::test]
async fn installation_submission_rejects_removed_owner_replacement_controls() {
    let setup = Arc::new(RecordingSetup::default());
    let mut payload = installation_payload();
    payload["replace_installation_owner"] = json!(true);
    let response = setup_router_with_state(setup.clone())
        .oneshot(json_request(SETUP_INSTALL_PATH, payload))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert!(setup.calls().is_empty());
}

#[tokio::test]
async fn invalid_structured_connection_input_has_localized_api_error_shape() {
    let payload = json!({
        "host": "postgres.internal",
        "port": 0,
        "username": "taco",
        "password": "secret",
        "database": "taco",
    });
    let response = setup_router_with_state(Arc::new(RecordingSetup::default()))
        .oneshot(json_request(SETUP_POSTGRES_TEST_PATH, payload))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(
        response_json(response).await,
        json!({"code": "invalid_input", "message": "参数错误", "details": "postgres.port必须大于零"})
    );
}

#[tokio::test]
async fn failed_connection_test_does_not_expose_infrastructure_error_text() {
    let setup = Arc::new(RecordingSetup::failing(SetupError::PostgresConnectionFailed));
    let response = setup_router_with_state(setup)
        .oneshot(json_request(SETUP_POSTGRES_TEST_PATH, postgres_payload()))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        response_json(response).await,
        json!({"code": "infrastructure_error", "message": "服务异常", "details": "PostgreSQL 连接测试失败"})
    );
}

#[tokio::test]
async fn invalid_initial_owner_has_a_stable_validation_response() {
    let setup = Arc::new(RecordingSetup::failing(SetupError::InstallationOwnerInvalid));
    let response = setup_router_with_state(setup)
        .oneshot(json_request(SETUP_INSTALL_PATH, installation_payload()))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response_json(response).await,
        json!({"code": "invalid_input", "message": "参数错误", "details": "初始管理员信息无效"})
    );
}

#[tokio::test]
async fn failed_data_reset_has_a_localized_infrastructure_error() {
    let setup = Arc::new(RecordingSetup::failing(SetupError::InstallationDataResetFailed));
    let response = setup_router_with_state(setup)
        .oneshot(json_request(SETUP_INSTALL_PATH, installation_payload()))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        response_json(response).await,
        json!({"code": "infrastructure_error", "message": "服务异常", "details": "安装数据重置失败"})
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SetupCall {
    Postgres { port: u16, use_tls: bool },
    Redis { port: u16, database: Option<u16>, use_tls: bool },
    Install,
}

struct RecordingSetup {
    failure: Option<SetupError>,
    calls: Mutex<Vec<SetupCall>>,
}

impl RecordingSetup {
    fn failing(error: SetupError) -> Self {
        Self {
            failure: Some(error),
            calls: Mutex::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<SetupCall> {
        self.calls.lock().unwrap().clone()
    }

    fn result(&self) -> Result<(), SetupError> {
        self.failure.clone().map_or(Ok(()), Err)
    }
}

impl Default for RecordingSetup {
    fn default() -> Self {
        Self {
            failure: None,
            calls: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl SetupUseCase for RecordingSetup {
    async fn test_postgres(&self, connection: PostgresConnection) -> Result<(), SetupError> {
        self.calls.lock().unwrap().push(SetupCall::Postgres {
            port: connection.port(),
            use_tls: connection.use_tls(),
        });
        self.result()
    }

    async fn test_redis(&self, connection: RedisConnection) -> Result<(), SetupError> {
        self.calls.lock().unwrap().push(SetupCall::Redis {
            port: connection.port(),
            database: connection.database(),
            use_tls: connection.use_tls(),
        });
        self.result()
    }

    async fn install(&self, _: SetupInstallationInput) -> Result<InstallationCompleted, SetupError> {
        self.calls.lock().unwrap().push(SetupCall::Install);
        self.result().map(|()| InstallationCompleted)
    }
}

async fn assert_status(app: Router, path: &str, status: StatusCode, expected_body: Value) {
    let response = app.oneshot(Request::get(path).body(Body::empty()).unwrap()).await.unwrap();

    assert_eq!(response.status(), status);
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
    assert_eq!(response_json(response).await, expected_body);
}

fn json_request(path: &str, payload: Value) -> Request<Body> {
    Request::post(path)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap()
}

async fn response_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn postgres_payload() -> Value {
    json!({
        "host": "postgres.internal",
        "username": "taco",
        "password": "postgres-secret",
        "database": "taco",
    })
}

fn installation_payload() -> Value {
    json!({
        "postgres": postgres_payload(),
        "redis": {"host": "redis.internal"},
        "administrator": {
            "username": "owner",
            "email": "owner@example.test",
            "password": "owner-secret",
        },
    })
}
