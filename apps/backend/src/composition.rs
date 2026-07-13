use std::sync::Arc;

use ::system::{
    api::{SystemApiState, SystemApiStateParts, create_router as create_system_router},
    application::{ServerMetricsUseCase, SystemMetricsService, SystemService, SystemUseCase},
    infra::{RedisSystemCache, StorageSystemRepository, SysinfoServerMetricsCollector},
};
use axum::{Router, middleware};
use captcha::{
    api::{CaptchaApiState, create_router as create_captcha_router},
    application::{CaptchaProvider, CaptchaService, CaptchaUseCase},
    infra::RedisCaptchaStore,
    providers::{
        cap::CapProvider,
        cloudflare_turnstile::{CloudflareTurnstileProvider, ReqwestTurnstileVerifier},
    },
};
use configuration::Settings;
use rbac::{
    api::{RbacApiState, create_router as create_rbac_router},
    application::{RbacAdminUseCase, RbacService, RbacUseCase},
    infra::{RedisRbacCache, StorageRbacRepository},
};
use scheduler::api::{SchedulerApiState, SchedulerApiStateParts, create_router as create_scheduler_router};
use storage::{Database, connect_database};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use user::{
    api::{ApiState, ApiStateParts, TokenService, TokenSettings, create_router as create_user_router},
    application::{UserService, UserUseCase},
    infra::{Argon2PasswordHasher, LocalAvatarStorage, PconlineIpLocationResolver, PublicIpAddressResolver, RedisOnlineSessionStore, StorageUserRepository},
};

use self::{
    routes::{authorization_config, data_scope_handlers},
    runtime_config::{CaptchaAccountVerifier, CaptchaSystemConfig, RuntimeRbacConfig, RuntimeSystemConfig, RuntimeUserConfig},
    scheduler_wiring::build_scheduler_services,
};
use crate::{
    BackendResult,
    app_state::AppState,
    auth::{AuthState, AuthStateParts, auth_middleware},
    docs, http_config, migration, system,
};

mod routes;
mod runtime_config;
mod scheduler_wiring;
#[cfg(test)]
mod tests;

const AVATAR_URL_PREFIX: &str = "/uploads/avatars";

struct RbacServices {
    use_case: Arc<dyn RbacUseCase>,
    admin: Arc<dyn RbacAdminUseCase>,
}

pub async fn build_app_state(settings: &Settings) -> BackendResult<AppState> {
    let database = connect_database(&settings.database_url()?).await?;
    migration::prepare_runtime_schema(database.pool(), settings.database.auto_migrate).await?;
    let rbac = build_rbac_services(settings, database.clone()).await?;
    let authorization = authorization_config(settings);
    rbac.use_case.validate_protected_handlers(&authorization)?;
    rbac.use_case.validate_data_scope_handlers(&data_scope_handlers())?;
    let system_cache = RedisSystemCache::connect(&settings.redis_url()?, settings.redis.key_prefix.clone()).await?;
    let system: Arc<dyn SystemUseCase> = Arc::new(SystemService::with_cache(StorageSystemRepository::new(database.clone()), system_cache));
    let metrics: Arc<dyn ServerMetricsUseCase> = Arc::new(SystemMetricsService::new(SysinfoServerMetricsCollector));
    rebuild_system_cache(&system).await?;
    let runtime_config = RuntimeUserConfig::new(system.clone());
    let online_sessions = RedisOnlineSessionStore::connect(&settings.redis_url()?, settings.redis.key_prefix.clone()).await?;
    let users: Arc<dyn UserUseCase> = Arc::new(UserService::with_password_policy(
        StorageUserRepository::new(database.clone()),
        Argon2PasswordHasher,
        runtime_config.clone(),
    ));
    let tokens = TokenService::with_ttl_reader(token_settings(settings)?, Arc::new(runtime_config), Arc::new(online_sessions));
    let captcha = build_captcha_service(settings, system.clone()).await?;
    let scheduler = build_scheduler_services(settings, database.clone(), system.clone())?;

    Ok(AppState {
        users,
        tokens,
        rbac: rbac.use_case,
        rbac_admin: rbac.admin,
        system,
        metrics,
        captcha,
        scheduler: scheduler.use_case,
        scheduler_export_config: scheduler.export_config,
        scheduler_runtime: scheduler.runtime,
        authorization,
    })
}

pub async fn build_router(settings: &Settings, metrics_handle: hook_tracing::MetricsHandle) -> BackendResult<Router> {
    let state = build_app_state(settings).await?;
    create_app(state, settings, metrics_handle)
}

#[cfg(test)]
pub(crate) fn build_public_router(settings: &Settings, metrics_handle: hook_tracing::MetricsHandle) -> BackendResult<Router> {
    let app = attach_metrics(public_routes(settings), metrics_handle);
    apply_http_layers(app, settings)
}

