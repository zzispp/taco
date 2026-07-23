use std::sync::Arc;

use ::system::{
    api::{SystemApiState, SystemApiStateParts, create_router as create_system_router},
    notice::{NoticeApiState, create_router as create_notice_router},
};
use audit::{
    api::{AuditApiState, AuditApiStateParts, OperationAuditState, create_router as create_audit_router},
    infra::UserLoginUnlocker,
};
use axum::{Extension, Router, middleware};
use captcha::api::{CaptchaApiState, create_router as create_captcha_router};
use configuration::Settings;
use file::{
    api::{FileApiState, create_router as create_file_router},
    infra::ManagedAvatarStorage,
};
use observability::api::{SystemLogApiState, SystemLogApiStateParts, create_router as create_system_log_router};
use rbac::api::{RbacApiState, RbacApiStateParts, create_router as create_rbac_router};
use scheduler::api::{SchedulerApiState, SchedulerApiStateParts, create_router as create_scheduler_router};
use user::api::{ApiState, ApiStateParts, AvatarProjectionApiState, create_avatar_projection_router, create_router as create_user_router};

#[cfg(test)]
use super::{http_pipeline, tracing_runtime::TracingConfigListenerHealth};
use super::{
    http_pipeline::{RuntimeLayerParts, add_metrics_route, apply_runtime_layers},
    runtime_config::{CaptchaAccountVerifier, RuntimeRbacConfig, RuntimeSystemConfig, RuntimeUserConfig},
    system_log_cleanup_execution,
};
use crate::{
    BackendResult,
    app_state::AppState,
    auth::{AuthState, AuthStateParts, auth_middleware},
    docs, system,
};

#[cfg(test)]
pub(crate) fn build_public_router(settings: &Settings, metrics_handle: taco_tracing::MetricsHandle) -> BackendResult<Router> {
    let app = crate::embedded_frontend::with_embedded_frontend(public_routes(system::HealthState::for_test(TracingConfigListenerHealth::new())));
    let app = add_metrics_route(app, &metrics_handle);
    let app = app.layer(middleware::from_fn(types::http::locale_middleware));
    let app = http_pipeline::with_timeout(app, settings)?;
    let app = http_pipeline::apply_metrics_layer(app, &metrics_handle);
    http_pipeline::apply_http_layers(app, settings)
}

pub fn create_app(state: AppState, settings: &Settings, metrics_handle: taco_tracing::MetricsHandle) -> BackendResult<Router> {
    let auth_state = auth_state(&state);
    let api_router = create_api_router(&state).layer(middleware::from_fn_with_state(auth_state, auth_middleware));
    let public = public_routes(system::HealthState::new(
        state.tracing_config_listener_health.clone(),
        state.system_log_runtime.clone(),
    ))
    .merge(create_avatar_router(&state));
    let app = crate::embedded_frontend::with_embedded_frontend(public.nest("/api", api_router));
    let app = app.layer(Extension(state.audit_outbox_runtime.clone()));
    let app = app.layer(Extension(state.session_cleanup_runtime.clone()));
    let app = app.layer(Extension(state.system_log_runtime.clone()));
    let app = app.layer(Extension(state._tracing_config_listener_runtime.clone()));
    apply_runtime_layers(
        add_metrics_route(app, &metrics_handle),
        RuntimeLayerParts {
            settings,
            audit: operation_audit_state(&state)?,
            metrics: &metrics_handle,
            http_logs: Some(state.http_log_state.clone()),
        },
    )
}

fn auth_state(state: &AppState) -> AuthState {
    AuthState::new(AuthStateParts {
        users: state.users.clone(),
        tokens: state.tokens.clone(),
        rbac: state.rbac.clone(),
        system: state.system.clone(),
        authorization: state.authorization.clone(),
        endpoints: state.endpoints.clone(),
    })
}

fn create_avatar_router(state: &AppState) -> Router {
    let avatar_storage = Arc::new(ManagedAvatarStorage::new(state.files.clone()));
    create_avatar_projection_router(AvatarProjectionApiState::new(state.users.clone(), avatar_storage))
}

