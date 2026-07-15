use std::sync::Arc;

use ::system::{
    api::{SystemApiState, SystemApiStateParts, create_router as create_system_router},
    application::{SystemService, SystemUseCase},
    infra::{RedisSystemCache, StorageSystemRepository},
    notice::{NoticeApiState, create_router as create_notice_router},
};
use audit::{
    api::{AuditApiState, AuditApiStateParts, OperationAuditState, create_router as create_audit_router},
    infra::UserLoginUnlocker,
};
use axum::{Extension, Router, middleware};
use captcha::{
    api::{CaptchaApiState, create_router as create_captcha_router},
    application::{CaptchaProvider, CaptchaService, CaptchaUseCase},
    infra::RedisCaptchaStore,
    providers::{
        cap::CapProvider,
        cloudflare_turnstile::{CloudflareTurnstileProvider, ReqwestTurnstileVerifier},
    },
};
use configuration::{Settings, SettingsError, ValidatedCorsList};
use rbac::api::{RbacApiState, RbacApiStateParts, create_router as create_rbac_router};
use scheduler::api::{SchedulerApiState, SchedulerApiStateParts, create_router as create_scheduler_router};
use storage::{Database, connect_database};
use tower_http::services::ServeDir;
use user::{
    api::{ApiState, ApiStateParts, AuthHttpConfig, RefreshCookieConfig, TokenSettings, create_router as create_user_router},
    infra::LocalAvatarStorage,
};

use self::{
    audit_wiring::{AuditServiceParts, audit_outbox_config, build_audit_services},
    core_wiring::{build_system_services, build_user_services},
    http_pipeline::{RuntimeLayerParts, add_metrics_route, apply_runtime_layers},
    rbac_wiring::build_rbac_services,
    routes::authorization_config,
    runtime_config::{CaptchaAccountVerifier, CaptchaSystemConfig, RuntimeRbacConfig, RuntimeSystemConfig, RuntimeUserConfig},
    scheduler_wiring::build_scheduler_services,
};
use crate::{
    BackendResult,
    app_state::AppState,
    auth::{AuthState, AuthStateParts, auth_middleware},
    docs, migration, system,
};

pub(crate) mod access_catalog;
mod audit_wiring;
mod bootstrap_wiring;
mod core_wiring;
mod http_pipeline;
mod rbac_wiring;
mod routes;
mod runtime_config;
mod scheduler_wiring;
pub(crate) use bootstrap_wiring::bootstrap_admin;
#[cfg(test)]
pub(crate) mod tests;

const AVATAR_URL_PREFIX: &str = "/uploads/avatars";

pub async fn build_app_state(settings: &Settings) -> BackendResult<AppState> {
    let database = connect_database(&settings.database_url()?).await?;
    migration::prepare_runtime_schema(database.pool(), settings.database.auto_migrate).await?;
    let rbac = build_rbac_services(settings, database.clone()).await?;
    let endpoints = access_catalog::EndpointCatalog::build()?;
    let authorization = authorization_config(settings, &endpoints)?;
    rbac.use_case.validate_protected_handlers(&authorization)?;
    let system = build_system_services(settings, database.clone()).await?;
    let users = build_user_services(settings, database.clone(), system.use_case.clone()).await?;
    let captcha = build_captcha_service(settings, system.use_case.clone()).await?;
    let scheduler = build_scheduler_services(settings, database.clone(), system.use_case.clone())?;
    let audit = build_audit_services(AuditServiceParts {
        database,
        system: system.use_case.clone(),
        location_resolver: users.location_resolver.clone(),
        outbox: audit_outbox_config(settings)?,
    })?;

    Ok(AppState {
        users: users.use_case,
        tokens: users.tokens,
        session_cleanup_runtime: users.session_cleanup_runtime,
        rbac: rbac.use_case,
        rbac_admin: rbac.admin,
        rbac_audited_admin: rbac.audited_admin,
        rbac_cache_refresher: rbac.cache_refresher,
        system: system.use_case,
        system_audited: system.audited,
        notices: system.notices,
        notices_audited: system.notices_audited,
        metrics: system.metrics,
        captcha,
        audit: audit.use_case,
        audit_outbox: audit.outbox,
        audit_outbox_runtime: audit.runtime,
        audit_export_config: audit.export_config,
        ip_location_resolver: users.location_resolver,
        scheduler: scheduler.use_case,
        scheduler_audited: scheduler.audited,
        scheduler_export_config: scheduler.export_config,
        scheduler_runtime: scheduler.runtime,
        authorization,
        endpoints,
    })
}

pub async fn build_router(settings: &Settings, metrics_handle: hook_tracing::MetricsHandle) -> BackendResult<Router> {
    let state = build_app_state(settings).await?;
    create_app(state, settings, metrics_handle)
}

#[cfg(test)]
pub(crate) fn build_public_router(settings: &Settings, metrics_handle: hook_tracing::MetricsHandle) -> BackendResult<Router> {
    let app = add_metrics_route(public_routes(settings), &metrics_handle);
    let app = app.layer(middleware::from_fn(types::http::locale_middleware));
    let app = http_pipeline::with_timeout(app, settings)?;
    let app = http_pipeline::apply_metrics_layer(app, &metrics_handle);
    http_pipeline::apply_http_layers(app, settings)
}