pub fn create_app(state: AppState, settings: &Settings, metrics_handle: hook_tracing::MetricsHandle) -> BackendResult<Router> {
    let user_config = Arc::new(RuntimeUserConfig::new(state.system.clone()));
    let rbac_config = Arc::new(RuntimeRbacConfig::new(state.system.clone()));
    let system_config = Arc::new(RuntimeSystemConfig::new(state.system.clone()));
    let account_verifier = Arc::new(CaptchaAccountVerifier::new(state.captcha.clone()));
    let avatar_storage = Arc::new(LocalAvatarStorage::new(settings.uploads.avatar_directory.clone(), AVATAR_URL_PREFIX));
    let user_state = ApiState::new(ApiStateParts {
        users: state.users.clone(),
        tokens: state.tokens.clone(),
        rbac: state.rbac.clone(),
        config: user_config.clone(),
        account_verifier,
        public_ip_resolver: Arc::new(PublicIpAddressResolver),
        ip_location_resolver: Arc::new(PconlineIpLocationResolver::new(user_config.clone())),
    })
    .with_avatar_storage(avatar_storage)
    .with_avatar_config(user_config.clone())
    .with_export_config(user_config.clone());
    let rbac_state = RbacApiState::new(state.rbac.clone(), state.rbac_admin.clone()).with_export_config(rbac_config);
    let system_state = SystemApiState::new(SystemApiStateParts {
        system: state.system.clone(),
        metrics: state.metrics.clone(),
        rbac: state.rbac.clone(),
        rbac_admin: state.rbac_admin.clone(),
    })
    .with_export_config(system_config);
    let captcha_state = CaptchaApiState::new(state.captcha.clone());
    let auth_state = AuthState::new(AuthStateParts {
        users: state.users,
        tokens: state.tokens,
        rbac: state.rbac,
        authorization: state.authorization,
    });

    let api_router = Router::new()
        .merge(create_user_router(user_state))
        .merge(create_rbac_router(rbac_state))
        .merge(create_system_router(system_state))
        .merge(create_captcha_router(captcha_state))
        .merge(create_scheduler_router(SchedulerApiState::new(SchedulerApiStateParts {
            scheduler: state.scheduler,
            export_config: state.scheduler_export_config,
            runtime: state.scheduler_runtime,
        })));

    let app = public_routes(settings).nest("/api", api_router);
    let app = attach_metrics(app, metrics_handle);
    let app = app.layer(middleware::from_fn_with_state(auth_state, auth_middleware));
    let app = app.layer(middleware::from_fn(types::http::locale_middleware));

    apply_http_layers(app, settings)
}

pub(crate) fn apply_http_layers(app: Router, settings: &Settings) -> BackendResult<Router> {
    Ok(app
        .layer(http_config::timeout_layer(settings)?)
        .layer(http_config::compression_layer(settings)?)
        .layer(http_config::cors_layer(settings)?)
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid)))
}

fn public_routes(settings: &Settings) -> Router {
    docs::router()
        .merge(system::create_router())
        .nest_service(AVATAR_URL_PREFIX, ServeDir::new(&settings.uploads.avatar_directory))
}

fn attach_metrics(mut app: Router, metrics_handle: hook_tracing::MetricsHandle) -> Router {
    if let Some(handle) = metrics_handle {
        app = app
            .route("/metrics", axum::routing::get(move || hook_tracing::metrics_handler(handle.clone())))
            .layer(middleware::from_fn(hook_tracing::metrics_middleware));
    }
    app
}

async fn build_rbac_services(settings: &Settings, database: Database) -> BackendResult<RbacServices> {
    let repository = StorageRbacRepository::new(database);
    let cache = RedisRbacCache::connect(&settings.redis_url()?, settings.redis.key_prefix.clone()).await?;
    let service = build_rbac_service(repository, cache).await?;

    let use_case: Arc<dyn RbacUseCase> = service.clone();
    let admin: Arc<dyn RbacAdminUseCase> = service;
    Ok(RbacServices { use_case, admin })
}

pub async fn rebuild_rbac_cache(settings: &Settings, database: Database) -> BackendResult<()> {
    let repository = StorageRbacRepository::new(database);
    let cache = RedisRbacCache::connect(&settings.redis_url()?, settings.redis.key_prefix.clone()).await?;
    let service = build_rbac_service(repository, cache).await?;
    service.rebuild_cache().await?;
    Ok(())
}

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
    let providers: Vec<Arc<dyn CaptchaProvider>> = vec![
        Arc::new(CapProvider::new(store)),
        Arc::new(CloudflareTurnstileProvider::new(ReqwestTurnstileVerifier::new())),
    ];
    Ok(Arc::new(CaptchaService::new(CaptchaSystemConfig::new(system), providers)))
}

fn token_settings(settings: &Settings) -> BackendResult<TokenSettings> {
    Ok(TokenSettings {
        secret: settings.jwt_secret()?,
    })
}

async fn build_rbac_service(
    repository: StorageRbacRepository,
    cache: RedisRbacCache,
) -> BackendResult<Arc<RbacService<StorageRbacRepository, RedisRbacCache>>> {
    let service = Arc::new(RbacService::new(repository, cache));
    service.rebuild_cache().await?;
    Ok(service)
}
