use std::sync::Arc;

use async_trait::async_trait;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, Response, StatusCode, header},
    middleware,
};
use constants::system_config::{INIT_PASSWORD_KEY, REGISTER_USER_KEY};
use kernel::error::LocalizedError;
use rbac::application::{ApiCheckRequest, AuthorizationConfig, RbacError, RbacResult, RbacUseCase};
use serde_json::{Value, json};
use tower::ServiceExt;
use types::rbac::{DataScopeFilter, NavResponse};

use crate::{
    api::{ApiState, ApiStateParts, TokenService, TokenSettings, TokenSettingsReader, TokenTtlConfig},
    application::{AccountVerifier, AppError, AppResult, SystemConfigProvider, UserService},
    test_support::{MemoryUserRepository, TestPasswordHasher, stored_user},
};

use super::super::create_router;

pub(super) use crate::test_support::VALID_PASSWORD;

const TEST_SECRET: &str = "test-secret-with-enough-entropy";
const ACCESS_TTL_SECONDS: u64 = 900;
const REFRESH_TTL_SECONDS: u64 = 604800;
const DEFAULT_INIT_PASSWORD: &str = "12345678";
pub(super) const VALID_CAPTCHA_TOKEN: &str = "valid-captcha-token";
const UNUSED_RBAC_ERROR: &str = "test.rbac.unexpected_call";

pub(super) struct SessionTokens {
    pub(super) access_token: String,
    pub(super) refresh_token: String,
}

pub(super) struct LocalizedJsonRequest<'a> {
    pub(super) method: Method,
    pub(super) uri: &'a str,
    pub(super) body: Value,
    pub(super) accept_language: &'a str,
}

pub(super) fn test_router() -> Router {
    test_router_with_config(TestConfig::new(true))
}

pub(super) fn test_router_with_config(config: TestConfig) -> Router {
    test_router_with_repository(base_repository(), config)
}

pub(super) fn test_router_with_captcha(captcha: TestCaptcha) -> Router {
    test_router_with_repository_and_captcha(base_repository(), TestConfig::new(true), captcha)
}

pub(super) fn test_router_with_repository(repository: MemoryUserRepository, config: TestConfig) -> Router {
    test_router_with_repository_and_captcha(repository, config, TestCaptcha::disabled())
}

fn test_router_with_repository_and_captcha(repository: MemoryUserRepository, config: TestConfig, captcha: TestCaptcha) -> Router {
    let users = UserService::new(repository, TestPasswordHasher);
    let state = ApiState::new(ApiStateParts {
        users: Arc::new(users),
        tokens: token_service(),
        rbac: Arc::new(UnusedRbac),
        config: Arc::new(config),
        account_verifier: Arc::new(captcha),
    });
    Router::new()
        .nest("/api", create_router(state))
        .layer(middleware::from_fn(types::http::locale_middleware))
}

pub(super) fn base_repository() -> MemoryUserRepository {
    MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"))
}

pub(super) struct TestCaptcha {
    enabled: bool,
}

impl TestCaptcha {
    pub(super) fn enabled() -> Self {
        Self { enabled: true }
    }

    fn disabled() -> Self {
        Self { enabled: false }
    }
}

#[async_trait]
impl AccountVerifier for TestCaptcha {
    async fn verify_account(&self, token: Option<&str>) -> AppResult<()> {
        if !self.enabled {
            return Ok(());
        }
        match token {
            Some(VALID_CAPTCHA_TOKEN) => Ok(()),
            Some(_) => Err(AppError::InvalidInput(LocalizedError::new("errors.captcha.verification_failed"))),
            None => Err(AppError::InvalidInput(LocalizedError::new("errors.captcha.verification_required"))),
        }
    }
}

struct UnusedRbac;

pub(super) struct TestConfig {
    register_enabled: bool,
}

impl TestConfig {
    pub(super) fn new(register_enabled: bool) -> Self {
        Self { register_enabled }
    }
}

#[async_trait]
impl SystemConfigProvider for TestConfig {
    async fn config_by_key(&self, key: &str) -> AppResult<String> {
        match key {
            REGISTER_USER_KEY => Ok(self.register_enabled.to_string()),
            INIT_PASSWORD_KEY => Ok(DEFAULT_INIT_PASSWORD.into()),
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
    RbacError::Infrastructure(UNUSED_RBAC_ERROR.into())
}

fn token_service() -> TokenService {
    TokenService::with_ttl_reader(TokenSettings { secret: TEST_SECRET.into() }, Arc::new(TestTokenSettingsReader))
}

struct TestTokenSettingsReader;

#[async_trait]
impl TokenSettingsReader for TestTokenSettingsReader {
    async fn token_ttl_config(&self) -> AppResult<TokenTtlConfig> {
        Ok(TokenTtlConfig {
            access_token_ttl_seconds: ACCESS_TTL_SECONDS,
            refresh_token_ttl_seconds: REFRESH_TTL_SECONDS,
        })
    }
}

pub(super) async fn sign_in(app: Router) -> SessionTokens {
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

pub(super) fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

pub(super) fn json_request_with_accept_language(input: LocalizedJsonRequest<'_>) -> Request<Body> {
    Request::builder()
        .method(input.method)
        .uri(input.uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT_LANGUAGE, input.accept_language)
        .body(Body::from(input.body.to_string()))
        .unwrap()
}

pub(super) fn authenticated_request(method: Method, uri: &str, token: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

pub(super) async fn response_json(response: Response<Body>) -> Value {
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await
}

pub(super) async fn json_body(response: Response<Body>) -> Value {
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

pub(super) fn assert_non_empty_string(value: &Value) {
    assert!(!value.as_str().unwrap().is_empty());
}
