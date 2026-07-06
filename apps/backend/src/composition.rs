use std::sync::Arc;

use ::system::{
    api::{SystemApiState, create_router as create_system_router},
    application::{SystemError, SystemService, SystemUseCase},
    infra::{RedisSystemCache, StorageSystemRepository},
};
use async_trait::async_trait;
use axum::{Router, middleware};
use captcha::{
    api::{CaptchaApiState, create_router as create_captcha_router},
    application::{CaptchaError, CaptchaProvider, CaptchaService, CaptchaSettingsReader, CaptchaUseCase},
    infra::RedisCaptchaStore,
    providers::{
        cap::CapProvider,
        cloudflare_turnstile::{CloudflareTurnstileProvider, ReqwestTurnstileVerifier},
    },
};
use configuration::Settings;
use constants::system_config::{AVATAR_CONFIG_KEY, CAPTCHA_CONFIG_KEY, EXPORT_BATCH_CONFIG_KEY, PASSWORD_POLICY_KEY, TOKEN_CONFIG_KEY};
use kernel::runtime_config::{ExportBatchConfig, ExportConfigProvider};
use rbac::{
    api::{RbacApiState, create_router as create_rbac_router},
    application::{AuthWhitelistRule, AuthorizationConfig, RbacAdminUseCase, RbacError, RbacService, RbacUseCase},
    domain::RoutePermissionRule,
    infra::{RedisRbacCache, StorageRbacRepository},
};
use serde_json::Value;
use storage::{Database, connect_database};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use user::{
    api::{ApiState, TokenService, TokenSettings, TokenSettingsReader, TokenTtlConfig, create_router as create_user_router, parse_token_ttl_config},
    application::{
        AccountVerifier, AppError, AppResult, AvatarConfig, AvatarConfigProvider, PasswordPolicy, PasswordPolicyProvider, SystemConfigProvider, UserService,
        UserUseCase, parse_avatar_config, parse_export_batch_config, parse_password_policy,
    },
    infra::{Argon2PasswordHasher, LocalAvatarStorage, StorageUserRepository},
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

const AVATAR_URL_PREFIX: &str = "/uploads/avatars";

struct CaptchaSystemConfig {
    system: Arc<dyn SystemUseCase>,
}

impl CaptchaSystemConfig {
    fn new(system: Arc<dyn SystemUseCase>) -> Self {
        Self { system }
    }
}

#[async_trait]
impl CaptchaSettingsReader for CaptchaSystemConfig {
    async fn config(&self) -> Result<Value, CaptchaError> {
        let value = self.system.config_by_key(CAPTCHA_CONFIG_KEY).await.map_err(captcha_config_error)?;
        serde_json::from_str(&value).map_err(captcha_json_error)
    }
}

#[derive(Clone)]
struct RuntimeUserConfig {
    system: Arc<dyn SystemUseCase>,
}

impl RuntimeUserConfig {
    fn new(system: Arc<dyn SystemUseCase>) -> Self {
        Self { system }
    }

    async fn user_config(&self, key: &str) -> AppResult<String> {
        self.system.config_by_key(key).await.map_err(user_config_error)
    }
}

#[async_trait]
impl SystemConfigProvider for RuntimeUserConfig {
    async fn config_by_key(&self, key: &str) -> Result<String, AppError> {
        self.user_config(key).await
    }
}

#[async_trait]
impl PasswordPolicyProvider for RuntimeUserConfig {
    async fn password_policy(&self) -> AppResult<PasswordPolicy> {
        parse_password_policy(&self.user_config(PASSWORD_POLICY_KEY).await?)
    }
}

#[async_trait]
impl AvatarConfigProvider for RuntimeUserConfig {
    async fn avatar_config(&self) -> AppResult<AvatarConfig> {
        parse_avatar_config(&self.user_config(AVATAR_CONFIG_KEY).await?)
    }
}

#[async_trait]
impl ExportConfigProvider for RuntimeUserConfig {
    type Error = AppError;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error> {
        parse_export_batch_config(&self.user_config(EXPORT_BATCH_CONFIG_KEY).await?)
    }
}

#[async_trait]
impl TokenSettingsReader for RuntimeUserConfig {
    async fn token_ttl_config(&self) -> AppResult<TokenTtlConfig> {
        parse_token_ttl_config(&self.user_config(TOKEN_CONFIG_KEY).await?)
    }
}

#[derive(Clone)]
struct RuntimeRbacConfig {
    system: Arc<dyn SystemUseCase>,
}

impl RuntimeRbacConfig {
    fn new(system: Arc<dyn SystemUseCase>) -> Self {
        Self { system }
    }
}

#[async_trait]
impl ExportConfigProvider for RuntimeRbacConfig {
    type Error = RbacError;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error> {
        let value = self.system.config_by_key(EXPORT_BATCH_CONFIG_KEY).await.map_err(rbac_config_error)?;
        parse_export_batch_config(&value).map_err(user_error_to_rbac)
    }
}

#[derive(Clone)]
struct RuntimeSystemConfig {
    system: Arc<dyn SystemUseCase>,
}

impl RuntimeSystemConfig {
    fn new(system: Arc<dyn SystemUseCase>) -> Self {
        Self { system }
    }
}

#[async_trait]
impl ExportConfigProvider for RuntimeSystemConfig {
    type Error = SystemError;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error> {
        let value = self.system.config_by_key(EXPORT_BATCH_CONFIG_KEY).await?;
        parse_export_batch_config(&value).map_err(user_error_to_system)
    }
}

struct CaptchaAccountVerifier {
    captcha: Arc<dyn CaptchaUseCase>,
}

impl CaptchaAccountVerifier {
    fn new(captcha: Arc<dyn CaptchaUseCase>) -> Self {
        Self { captcha }
    }
}

#[async_trait]
impl AccountVerifier for CaptchaAccountVerifier {
    async fn verify_account(&self, token: Option<&str>) -> AppResult<()> {
        self.captcha.verify_account(token).await.map_err(captcha_account_error)
    }
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
    rebuild_system_cache(&system).await?;
    let runtime_config = RuntimeUserConfig::new(system.clone());
    let users: Arc<dyn UserUseCase> = Arc::new(UserService::with_password_policy(
        StorageUserRepository::new(database.clone()),
        Argon2PasswordHasher,
        runtime_config.clone(),
    ));
    let tokens = TokenService::with_ttl_reader(token_settings(settings)?, Arc::new(runtime_config));
    let captcha = build_captcha_service(settings, system.clone()).await?;

    Ok(AppState {
        users,
        tokens,
        rbac: rbac.use_case,
        rbac_admin: rbac.admin,
        system,
        captcha,
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
    let user_state = ApiState::new(
        state.users.clone(),
        state.tokens.clone(),
        state.rbac.clone(),
        user_config.clone(),
        account_verifier,
    )
    .with_avatar_storage(avatar_storage)
    .with_avatar_config(user_config.clone())
    .with_export_config(user_config.clone());
    let rbac_state = RbacApiState::new(state.rbac.clone(), state.rbac_admin.clone()).with_export_config(rbac_config);
    let system_state = SystemApiState::new(state.system.clone(), state.rbac.clone(), state.rbac_admin.clone()).with_export_config(system_config);
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
        .merge(create_captcha_router(captcha_state));

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

fn authorization_config(settings: &Settings) -> AuthorizationConfig {
    AuthorizationConfig {
        whitelist: auth_whitelist(settings),
        route_permissions: route_permissions(),
    }
}

fn auth_whitelist(settings: &Settings) -> Vec<AuthWhitelistRule> {
    let mut rules = settings
        .auth
        .whitelist
        .iter()
        .map(|rule| AuthWhitelistRule {
            methods: rule.methods.clone(),
            path_pattern: rule.path_pattern.clone(),
        })
        .collect::<Vec<_>>();
    ensure_auth_whitelist_rule(&mut rules, "GET", "/api/app/configs");
    ensure_auth_whitelist_rule(&mut rules, "GET", "/api/auth/me");
    ensure_auth_whitelist_rule(&mut rules, "GET", "/uploads/avatars/{*file}");
    ensure_auth_whitelist_rule(&mut rules, "GET", "/api/captcha/config");
    ensure_auth_whitelist_rule(&mut rules, "POST", "/api/captcha/challenge");
    ensure_auth_whitelist_rule(&mut rules, "POST", "/api/captcha/redeem");
    rules
}

fn ensure_auth_whitelist_rule(rules: &mut Vec<AuthWhitelistRule>, method: &str, path_pattern: &str) {
    let exists = rules
        .iter()
        .any(|rule| rule.path_pattern == path_pattern && rule.methods.iter().any(|item| item.eq_ignore_ascii_case(method)));
    if !exists {
        rules.push(AuthWhitelistRule {
            methods: vec![method.into()],
            path_pattern: path_pattern.into(),
        });
    }
}

fn route_permissions() -> Vec<RoutePermissionRule> {
    vec![
        route_rule(&["GET"], "/api/system/users", "system:user:list", "list_users"),
        route_rule(&["POST"], "/api/system/users", "system:user:add", "create_user"),
        route_rule(&["POST"], "/api/system/users/export", "system:user:export", "export_users"),
        route_rule(&["POST"], "/api/system/users/import", "system:user:import", "import_users"),
        route_rule(&["POST"], "/api/system/users/import-template", "system:user:import", "user_import_template"),
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
        route_rule(&["POST"], "/api/system/roles/export", "system:role:export", "export_roles"),
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
        route_rule(&["POST"], "/api/system/posts/export", "system:post:export", "export_posts"),
        route_rule(&["GET"], "/api/system/posts/options", "system:post:list", "post_options"),
        route_rule(&["GET"], "/api/system/posts/{id}", "system:post:query", "get_post"),
        route_rule(&["PUT"], "/api/system/posts/{id}", "system:post:edit", "replace_post"),
        route_rule(&["DELETE"], "/api/system/posts/{id}", "system:post:remove", "delete_post"),
        route_rule(&["DELETE"], "/api/system/posts/batch", "system:post:remove", "delete_posts"),
        route_rule(&["GET"], "/api/system/dict-types", "system:dict:list", "list_dict_types"),
        route_rule(&["POST"], "/api/system/dict-types", "system:dict:add", "create_dict_type"),
        route_rule(&["POST"], "/api/system/dict-types/export", "system:dict:export", "export_dict_types"),
        route_rule(&["GET"], "/api/system/dict-types/options", "system:dict:list", "dict_type_options"),
        route_rule(&["DELETE"], "/api/system/dict-types/cache", "system:dict:remove", "refresh_dict_cache"),
        route_rule(&["GET"], "/api/system/dict-types/{id}", "system:dict:query", "get_dict_type"),
        route_rule(&["PUT"], "/api/system/dict-types/{id}", "system:dict:edit", "replace_dict_type"),
        route_rule(&["DELETE"], "/api/system/dict-types/{id}", "system:dict:remove", "delete_dict_type"),
        route_rule(&["DELETE"], "/api/system/dict-types/batch", "system:dict:remove", "delete_dict_types"),
        route_rule(&["GET"], "/api/system/dict-data", "system:dict:list", "list_dict_data"),
        route_rule(&["POST"], "/api/system/dict-data", "system:dict:add", "create_dict_data"),
        route_rule(&["POST"], "/api/system/dict-data/export", "system:dict:export", "export_dict_data"),
        route_rule(&["GET"], "/api/system/dict-data/type/{dict_type}", "system:dict:list", "dict_data_by_type"),
        route_rule(&["GET"], "/api/system/dict-data/{id}", "system:dict:query", "get_dict_data"),
        route_rule(&["PUT"], "/api/system/dict-data/{id}", "system:dict:edit", "replace_dict_data"),
        route_rule(&["DELETE"], "/api/system/dict-data/{id}", "system:dict:remove", "delete_dict_data"),
        route_rule(&["DELETE"], "/api/system/dict-data/batch", "system:dict:remove", "delete_dict_data_batch"),
        route_rule(&["GET"], "/api/system/configs", "system:config:list", "list_configs"),
        route_rule(&["POST"], "/api/system/configs", "system:config:add", "create_config"),
        route_rule(&["POST"], "/api/system/configs/export", "system:config:export", "export_configs"),
        route_rule(&["DELETE"], "/api/system/configs/cache", "system:config:remove", "refresh_config_cache"),
        route_rule(&["GET"], "/api/system/configs/key/{key}", "system:config:query", "config_by_key"),
        route_rule(&["GET"], "/api/system/configs/{id}", "system:config:query", "get_config"),
        route_rule(&["PUT"], "/api/system/configs/{id}", "system:config:edit", "replace_config"),
        route_rule(&["DELETE"], "/api/system/configs/{id}", "system:config:remove", "delete_config"),
        route_rule(&["DELETE"], "/api/system/configs/batch", "system:config:remove", "delete_configs"),
    ]
}

fn data_scope_handlers() -> Vec<&'static str> {
    vec![
        "list_users",
        "export_users",
        "list_roles",
        "export_roles",
        "role_users",
        "list_depts",
        "dept_tree_select",
    ]
}

async fn build_captcha_service(settings: &Settings, system: Arc<dyn SystemUseCase>) -> BackendResult<Arc<dyn CaptchaUseCase>> {
    let store = RedisCaptchaStore::connect(&settings.redis_url()?, settings.redis.key_prefix.clone()).await?;
    let providers: Vec<Arc<dyn CaptchaProvider>> = vec![
        Arc::new(CapProvider::new(store)),
        Arc::new(CloudflareTurnstileProvider::new(ReqwestTurnstileVerifier::new())),
    ];
    Ok(Arc::new(CaptchaService::new(CaptchaSystemConfig::new(system), providers)))
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
    })
}

fn captcha_json_error(error: serde_json::Error) -> CaptchaError {
    let _ = error;
    CaptchaError::InvalidInput(kernel::error::LocalizedError::new("errors.captcha.invalid_config_json").with_param("key", CAPTCHA_CONFIG_KEY))
}

fn captcha_config_error(error: SystemError) -> CaptchaError {
    match error {
        SystemError::NotFound => CaptchaError::Infrastructure("required captcha system config not found".into()),
        SystemError::Forbidden(message) | SystemError::Conflict(message) | SystemError::InvalidInput(message) => CaptchaError::InvalidInput(message),
        SystemError::Infrastructure(message) => CaptchaError::Infrastructure(message),
    }
}

fn user_config_error(error: SystemError) -> AppError {
    match error {
        SystemError::NotFound => AppError::Infrastructure("required system config not found".into()),
        SystemError::Forbidden(message) => AppError::Forbidden(message),
        SystemError::Conflict(message) => AppError::Conflict(message),
        SystemError::InvalidInput(message) => AppError::InvalidInput(message),
        SystemError::Infrastructure(message) => AppError::Infrastructure(message),
    }
}

fn rbac_config_error(error: SystemError) -> RbacError {
    match error {
        SystemError::NotFound => RbacError::Infrastructure("required system config not found".into()),
        SystemError::Forbidden(_) => RbacError::Forbidden,
        SystemError::Conflict(message) => RbacError::Conflict(message),
        SystemError::InvalidInput(message) => RbacError::InvalidInput(message),
        SystemError::Infrastructure(message) => RbacError::Infrastructure(message),
    }
}

fn user_error_to_rbac(error: AppError) -> RbacError {
    match error {
        AppError::InvalidInput(message) => RbacError::InvalidInput(message),
        AppError::Unauthorized => RbacError::Unauthorized,
        AppError::Forbidden(_) => RbacError::Forbidden,
        AppError::Conflict(message) => RbacError::Conflict(message),
        AppError::NotFound => RbacError::NotFound,
        AppError::Infrastructure(message) => RbacError::Infrastructure(message),
    }
}

fn user_error_to_system(error: AppError) -> SystemError {
    match error {
        AppError::InvalidInput(message) => SystemError::InvalidInput(message),
        AppError::Unauthorized => SystemError::Forbidden(kernel::error::LocalizedError::new("errors.common.forbidden")),
        AppError::Forbidden(message) => SystemError::Forbidden(message),
        AppError::Conflict(message) => SystemError::Conflict(message),
        AppError::NotFound => SystemError::NotFound,
        AppError::Infrastructure(message) => SystemError::Infrastructure(message),
    }
}

fn captcha_account_error(error: CaptchaError) -> AppError {
    match error {
        CaptchaError::InvalidInput(message) => AppError::InvalidInput(message),
        CaptchaError::Infrastructure(message) => AppError::Infrastructure(message),
    }
}

async fn build_rbac_service(
    repository: StorageRbacRepository,
    cache: RedisRbacCache,
) -> BackendResult<Arc<RbacService<StorageRbacRepository, RedisRbacCache>>> {
    let service = Arc::new(RbacService::new(repository, cache));
    service.rebuild_cache().await?;
    Ok(service)
}

#[cfg(test)]
mod tests {
    use super::{auth_whitelist, ensure_auth_whitelist_rule};
    use configuration::{
        AuthSettings, CorsSettings, DatabaseSettings, HttpSettings, JwtSettings, MetricsSettings, RedisSettings, ServerSettings, Settings, TracingFileSettings,
        TracingSettings, UploadSettings,
    };

    #[test]
    fn ensure_auth_whitelist_rule_adds_rule_once() {
        let mut rules = vec![];

        ensure_auth_whitelist_rule(&mut rules, "GET", "/api/auth/me");
        ensure_auth_whitelist_rule(&mut rules, "GET", "/api/auth/me");

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].methods, vec!["GET"]);
        assert_eq!(rules[0].path_pattern, "/api/auth/me");
    }

    #[test]
    fn auth_whitelist_includes_public_avatar_files() {
        let rules = auth_whitelist(&test_settings());

        assert!(
            rules
                .iter()
                .any(|rule| { rule.path_pattern == "/uploads/avatars/{*file}" && rule.methods.iter().any(|method| method == "GET") })
        );
    }

    fn test_settings() -> Settings {
        Settings {
            server: ServerSettings {
                host: "127.0.0.1".into(),
                port: 3000,
            },
            database: DatabaseSettings {
                auto_migrate: false,
                url: None,
                scheme: "postgres".into(),
                host: "localhost".into(),
                port: 5432,
                username: "postgres".into(),
                password: Some("postgres".into()),
                name: "postgres".into(),
            },
            jwt: JwtSettings { secret: "secret".into() },
            auth: AuthSettings { whitelist: vec![] },
            cors: CorsSettings {
                allowed_origins: vec!["*".into()],
                allowed_methods: vec!["*".into()],
                allowed_headers: vec!["*".into()],
                exposed_headers: vec!["*".into()],
                allow_credentials: false,
                max_age_seconds: None,
            },
            http: HttpSettings {
                request_timeout_ms: 30_000,
                compression_enabled: true,
            },
            metrics: MetricsSettings { enabled: true },
            redis: RedisSettings {
                url: None,
                scheme: "redis".into(),
                host: "localhost".into(),
                port: 6379,
                username: None,
                password: None,
                database: Some(0),
                protocol: Some("resp3".into()),
                key_prefix: "taco".into(),
            },
            uploads: UploadSettings::default(),
            tracing: TracingSettings {
                log_level: "info".into(),
                file: TracingFileSettings {
                    enabled: false,
                    directory: "logs".into(),
                    prefix: "taco.log".into(),
                },
            },
        }
    }
}