pub fn create_app(state: AppState, settings: &Settings, metrics_handle: hook_tracing::MetricsHandle) -> BackendResult<Router> {
    let auth_state = AuthState::new(AuthStateParts {
        users: state.users.clone(),
        tokens: state.tokens.clone(),
        rbac: state.rbac.clone(),
        system: state.system.clone(),
        authorization: state.authorization.clone(),
        endpoints: state.endpoints.clone(),
    });
    let api_router = create_api_router(&state, settings)?;
    let app = public_routes(settings).nest("/api", api_router);
    let app = add_metrics_route(app, &metrics_handle);
    let app = app.layer(Extension(state.audit_outbox_runtime.clone()));
    let app = app.layer(Extension(state.session_cleanup_runtime.clone()));
    let app = app.layer(middleware::from_fn_with_state(auth_state, auth_middleware));
    apply_runtime_layers(
        app,
        RuntimeLayerParts {
            settings,
            audit: operation_audit_state(&state)?,
            metrics: &metrics_handle,
        },
    )
}

fn create_api_router(state: &AppState, settings: &Settings) -> BackendResult<Router> {
    Ok(Router::new()
        .merge(create_user_router(user_api_state(state, settings)?))
        .merge(create_rbac_router(
            RbacApiState::new(RbacApiStateParts {
                rbac: state.rbac.clone(),
                rbac_admin: state.rbac_admin.clone(),
                rbac_audited_admin: state.rbac_audited_admin.clone(),
                rbac_cache_refresher: state.rbac_cache_refresher.clone(),
            })
            .with_export_config(Arc::new(RuntimeRbacConfig::new(state.system.clone()))),
        ))
        .merge(create_system_router(system_api_state(state)))
        .merge(create_notice_router(NoticeApiState::new(state.notices.clone(), state.notices_audited.clone())))
        .merge(create_captcha_router(CaptchaApiState::new(state.captcha.clone())))
        .merge(create_audit_router(audit_api_state(state)))
        .merge(create_scheduler_router(SchedulerApiState::new(SchedulerApiStateParts {
            scheduler: state.scheduler.clone(),
            audited_scheduler: state.scheduler_audited.clone(),
            export_config: state.scheduler_export_config.clone(),
            runtime: state.scheduler_runtime.clone(),
        }))))
}

fn user_api_state(state: &AppState, settings: &Settings) -> BackendResult<ApiState> {
    let user_config = Arc::new(RuntimeUserConfig::new(state.system.clone()));
    let account_verifier = Arc::new(CaptchaAccountVerifier::new(state.captcha.clone()));
    let avatar_storage = Arc::new(LocalAvatarStorage::new(settings.uploads.avatar_directory.clone(), AVATAR_URL_PREFIX));
    Ok(ApiState::new(ApiStateParts {
        users: state.users.clone(),
        tokens: state.tokens.clone(),
        rbac: state.rbac.clone(),
        config: user_config.clone(),
        account_verifier,
        ip_location_resolver: state.ip_location_resolver.clone(),
        operation_audit: state.audit_outbox.clone(),
        security_audit: state.audit_outbox.clone(),
        auth_http: auth_http_config(settings)?,
    })
    .with_avatar_storage(avatar_storage)
    .with_avatar_config(user_config.clone())
    .with_export_config(user_config))
}

fn auth_http_config(settings: &Settings) -> BackendResult<AuthHttpConfig> {
    let cookie = settings.refresh_cookie_config()?;
    let cors = settings.validated_cors()?;
    let ValidatedCorsList::Values(trusted_origins) = cors.allowed_origins else {
        return Err(SettingsError::WildcardCorsOrigin("cors.allowed_origins").into());
    };
    Ok(AuthHttpConfig {
        refresh_cookie: RefreshCookieConfig {
            secure: cookie.secure,
            domain: cookie.domain,
            path: cookie.path,
        },
        trusted_origins,
    })
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

fn operation_audit_state(state: &AppState) -> Result<OperationAuditState, audit::application::AuditError> {
    OperationAuditState::try_new(state.endpoints.specs().to_vec(), state.audit_outbox.clone())
}

fn public_routes(settings: &Settings) -> Router {
    docs::router()
        .merge(system::create_router())
        .nest_service(AVATAR_URL_PREFIX, ServeDir::new(&settings.uploads.avatar_directory))
}

pub use rbac_wiring::rebuild_rbac_cache;

pub async fn rebuild_persistent_system_cache(settings: &Settings, database: Database) -> BackendResult<()> {
    let cache = RedisSystemCache::connect(&settings.redis_url()?, settings.redis.key_prefix.clone()).await?;
    let system: Arc<dyn SystemUseCase> = Arc::new(SystemService::with_cache(StorageSystemRepository::new(database), cache));
    rebuild_system_cache(&system).await
}

async fn rebuild_system_cache(system: &Arc<dyn SystemUseCase>) -> BackendResult<()> {
    system.refresh_config_cache().await?;
    system.refresh_dict_cache().await?;
    Ok(())
}

async fn build_captcha_service(settings: &Settings, system: Arc<dyn SystemUseCase>) -> BackendResult<Arc<dyn CaptchaUseCase>> {
    let store = RedisCaptchaStore::connect(&settings.redis_url()?, settings.redis.key_prefix.clone()).await?;
    let turnstile_secret_key = settings.cloudflare_turnstile_secret_key();
    let providers: Vec<Arc<dyn CaptchaProvider>> = vec![
        Arc::new(CapProvider::new(store)),
        Arc::new(CloudflareTurnstileProvider::new(ReqwestTurnstileVerifier::new(), turnstile_secret_key)),
    ];
    Ok(Arc::new(CaptchaService::new(CaptchaSystemConfig::new(system), providers)))
}

fn token_settings(settings: &Settings) -> BackendResult<TokenSettings> {
    Ok(TokenSettings {
        secret: settings.jwt_secret()?,
    })
}
