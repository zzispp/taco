use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, Response, StatusCode, header},
};
use captcha::application::{CaptchaConfigResponse, CaptchaResult, CaptchaUseCase};
use rbac::application::{ApiCheckRequest, AuthorizationConfig, RbacError, RbacResult, RbacUseCase};
use serde_json::{Value, json};
use tower::ServiceExt;
use types::rbac::{DataScopeFilter, NavResponse};

use super::create_router;
use crate::{
    api::{ApiState, TokenService, TokenSettings},
    application::{AppResult, SystemConfigProvider, UserService},
    test_support::{MemoryUserRepository, TestPasswordHasher, VALID_PASSWORD, stored_user},
};

const TEST_SECRET: &str = "test-secret-with-enough-entropy";
const ACCESS_TTL_SECONDS: u64 = 900;
const REFRESH_TTL_SECONDS: u64 = 604800;
const DEFAULT_INIT_PASSWORD: &str = "12345678";
const VALID_CAPTCHA_TOKEN: &str = "valid-captcha-token";

#[tokio::test]
async fn sign_in_accepts_email_identifier_and_returns_token_pair() {
    let app = test_router();

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-in",
            json!({
                "identifier": "alice@example.com",
                "password": VALID_PASSWORD
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["user"]["username"], "alice");
    assert_non_empty_string(&body["access_token"]);
    assert_non_empty_string(&body["refresh_token"]);
}

#[tokio::test]
async fn sign_up_accepts_public_payload_and_sets_backend_fields() {
    let app = test_router();

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-up",
            json!({
                "username": "bob",
                "email": "bob@example.com",
                "password": VALID_PASSWORD
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["user"]["role_ids"], json!(["2"]));
    assert_eq!(body["user"]["status"], "0");
    assert_eq!(body["user"]["is_active"], true);
    assert_eq!(body["user"]["auth_source"], "local");
    assert_eq!(body["user"]["email_verified"], false);
    assert_non_empty_string(&body["access_token"]);
}

#[tokio::test]
async fn sign_up_rejects_when_registration_is_disabled() {
    let app = test_router_with_config(TestConfig::new(false));

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-up",
            json!({
                "username": "bob",
                "email": "bob@example.com",
                "password": VALID_PASSWORD
            }),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let body = json_body(response).await;

    assert_eq!(body["code"], "forbidden");
}

#[tokio::test]
async fn sign_in_rejects_missing_captcha_when_enabled() {
    let app = test_router_with_captcha(TestCaptcha::enabled());

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-in",
            json!({
                "identifier": "alice@example.com",
                "password": VALID_PASSWORD
            }),
        ))
        .await
        .unwrap();
    let body = json_body(response).await;

    assert_eq!(body["code"], "invalid_input");
    assert_eq!(body["details"], "captcha verification is required");
}