fn create_api_router(state: &AppState) -> Router {
    Router::new()
        .merge(create_user_router(user_api_state(state)))
        .merge(create_rbac_router(rbac_api_state(state)))
        .merge(create_system_router(system_api_state(state)))
        .merge(create_notice_router(NoticeApiState::new(state.notices.clone(), state.notices_audited.clone())))
        .merge(create_captcha_router(CaptchaApiState::new(state.captcha.clone())))
        .merge(create_file_router(FileApiState::new(state.files.clone(), state.audit_outbox.clone())))
        .merge(create_audit_router(audit_api_state(state)))
        .merge(create_system_log_router(system_log_api_state(state)))
        .merge(create_scheduler_router(scheduler_api_state(state)))
}

fn user_api_state(state: &AppState) -> ApiState {
    let user_config = Arc::new(RuntimeUserConfig::new(state.system.clone()));
    ApiState::new(ApiStateParts {
        users: state.users.clone(),
        tokens: state.tokens.clone(),
        rbac: state.rbac.clone(),
        config: user_config.clone(),
        account_verifier: Arc::new(CaptchaAccountVerifier::new(state.captcha.clone())),
        ip_location_resolver: state.ip_location_resolver.clone(),
        operation_audit: state.audit_outbox.clone(),
        security_audit: state.audit_outbox.clone(),
    })
    .with_avatar_storage(Arc::new(ManagedAvatarStorage::new(state.files.clone())))
    .with_avatar_config(user_config.clone())
    .with_export_config(user_config)
}

fn rbac_api_state(state: &AppState) -> RbacApiState {
    RbacApiState::new(RbacApiStateParts {
        rbac: state.rbac.clone(),
        rbac_admin: state.rbac_admin.clone(),
        rbac_audited_admin: state.rbac_audited_admin.clone(),
        rbac_cache_refresher: state.rbac_cache_refresher.clone(),
    })
    .with_export_config(Arc::new(RuntimeRbacConfig::new(state.system.clone())))
}

fn system_api_state(state: &AppState) -> SystemApiState {
    SystemApiState::new(SystemApiStateParts {
        system: state.system.clone(),
        system_audited: state.system_audited.clone(),
        operation_audit: state.audit_outbox.clone(),
        metrics: state.metrics.clone(),
        rbac: state.rbac.clone(),
        rbac_admin: state.rbac_admin.clone(),
    })
    .with_export_config(Arc::new(RuntimeSystemConfig::new(state.system.clone())))
}

fn audit_api_state(state: &AppState) -> AuditApiState {
    AuditApiState::new(AuditApiStateParts {
        audit: state.audit.clone(),
        export_config: state.audit_export_config.clone(),
        unlocker: Arc::new(UserLoginUnlocker::new(state.users.clone())),
    })
}

fn system_log_api_state(state: &AppState) -> SystemLogApiState {
    SystemLogApiState::new(SystemLogApiStateParts {
        logs: state.system_logs.clone(),
        exporter: state.system_log_exporter.clone(),
        cleanup_executions: Arc::new(system_log_cleanup_execution::SchedulerSystemLogCleanupExecutionAdapter::new(
            state.scheduler.clone(),
            state.scheduler_audited.clone(),
        )),
        export_config: state.system_log_export_config.clone(),
    })
}

fn scheduler_api_state(state: &AppState) -> SchedulerApiState {
    SchedulerApiState::new(SchedulerApiStateParts {
        scheduler: state.scheduler.clone(),
        audited_scheduler: state.scheduler_audited.clone(),
        export_config: state.scheduler_export_config.clone(),
        runtime: state.scheduler_runtime.clone(),
    })
}

fn operation_audit_state(state: &AppState) -> Result<OperationAuditState, audit::application::AuditError> {
    OperationAuditState::try_new(state.endpoints.specs().to_vec(), state.audit_outbox.clone())
}

fn public_routes(health_state: system::HealthState) -> Router {
    docs::router().merge(system::create_router(health_state))
}
