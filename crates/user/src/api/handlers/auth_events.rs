use std::collections::BTreeMap;

use audit_contract::{AuditOutboxError, AuditOutboxEvent, AuditOutboxRecord, AuditStatus, LoginEventType, SecurityAuditEvent, SecurityAuditRecorder};

use crate::{
    application::{AppError, AppResult},
    domain::{User, UserId},
};

const LOGIN_SUCCESS_KEY: &str = "messages.user.login_success";
const LOGOUT_SUCCESS_KEY: &str = "messages.user.logout_success";
const REGISTER_SUCCESS_KEY: &str = "messages.user.register_success";
const REFRESH_SUCCESS_KEY: &str = "messages.user.refresh_success";
const SERVICE_UNAVAILABLE_KEY: &str = "errors.common.service_unavailable";

#[derive(Clone)]
pub struct AuthenticationEventContext {
    request_id: String,
    route: String,
    ip_address: String,
    browser: String,
    os: String,
}

impl AuthenticationEventContext {
    pub fn from_client(client: &client_info::ClientInfo, request_id: String, route: String) -> Self {
        Self {
            request_id,
            route,
            ip_address: client.ipaddr(),
            browser: client.browser.clone(),
            os: client.os.clone(),
        }
    }
}

pub struct AuthEventPublisher<'a> {
    recorder: &'a dyn SecurityAuditRecorder,
    context: AuthenticationEventContext,
}

impl<'a> AuthEventPublisher<'a> {
    pub fn new(recorder: &'a dyn SecurityAuditRecorder, context: AuthenticationEventContext) -> Self {
        Self { recorder, context }
    }

    pub(super) fn login_success_record(&self, user: &User) -> AuditOutboxRecord {
        self.record(success_event(LoginEventType::LoginSuccess, SecuritySubject::from(user), LOGIN_SUCCESS_KEY))
    }

    pub(super) async fn login_failure(&self, username: String, error: &AppError) -> AppResult<()> {
        self.authentication_failure(LoginEventType::LoginFailure, username, error).await
    }

    pub(super) fn register_success_record(&self, username: String) -> AuditOutboxRecord {
        self.record(success_event(
            LoginEventType::RegisterSuccess,
            SecuritySubject::new(username, None),
            REGISTER_SUCCESS_KEY,
        ))
    }

    pub(super) async fn register_failure(&self, username: String, error: &AppError) -> AppResult<()> {
        self.authentication_failure(LoginEventType::RegisterFailure, username, error).await
    }

    pub(super) async fn refresh_success(&self, user: &User) -> AppResult<()> {
        self.publish(success_event(LoginEventType::RefreshSuccess, SecuritySubject::from(user), REFRESH_SUCCESS_KEY))
            .await
    }

    pub(super) async fn refresh_failure(&self, username: String, error: &AppError) -> AppResult<()> {
        self.authentication_failure(LoginEventType::RefreshFailure, username, error).await
    }

    pub(super) async fn logout_success(&self, username: String, user_id: UserId) -> AppResult<()> {
        self.publish(success_event(
            LoginEventType::LogoutSuccess,
            SecuritySubject::new(username, Some(user_id)),
            LOGOUT_SUCCESS_KEY,
        ))
        .await
    }

    pub async fn logout_failure(&self, username: String, error: &AppError) -> AppResult<()> {
        self.authentication_failure(LoginEventType::LogoutFailure, username, error).await
    }

    pub(super) async fn authentication_failure(&self, event_type: LoginEventType, username: String, error: &AppError) -> AppResult<()> {
        self.publish(failure_event(event_type, username, error)).await
    }

    pub(super) async fn publish_record(&self, record: AuditOutboxRecord) -> AppResult<()> {
        self.recorder.record(record).await.map_err(audit_error)
    }

    async fn publish(&self, event: SecurityAuditEvent) -> AppResult<()> {
        self.publish_record(self.record(event)).await
    }

    fn record(&self, event: SecurityAuditEvent) -> AuditOutboxRecord {
        let event = SecurityAuditEvent {
            request_id: self.context.request_id.clone(),
            route: self.context.route.clone(),
            ip_address: self.context.ip_address.clone(),
            browser: self.context.browser.clone(),
            os: self.context.os.clone(),
            ..event
        };
        AuditOutboxRecord {
            id: uuid::Uuid::now_v7().to_string(),
            occurred_at: time::OffsetDateTime::now_utc(),
            event: AuditOutboxEvent::Security(event),
        }
    }
}

struct SecuritySubject {
    username: String,
    user_id: Option<UserId>,
}

impl SecuritySubject {
    fn new(username: String, user_id: Option<UserId>) -> Self {
        Self { username, user_id }
    }
}

impl From<&User> for SecuritySubject {
    fn from(user: &User) -> Self {
        Self::new(user.username.clone(), Some(user.id.clone()))
    }
}

fn success_event(event_type: LoginEventType, subject: SecuritySubject, message_key: &'static str) -> SecurityAuditEvent {
    SecurityAuditEvent {
        request_id: String::new(),
        route: String::new(),
        user_id: subject.user_id.map(|id| id.0),
        username: subject.username,
        ip_address: String::new(),
        browser: String::new(),
        os: String::new(),
        status: AuditStatus::Success,
        event_type,
        message_key: message_key.into(),
        message_params: BTreeMap::new(),
    }
}

fn failure_event(event_type: LoginEventType, username: String, error: &AppError) -> SecurityAuditEvent {
    let (message_key, message_params) = failure_message(error);
    SecurityAuditEvent {
        request_id: String::new(),
        route: String::new(),
        user_id: None,
        username,
        ip_address: String::new(),
        browser: String::new(),
        os: String::new(),
        status: AuditStatus::Failure,
        event_type,
        message_key: message_key.into(),
        message_params,
    }
}

fn audit_error(error: AuditOutboxError) -> AppError {
    AppError::Infrastructure(format!("audit outbox recording failed: {error}"))
}

fn failure_message(error: &AppError) -> (&'static str, BTreeMap<String, String>) {
    match error {
        AppError::InvalidCursor => ("errors.common.invalid_cursor", BTreeMap::new()),
        AppError::InvalidInput(error) | AppError::Forbidden(error) | AppError::Conflict(error) => (error.key(), error_params(error)),
        AppError::ImportValidation(_) => ("errors.common.invalid_input", BTreeMap::new()),
        AppError::Unauthorized => ("errors.user.invalid_credentials", BTreeMap::new()),
        AppError::AccountDisabled => ("errors.user.account_disabled", BTreeMap::new()),
        AppError::AccountLocked { lock_minutes } => ("errors.user.account_locked", BTreeMap::from([("minutes".into(), lock_minutes.to_string())])),
        AppError::NotFound => ("errors.common.not_found", BTreeMap::new()),
        AppError::Infrastructure(_) => (SERVICE_UNAVAILABLE_KEY, BTreeMap::new()),
    }
}

fn error_params(error: &kernel::error::LocalizedError) -> BTreeMap<String, String> {
    error.params().iter().map(|param| (param.key().into(), param.value().into())).collect()
}