#[tokio::test]
async fn sign_in_accepts_captcha_token_when_enabled() {
    let app = test_router_with_captcha(TestCaptcha::enabled());

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-in",
            json!({
                "identifier": "alice@example.com",
                "password": VALID_PASSWORD,
                "captcha_token": VALID_CAPTCHA_TOKEN
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["user"]["username"], "alice");
    assert_non_empty_string(&body["access_token"]);
    assert_non_empty_string(&body["refresh_token"]);
}

#[tokio::test]
async fn sign_up_rejects_missing_captcha_when_enabled() {
    let app = test_router_with_captcha(TestCaptcha::enabled());

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-up",
            json!({
                "username": "bob",
                "email": "bob@example.com",
                "password": VALID_PASSWORD
            }),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;
    assert_eq!(body["code"], "invalid_input");
    assert_eq!(body["details"], "captcha verification is required");
}

#[tokio::test]
async fn sign_up_accepts_captcha_token_when_enabled() {
    let app = test_router_with_captcha(TestCaptcha::enabled());

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-up",
            json!({
                "username": "bob",
                "email": "bob@example.com",
                "password": VALID_PASSWORD,
                "captcha_token": VALID_CAPTCHA_TOKEN
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["user"]["username"], "bob");
    assert_non_empty_string(&body["access_token"]);
}

#[tokio::test]
async fn create_user_uses_default_password_when_payload_password_is_empty() {
    let repository = base_repository();
    let app = test_router_with_repository(repository.clone(), TestConfig::new(true));

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/system/users",
            json!({
                "username": "charlie",
                "password": "",
                "nick_name": "Charlie",
                "dept_id": null,
                "email": "charlie@example.com",
                "phonenumber": null,
                "sex": "2",
                "status": "0",
                "remark": null,
                "role_ids": ["2"],
                "post_ids": []
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["username"], "charlie");
    assert_eq!(repository.created_records()[0].password_hash.as_deref(), Some("hashed:12345678"));
}

#[tokio::test]
async fn me_returns_user_for_bearer_access_token() {
    let app = test_router();
    let tokens = sign_in(app.clone()).await;

    let response = app
        .oneshot(authenticated_request(Method::GET, "/api/auth/me", &tokens.access_token))
        .await
        .unwrap();
    let body = response_json(response).await;

    assert_eq!(body["user"]["email"], "alice@example.com");
}

#[tokio::test]
async fn refresh_returns_new_token_pair_and_me_accepts_new_access_token() {
    let app = test_router();
    let tokens = sign_in(app.clone()).await;

    let response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/api/auth/refresh",
            json!({ "refresh_token": tokens.refresh_token }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    let access_token = body["access_token"].as_str().unwrap();
    assert_non_empty_string(&body["refresh_token"]);

    let response = app.oneshot(authenticated_request(Method::GET, "/api/auth/me", access_token)).await.unwrap();
    let body = response_json(response).await;

    assert_eq!(body["user"]["username"], "alice");
}

#[tokio::test]
async fn refresh_rejects_access_token() {
    let app = test_router();
    let tokens = sign_in(app.clone()).await;

    let response = app
        .oneshot(json_request(Method::POST, "/api/auth/refresh", json!({ "refresh_token": tokens.access_token })))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = json_body(response).await;

    assert_eq!(body["code"], "unauthorized");
}

#[tokio::test]
async fn sign_in_rejects_malformed_json_with_uniform_error_shape() {
    let app = test_router();

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/auth/sign-in")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"identifier":"alice","password":"secret""#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = json_body(response).await;

    assert_eq!(body["code"], "invalid_json");
    assert_eq!(body["message"], "invalid JSON payload");
    assert!(body["details"].as_str().is_some());
}

struct SessionTokens {
    access_token: String,
    refresh_token: String,
}

fn test_router() -> Router {
    test_router_with_config(TestConfig::new(true))
}

fn test_router_with_config(config: TestConfig) -> Router {
    test_router_with_repository(base_repository(), config)
}

fn test_router_with_captcha(captcha: TestCaptcha) -> Router {
    test_router_with_repository_and_captcha(base_repository(), TestConfig::new(true), captcha)
}

fn test_router_with_repository(repository: MemoryUserRepository, config: TestConfig) -> Router {
    test_router_with_repository_and_captcha(repository, config, TestCaptcha::disabled())
}

fn test_router_with_repository_and_captcha(repository: MemoryUserRepository, config: TestConfig, captcha: TestCaptcha) -> Router {
    let users = UserService::new(repository, TestPasswordHasher);
    let state = ApiState::new(Arc::new(users), token_service(), Arc::new(UnusedRbac), Arc::new(config), Arc::new(captcha));
    Router::new().nest("/api", create_router(state))
}

fn base_repository() -> MemoryUserRepository {
    MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"))
}

struct TestCaptcha {
    enabled: bool,
}

impl TestCaptcha {
    fn enabled() -> Self {
        Self { enabled: true }
    }

    fn disabled() -> Self {
        Self { enabled: false }
    }
}

#[async_trait]
impl CaptchaUseCase for TestCaptcha {
    async fn config(&self) -> CaptchaResult<CaptchaConfigResponse> {
        Ok(CaptchaConfigResponse {
            enabled: self.enabled,
            provider: "cap".into(),
            public_config: json!({}),
        })
    }

    async fn challenge(&self) -> CaptchaResult<Value> {
        unimplemented!("auth route tests do not call captcha challenge")
    }

    async fn redeem(&self, _payload: Value) -> CaptchaResult<Value> {
        unimplemented!("auth route tests do not call captcha redeem")
    }

    async fn verify_account(&self, token: Option<&str>) -> CaptchaResult<()> {
        if !self.enabled {
            return Ok(());
        }
        match token {
            Some(VALID_CAPTCHA_TOKEN) => Ok(()),
            Some(_) => Err(captcha::application::CaptchaError::InvalidInput("captcha verification failed".into())),
            None => Err(captcha::application::CaptchaError::InvalidInput("captcha verification is required".into())),
        }
    }
}

struct UnusedRbac;
struct TestConfig {
    register_enabled: bool,
}

impl TestConfig {
    fn new(register_enabled: bool) -> Self {
        Self { register_enabled }
    }
}

#[async_trait]
impl SystemConfigProvider for TestConfig {
    async fn config_by_key(&self, key: &str) -> AppResult<String> {
        match key {
            "sys.account.registerUser" => Ok(self.register_enabled.to_string()),
            "sys.user.initPassword" => Ok(DEFAULT_INIT_PASSWORD.into()),
            _ => Err(crate::application::AppError::NotFound),
        }
    }
}

#[async_trait]
impl RbacUseCase for UnusedRbac {
    async fn navbar(&self, _current_user: &rbac::api::CurrentUser) -> RbacResult<NavResponse> {
        Err(unused_rbac_error())
    }

    async fn authorize_api(&self, _config: &AuthorizationConfig, _request: ApiCheckRequest) -> RbacResult<()> {
        Err(unused_rbac_error())
    }

    async fn data_scope_filter(&self, _current_user: &rbac::api::CurrentUser) -> RbacResult<DataScopeFilter> {
        Err(unused_rbac_error())
    }

    fn validate_protected_handlers(&self, _config: &AuthorizationConfig) -> RbacResult<()> {
        Err(unused_rbac_error())
    }

    fn validate_data_scope_handlers(&self, _handlers: &[&str]) -> RbacResult<()> {
        Err(unused_rbac_error())
    }

    fn is_whitelisted(&self, _config: &AuthorizationConfig, _method: &str, _path: &str) -> RbacResult<bool> {
        Err(unused_rbac_error())
    }
}

fn unused_rbac_error() -> RbacError {
    RbacError::Infrastructure("rbac should not be called by auth route tests".into())
}

fn token_service() -> TokenService {
    TokenService::new(TokenSettings {
        secret: TEST_SECRET.into(),
        access_token_ttl_seconds: ACCESS_TTL_SECONDS,
        refresh_token_ttl_seconds: REFRESH_TTL_SECONDS,
    })
}

async fn sign_in(app: Router) -> SessionTokens {
    let response = app
        .oneshot(json_request(
            Method::POST,
            "/api/auth/sign-in",
            json!({
                "identifier": "alice",
                "password": VALID_PASSWORD
            }),
        ))
        .await
        .unwrap();
    let body = response_json(response).await;

    SessionTokens {
        access_token: body["access_token"].as_str().unwrap().into(),
        refresh_token: body["refresh_token"].as_str().unwrap().into(),
    }
}

fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn authenticated_request(method: Method, uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

async fn response_json(response: Response<Body>) -> Value {
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await
}

async fn json_body(response: Response<Body>) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

fn assert_non_empty_string(value: &Value) {
    assert!(!value.as_str().unwrap().is_empty());
}
