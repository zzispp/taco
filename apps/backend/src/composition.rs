use std::sync::Arc;

use ::system::{
    api::{SystemApiState, create_router as create_system_router},
    application::{SystemService, SystemUseCase},
    infra::{RedisSystemCache, StorageSystemRepository},
};
use axum::{Router, middleware};
use configuration::Settings;
use rbac::{
    api::{RbacApiState, create_router as create_rbac_router},
    application::{AuthWhitelistRule, AuthorizationConfig, RbacAdminUseCase, RbacService, RbacUseCase},
    domain::RoutePermissionRule,
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
    infra::{Argon2PasswordHasher, StorageUserRepository},
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
    migration::prepare_runtime_schema(database.pool(), settings.database.auto_migrate).await?;
    let rbac = build_rbac_services(settings, database.clone()).await?;
    let authorization = authorization_config(settings);
    rbac.use_case.validate_protected_handlers(&authorization)?;
    rbac.use_case.validate_data_scope_handlers(&data_scope_handlers())?;
    let users: Arc<dyn UserUseCase> = Arc::new(UserService::new(StorageUserRepository::new(database.clone()), Argon2PasswordHasher));
    let tokens = TokenService::new(token_settings(settings)?);
    let system_cache = RedisSystemCache::connect(&settings.redis_url()?, settings.redis.key_prefix.clone()).await?;
    let system: Arc<dyn SystemUseCase> = Arc::new(SystemService::with_cache(StorageSystemRepository::new(database), system_cache));

    Ok(AppState {
        users,
        tokens,
        rbac: rbac.use_case,
        rbac_admin: rbac.admin,
        system,
        authorization,
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
    let user_state = ApiState::new(state.users.clone(), state.tokens.clone(), state.rbac.clone());
    let rbac_state = RbacApiState::new(state.rbac.clone(), state.rbac_admin.clone());
    let system_state = SystemApiState::new(state.system, state.rbac.clone(), state.rbac_admin.clone());
    let auth_state = AuthState::new(AuthStateParts {
        users: state.users,
        tokens: state.tokens,
        rbac: state.rbac,
        authorization: state.authorization,
    });

    let api_router = Router::new()
        .merge(create_user_router(user_state))
        .merge(create_rbac_router(rbac_state))
        .merge(create_system_router(system_state));

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
        route_permissions: route_permissions(),
    }
}

fn route_permissions() -> Vec<RoutePermissionRule> {
    vec![
        route_rule(&["GET"], "/api/system/users", "system:user:list", "list_users"),
        route_rule(&["POST"], "/api/system/users", "system:user:add", "create_user"),
        route_rule(&["GET"], "/api/system/users/dept-tree", "system:user:list", "user_dept_tree"),
        route_rule(&["GET"], "/api/system/users/form-options", "system:user:list", "user_form_options"),
        route_rule(&["GET"], "/api/system/users/{id}", "system:user:query", "get_user"),
        route_rule(&["PUT"], "/api/system/users/{id}", "system:user:edit", "replace_user"),
        route_rule(&["DELETE"], "/api/system/users/{id}", "system:user:remove", "delete_user"),
        route_rule(&["DELETE"], "/api/system/users/batch", "system:user:remove", "delete_users"),
        route_rule(&["PUT"], "/api/system/users/{id}/password", "system:user:resetPwd", "reset_user_password"),
        route_rule(&["PUT"], "/api/system/users/{id}/status", "system:user:edit", "update_user_status"),
        route_rule(&["GET"], "/api/system/users/{id}/roles", "system:user:query", "user_roles"),
        route_rule(&["PUT"], "/api/system/users/{id}/roles", "system:user:edit", "replace_user_roles"),
        route_rule(&["GET"], "/api/system/roles", "system:role:list", "list_roles"),
        route_rule(&["POST"], "/api/system/roles", "system:role:add", "create_role"),
        route_rule(&["GET"], "/api/system/roles/options", "system:role:list", "role_options"),
        route_rule(&["GET"], "/api/system/roles/{id}", "system:role:query", "get_role"),
        route_rule(&["PUT"], "/api/system/roles/{id}", "system:role:edit", "replace_role"),
        route_rule(&["DELETE"], "/api/system/roles/{id}", "system:role:remove", "delete_role"),
        route_rule(&["DELETE"], "/api/system/roles/batch", "system:role:remove", "delete_roles"),
        route_rule(&["PUT"], "/api/system/roles/{id}/status", "system:role:edit", "update_role_status"),
        route_rule(&["PUT"], "/api/system/roles/{id}/data-scope", "system:role:edit", "update_role_data_scope"),
        route_rule(&["GET"], "/api/system/roles/{id}/menus", "system:role:query", "role_menu_bindings"),
        route_rule(&["PUT"], "/api/system/roles/{id}/menus", "system:role:edit", "replace_role_menus"),
        route_rule(&["GET"], "/api/system/roles/{id}/depts", "system:role:query", "role_dept_bindings"),
        route_rule(&["PUT"], "/api/system/roles/{id}/depts", "system:role:edit", "replace_role_depts"),
        route_rule(&["GET"], "/api/system/roles/{id}/users", "system:role:list", "role_users"),
        route_rule(&["PUT"], "/api/system/roles/{id}/users", "system:role:edit", "replace_role_users"),
        route_rule(&["DELETE"], "/api/system/roles/{id}/users/batch", "system:role:remove", "delete_role_users"),
        route_rule(&["DELETE"], "/api/system/roles/{id}/users/{user_id}", "system:role:remove", "delete_role_user"),
        route_rule(&["GET"], "/api/system/menus", "system:menu:list", "list_menus"),
        route_rule(&["POST"], "/api/system/menus", "system:menu:add", "create_menu"),
        route_rule(&["GET"], "/api/system/menus/tree", "system:menu:list", "list_menu_tree"),
        route_rule(&["GET"], "/api/system/menus/tree-select", "system:menu:list", "menu_tree_select"),
        route_rule(
            &["GET"],
            "/api/system/menus/role-tree-select/{id}",
            "system:role:query",
            "role_menu_tree_select",
        ),
        route_rule(&["GET"], "/api/system/menus/{id}", "system:menu:query", "get_menu"),
        route_rule(&["PUT"], "/api/system/menus/{id}", "system:menu:edit", "replace_menu"),
        route_rule(&["DELETE"], "/api/system/menus/{id}", "system:menu:remove", "delete_menu"),
        route_rule(&["PUT"], "/api/system/menus/{id}/sort", "system:menu:edit", "update_menu_sort"),
        route_rule(&["PUT"], "/api/system/menus/sort", "system:menu:edit", "update_menu_sorts"),
        route_rule(&["GET"], "/api/system/depts", "system:dept:list", "list_depts"),
        route_rule(&["POST"], "/api/system/depts", "system:dept:add", "create_dept"),
        route_rule(&["GET"], "/api/system/depts/tree-select", "system:dept:list", "dept_tree_select"),
        route_rule(&["GET"], "/api/system/depts/exclude/{id}", "system:dept:list", "exclude_dept_tree"),
        route_rule(&["GET"], "/api/system/depts/{id}", "system:dept:query", "get_dept"),
        route_rule(&["PUT"], "/api/system/depts/{id}", "system:dept:edit", "replace_dept"),
        route_rule(&["DELETE"], "/api/system/depts/{id}", "system:dept:remove", "delete_dept"),
        route_rule(&["PUT"], "/api/system/depts/{id}/sort", "system:dept:edit", "update_dept_sort"),
        route_rule(&["PUT"], "/api/system/depts/sort", "system:dept:edit", "update_dept_sorts"),
        route_rule(
            &["GET"],
            "/api/system/roles/{id}/dept-tree-select",
            "system:role:query",
            "role_dept_tree_select",
        ),
        route_rule(&["GET"], "/api/system/posts", "system:post:list", "list_posts"),
        route_rule(&["POST"], "/api/system/posts", "system:post:add", "create_post"),
        route_rule(&["GET"], "/api/system/posts/options", "system:post:list", "post_options"),
        route_rule(&["GET"], "/api/system/posts/{id}", "system:post:query", "get_post"),
        route_rule(&["PUT"], "/api/system/posts/{id}", "system:post:edit", "replace_post"),
        route_rule(&["DELETE"], "/api/system/posts/{id}", "system:post:remove", "delete_post"),
        route_rule(&["DELETE"], "/api/system/posts/batch", "system:post:remove", "delete_posts"),
        route_rule(&["GET"], "/api/system/dict-types", "system:dict:list", "list_dict_types"),
        route_rule(&["POST"], "/api/system/dict-types", "system:dict:add", "create_dict_type"),
        route_rule(&["GET"], "/api/system/dict-types/options", "system:dict:list", "dict_type_options"),
        route_rule(&["DELETE"], "/api/system/dict-types/cache", "system:dict:remove", "refresh_dict_cache"),
        route_rule(&["GET"], "/api/system/dict-types/{id}", "system:dict:query", "get_dict_type"),
        route_rule(&["PUT"], "/api/system/dict-types/{id}", "system:dict:edit", "replace_dict_type"),
        route_rule(&["DELETE"], "/api/system/dict-types/{id}", "system:dict:remove", "delete_dict_type"),
        route_rule(&["DELETE"], "/api/system/dict-types/batch", "system:dict:remove", "delete_dict_types"),
        route_rule(&["GET"], "/api/system/dict-data", "system:dict:list", "list_dict_data"),
        route_rule(&["POST"], "/api/system/dict-data", "system:dict:add", "create_dict_data"),
        route_rule(&["GET"], "/api/system/dict-data/type/{dict_type}", "system:dict:list", "dict_data_by_type"),
        route_rule(&["GET"], "/api/system/dict-data/{id}", "system:dict:query", "get_dict_data"),
        route_rule(&["PUT"], "/api/system/dict-data/{id}", "system:dict:edit", "replace_dict_data"),
        route_rule(&["DELETE"], "/api/system/dict-data/{id}", "system:dict:remove", "delete_dict_data"),
        route_rule(&["DELETE"], "/api/system/dict-data/batch", "system:dict:remove", "delete_dict_data_batch"),
        route_rule(&["GET"], "/api/system/configs", "system:config:list", "list_configs"),
        route_rule(&["POST"], "/api/system/configs", "system:config:add", "create_config"),
        route_rule(&["DELETE"], "/api/system/configs/cache", "system:config:remove", "refresh_config_cache"),
        route_rule(&["GET"], "/api/system/configs/key/{key}", "system:config:query", "config_by_key"),
        route_rule(&["GET"], "/api/system/configs/{id}", "system:config:query", "get_config"),
        route_rule(&["PUT"], "/api/system/configs/{id}", "system:config:edit", "replace_config"),
        route_rule(&["DELETE"], "/api/system/configs/{id}", "system:config:remove", "delete_config"),
        route_rule(&["DELETE"], "/api/system/configs/batch", "system:config:remove", "delete_configs"),
    ]
}

fn data_scope_handlers() -> Vec<&'static str> {
    vec!["list_users", "list_roles", "role_users", "list_depts", "dept_tree_select"]
}

fn route_rule(methods: &[&str], path_pattern: &str, permission: &str, handler: &'static str) -> RoutePermissionRule {
    RoutePermissionRule {
        methods: methods.iter().map(|method| (*method).into()).collect(),
        path_pattern: path_pattern.into(),
        permission: permission.into(),
        handler,
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
