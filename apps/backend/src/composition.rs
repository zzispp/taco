use std::sync::Arc;

use axum::{Router, middleware};
use configuration::Settings;
use rbac::{
    api::{RbacApiState, create_router as create_rbac_router},
    application::{AuthWhitelistRule, AuthorizationConfig, RbacAdminUseCase, RbacService, RbacUseCase},
    infra::{RedisRbacCache, StorageRbacRepository},
};
use storage::{Database, connect_database};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use user::{
    api::{ApiState, TokenService, TokenSettings, create_router as create_user_router},
    application::{UserService, UserUseCase},
    infra::{Argon2PasswordHasher, ConfigSystemUserProvider, StorageUserRepository},
};

use crate::{
    BackendResult,
    app_state::AppState,
    auth::{AuthState, AuthStateParts, auth_middleware},
    docs, http_config, migration, system,
};

struct RbacServices {
    use_case: Arc<dyn RbacUseCase>,
    admin: Arc<dyn RbacAdminUseCase>,
}

pub async fn build_app_state(settings: &Settings) -> BackendResult<AppState> {
    let database = connect_database(&settings.database_url()?).await?;
    migration::ensure_runtime_schema_ready(database.pool()).await?;
    let rbac = build_rbac_services(settings, database.clone()).await?;
    let users: Arc<dyn UserUseCase> = Arc::new(UserService::with_system_user(
        StorageUserRepository::new(database.clone()),
        Argon2PasswordHasher,
        ConfigSystemUserProvider::from_settings(settings)?,
    ));
    let tokens = TokenService::new(token_settings(settings)?);

    Ok(AppState {
        users,
        tokens,
        rbac: rbac.use_case,
        rbac_admin: rbac.admin,
        authorization: authorization_config(settings),
    })
}

pub async fn build_router(settings: &Settings, metrics_handle: hook_tracing::MetricsHandle) -> BackendResult<Router> {
    let state = build_app_state(settings).await?;
    create_app(state, settings, metrics_handle)
}

#[cfg(test)]
pub(crate) fn build_public_router(settings: &Settings, metrics_handle: hook_tracing::MetricsHandle) -> BackendResult<Router> {
    let app = attach_metrics(public_routes(), metrics_handle);
    apply_http_layers(app, settings)
}

pub fn create_app(state: AppState, settings: &Settings, metrics_handle: hook_tracing::MetricsHandle) -> BackendResult<Router> {
    let user_state = ApiState::new(state.users.clone(), state.tokens.clone());
    let rbac_state = RbacApiState::new(state.rbac.clone(), state.rbac_admin.clone());
    let auth_state = AuthState::new(AuthStateParts {
        users: state.users,
        tokens: state.tokens,
        rbac: state.rbac,
        authorization: state.authorization,
    });

    let api_router = Router::new().merge(create_user_router(user_state)).merge(create_rbac_router(rbac_state));

    let app = public_routes().nest("/api", api_router);
    let app = attach_metrics(app, metrics_handle);
    let app = app.layer(middleware::from_fn_with_state(auth_state, auth_middleware));

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

fn public_routes() -> Router {
    docs::router().merge(system::create_router())
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

fn authorization_config(settings: &Settings) -> AuthorizationConfig {
    AuthorizationConfig {
        whitelist: settings
            .auth
            .whitelist
            .iter()
            .map(|rule| AuthWhitelistRule {
                methods: rule.methods.clone(),
                path_pattern: rule.path_pattern.clone(),
            })
            .collect(),
    }
}

fn token_settings(settings: &Settings) -> BackendResult<TokenSettings> {
    Ok(TokenSettings {
        secret: settings.jwt_secret()?,
        access_token_ttl_seconds: settings.jwt.access_token_ttl_seconds,
        refresh_token_ttl_seconds: settings.jwt.refresh_token_ttl_seconds,
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
