use std::sync::Arc;

use ::rbac::application::{ApiCheckRequest, AuthorizationConfig, RbacError, RbacResult, RbacUseCase};
use async_trait::async_trait;
use axum::{Extension, Router, middleware};
use constants::system_config::{INIT_PASSWORD_KEY, REGISTER_USER_KEY};
use kernel::error::LocalizedError;
use types::{
    http::Locale,
    rbac::{DataScopeFilter, NavResponse},
};

use crate::{
    api::{ApiState, ApiStateParts, TokenService, TokenSettings, TokenSettingsReader, TokenTtlConfig},
    application::{AccountVerifier, AppError, AppResult, IpLocationResolver, PublicIpResolver, SystemConfigProvider, UserService},
    test_support::{MemoryOnlineSessionStore, MemoryUserRepository, TestPasswordHasher, stored_user},
};

use super::super::create_router;

#[path = "support/http.rs"]
mod http;
#[path = "support/rbac.rs"]
mod rbac_fixtures;

pub(super) use http::{
    LocalizedJsonRequest, assert_non_empty_string, authenticated_request, json_body, json_request, json_request_with_accept_language, response_json, sign_in,
};
pub(super) use rbac_fixtures::{admin_current_user, all_data_scope, self_current_user, self_data_scope};

pub(super) use crate::test_support::VALID_PASSWORD;

const TEST_SECRET: &str = "test-secret-with-enough-entropy";
const ACCESS_TTL_SECONDS: u64 = 900;
const REFRESH_TTL_SECONDS: u64 = 604800;
const DEFAULT_INIT_PASSWORD: &str = "12345678";
pub(super) const TEST_PUBLIC_IP: &str = "8.8.8.8";
pub(super) const TEST_LOGIN_LOCATION: &str = "广东省 深圳市";
pub(super) const VALID_CAPTCHA_TOKEN: &str = "valid-captcha-token";
const UNUSED_RBAC_ERROR: &str = "test.rbac.unexpected_call";

pub(super) struct SessionTokens {
    pub(super) access_token: String,
    pub(super) refresh_token: String,
}

pub(super) struct TestApp {
    pub(super) router: Router,
    pub(super) sessions: Arc<MemoryOnlineSessionStore>,
}

struct TestAppInput {
    repository: MemoryUserRepository,
    config: TestConfig,
    captcha: TestCaptcha,
    current_user: ::rbac::api::CurrentUser,
    data_scope: DataScopeFilter,
}

pub(super) fn test_router() -> Router {
    test_app().router
}

pub(super) fn test_app() -> TestApp {
    test_app_with_repository(base_repository(), TestConfig::new(true), TestCaptcha::disabled())
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
    test_app_with_repository(repository, config, captcha).router
}

fn test_app_with_repository(repository: MemoryUserRepository, config: TestConfig, captcha: TestCaptcha) -> TestApp {
    test_app_from_input(TestAppInput {
        repository,
        config,
        captcha,
        current_user: admin_current_user(),
        data_scope: all_data_scope(),
    })
}

pub(super) fn test_app_with_scope(repository: MemoryUserRepository, current_user: ::rbac::api::CurrentUser, data_scope: DataScopeFilter) -> TestApp {
    test_app_from_input(TestAppInput {
        repository,
        config: TestConfig::new(true),
        captcha: TestCaptcha::disabled(),
        current_user,
        data_scope,
    })
}

fn test_app_from_input(input: TestAppInput) -> TestApp {
    let users = UserService::new(input.repository, TestPasswordHasher);
    let sessions = Arc::new(MemoryOnlineSessionStore::default());
    let state = ApiState::new(ApiStateParts {
        users: Arc::new(users),
        tokens: token_service(sessions.clone()),
        rbac: Arc::new(UnusedRbac),
        config: Arc::new(input.config),
        account_verifier: Arc::new(input.captcha),
        public_ip_resolver: Arc::new(TestPublicIpResolver),
        ip_location_resolver: Arc::new(TestIpLocationResolver),
    });
    let router = Router::new()
        .nest("/api", create_router(state))
        .layer(Extension(input.current_user))
        .layer(Extension(input.data_scope))
        .layer(middleware::from_fn(types::http::locale_middleware));
    TestApp { router, sessions }
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

struct TestPublicIpResolver;

#[async_trait]
impl PublicIpResolver for TestPublicIpResolver {
    async fn resolve_public_ip(&self) -> AppResult<String> {
        Ok(TEST_PUBLIC_IP.into())
    }
}

struct TestIpLocationResolver;

#[async_trait]
impl IpLocationResolver for TestIpLocationResolver {
    async fn resolve_login_location(&self, _ipaddr: &str, _locale: Locale) -> AppResult<String> {
        Ok(TEST_LOGIN_LOCATION.into())
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
    async fn navbar(&self, _current_user: &::rbac::api::CurrentUser) -> RbacResult<NavResponse> {
        Err(unused_rbac_error())
    }

    async fn authorize_api(&self, _config: &AuthorizationConfig, _request: ApiCheckRequest) -> RbacResult<()> {
        Err(unused_rbac_error())
    }

    async fn data_scope_filter(&self, _current_user: &::rbac::api::CurrentUser) -> RbacResult<DataScopeFilter> {
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

fn token_service(sessions: Arc<MemoryOnlineSessionStore>) -> TokenService {
    TokenService::with_ttl_reader(TokenSettings { secret: TEST_SECRET.into() }, Arc::new(TestTokenSettingsReader), sessions)
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
