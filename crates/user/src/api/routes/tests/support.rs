use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use ::rbac::application::{ApiCheckRequest, AuthorizationConfig, RbacError, RbacResult, RbacUseCase};
use async_trait::async_trait;
use audit_contract::{AuditOutboxEvent, SecurityAuditEvent};
use axum::{Extension, Router, extract::ConnectInfo, middleware};
use client_info::{ClientInfoResult, IpLocation, IpLocationResolver};
use constants::system_config::REGISTER_USER_KEY;
use kernel::error::LocalizedError;
use rbac::domain::DataScopeFilter;
use types::rbac::NavResponse;

use crate::{
    api::{ApiState, ApiStateParts, AuthHttpConfig, RefreshCookieConfig},
    application::{AccountVerifier, AppError, AppResult, SystemConfigProvider, UserService},
    test_support::{MemoryLoginFailureStore, MemoryOnlineSessionStore, MemoryUserRepository, TestLoginLockConfigProvider, TestPasswordHasher, stored_user},
};

use super::super::create_router;

#[path = "support/audit.rs"]
mod audit;
#[path = "support/audit_recorders.rs"]
mod audit_recorders;
#[path = "support/http.rs"]
mod http;
#[path = "support/rbac.rs"]
mod rbac_fixtures;
#[path = "support/tokens.rs"]
mod tokens;

pub(super) use audit_recorders::{MemoryOperationAuditRecorder, MemorySecurityAuditRecorder};
pub(super) use http::{
    LocalizedJsonRequest, assert_non_empty_string, authenticated_request, json_body, json_request, json_request_with_accept_language, refresh_cookie_request,
    response_json, sign_in,
};
pub(super) use rbac_fixtures::{admin_current_user, all_data_scope, self_current_user, self_data_scope};

pub(super) use crate::test_support::VALID_PASSWORD;

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
    pub(super) repository: MemoryUserRepository,
    pub(super) events: Arc<MemorySecurityAuditRecorder>,
    pub(super) operation_events: Arc<MemoryOperationAuditRecorder>,
}

impl TestApp {
    pub(super) fn persisted_security_events(&self) -> Vec<SecurityAuditEvent> {
        self.repository
            .audit_records()
            .into_iter()
            .filter_map(|record| match record.event {
                AuditOutboxEvent::Security(event) => Some(event),
                AuditOutboxEvent::Operation(_) => None,
            })
            .collect()
    }
}

struct TestAppInput {
    repository: MemoryUserRepository,
    login_failures: MemoryLoginFailureStore,
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

pub(super) fn test_app_with_failed_login_cleanup() -> TestApp {
    let login_failures = MemoryLoginFailureStore::default();
    login_failures.fail_clear_with("login counter cleanup failed");
    test_app_from_input(TestAppInput {
        repository: base_repository(),
        login_failures,
        config: TestConfig::new(true),
        captcha: TestCaptcha::disabled(),
        current_user: admin_current_user(),
        data_scope: all_data_scope(),
    })
}

pub(super) fn test_app_with_config(config: TestConfig) -> TestApp {
    test_app_with_repository(base_repository(), config, TestCaptcha::disabled())
}

pub(super) fn test_router_with_captcha(captcha: TestCaptcha) -> Router {
    test_app_with_captcha(captcha).router
}

pub(super) fn test_app_with_captcha(captcha: TestCaptcha) -> TestApp {
    test_app_with_repository(base_repository(), TestConfig::new(true), captcha)
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
        login_failures: MemoryLoginFailureStore::default(),
        config,
        captcha,
        current_user: admin_current_user(),
        data_scope: all_data_scope(),
    })
}

pub(super) fn test_app_with_scope(repository: MemoryUserRepository, current_user: ::rbac::api::CurrentUser, data_scope: DataScopeFilter) -> TestApp {
    test_app_from_input(TestAppInput {
        repository,
        login_failures: MemoryLoginFailureStore::default(),
        config: TestConfig::new(true),
        captcha: TestCaptcha::disabled(),
        current_user,
        data_scope,
    })
}

fn test_app_from_input(input: TestAppInput) -> TestApp {
    let repository = input.repository.clone();
    let users = UserService::new(input.repository, TestPasswordHasher).with_login_security(input.login_failures, TestLoginLockConfigProvider::default());
    let sessions = Arc::new(MemoryOnlineSessionStore::default());
    let events = Arc::new(MemorySecurityAuditRecorder::new(sessions.clone()));
    let operation_events = Arc::new(MemoryOperationAuditRecorder::default());
    let state = ApiState::new(ApiStateParts {
        users: Arc::new(users),
        tokens: tokens::token_service(sessions.clone()),
        rbac: Arc::new(UnusedRbac),
        config: Arc::new(input.config),
        account_verifier: Arc::new(input.captcha),
        ip_location_resolver: Arc::new(TestIpLocationResolver),
        operation_audit: operation_events.clone(),
        security_audit: events.clone(),
        auth_http: test_auth_http_config(),
    });
    let router = Router::new()
        .nest("/api", create_router(state))
        .layer(Extension(input.current_user))
        .layer(Extension(input.data_scope))
        .layer(Extension(ConnectInfo(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 40000))))
        .layer(middleware::from_fn(audit::operation_context_middleware))
        .layer(middleware::from_fn(types::http::locale_middleware));
    TestApp {
        router,
        sessions,
        repository,
        events,
        operation_events,
    }
}

fn test_auth_http_config() -> AuthHttpConfig {
    AuthHttpConfig {
        refresh_cookie: RefreshCookieConfig {
            secure: true,
            path: "/api/auth".into(),
        },
        trusted_origins: vec!["http://localhost:8082".into()],
    }
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

struct TestIpLocationResolver;

#[async_trait]
impl IpLocationResolver for TestIpLocationResolver {
    async fn resolve_ip_location(&self, _ipaddr: &str) -> ClientInfoResult<IpLocation> {
        Ok(IpLocation::Resolved(TEST_LOGIN_LOCATION.into()))
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

    fn is_whitelisted(&self, _config: &AuthorizationConfig, _method: &str, _path: &str) -> RbacResult<bool> {
        Err(unused_rbac_error())
    }
}

fn unused_rbac_error() -> RbacError {
    RbacError::Infrastructure(UNUSED_RBAC_ERROR.into())
}
