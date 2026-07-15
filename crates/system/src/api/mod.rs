mod dashboard;
mod endpoints;
mod error;
mod export;
mod handlers;
mod input;
pub(crate) mod operation_audit;

use std::sync::Arc;

use audit_contract::AuditOutboxRecorder;
use axum::{
    Router,
    routing::{get, put},
};
use kernel::{
    error::LocalizedError,
    runtime_config::{ExportBatchConfig, ExportConfigProvider},
};

use rbac::application::{RbacAdminUseCase, RbacUseCase};

use crate::application::{ServerMetricsUseCase, SystemAuditedUseCase, SystemError, SystemUseCase};

use self::{
    dashboard::get_server_dashboard,
    handlers::{
        config_by_key, create_config, create_dept, create_dict_data, create_dict_type, create_post, delete_config, delete_configs, delete_dept,
        delete_dict_data, delete_dict_data_batch, delete_dict_type, delete_dict_types, delete_post, delete_posts, dept_tree_select, dict_data_by_type,
        dict_type_options, exclude_dept_tree, export_configs, export_dict_data, export_dict_types, export_posts, get_config, get_dept, get_dict_data,
        get_dict_type, get_post, list_configs, list_depts, list_dict_data, list_dict_types, list_posts, post_options, public_configs, refresh_config_cache,
        refresh_dict_cache, replace_config, replace_dept, replace_dict_data, replace_dict_type, replace_post, role_dept_tree_select, update_dept_sort,
        update_dept_sorts,
    },
};

pub use endpoints::endpoint_specs;
pub use error::SystemApiError;

#[derive(Clone)]
pub struct SystemApiState {
    pub system: Arc<dyn SystemUseCase>,
    pub system_audited: Arc<dyn SystemAuditedUseCase>,
    pub operation_audit: Arc<dyn AuditOutboxRecorder>,
    pub metrics: Arc<dyn ServerMetricsUseCase>,
    pub rbac: Arc<dyn RbacUseCase>,
    pub rbac_admin: Arc<dyn RbacAdminUseCase>,
    pub export_config: Arc<dyn ExportConfigProvider<Error = SystemError>>,
}

pub struct SystemApiStateParts {
    pub system: Arc<dyn SystemUseCase>,
    pub system_audited: Arc<dyn SystemAuditedUseCase>,
    pub operation_audit: Arc<dyn AuditOutboxRecorder>,
    pub metrics: Arc<dyn ServerMetricsUseCase>,
    pub rbac: Arc<dyn RbacUseCase>,
    pub rbac_admin: Arc<dyn RbacAdminUseCase>,
}

impl SystemApiState {
    pub fn new(parts: SystemApiStateParts) -> Self {
        Self {
            system: parts.system,
            system_audited: parts.system_audited,
            operation_audit: parts.operation_audit,
            metrics: parts.metrics,
            rbac: parts.rbac,
            rbac_admin: parts.rbac_admin,
            export_config: Arc::new(DisabledExportConfigProvider),
        }
    }

    pub fn with_export_config(mut self, export_config: Arc<dyn ExportConfigProvider<Error = SystemError>>) -> Self {
        self.export_config = export_config;
        self
    }
}

struct DisabledExportConfigProvider;

#[async_trait::async_trait]
impl ExportConfigProvider for DisabledExportConfigProvider {
    type Error = SystemError;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error> {
        Err(SystemError::InvalidInput(LocalizedError::new("errors.system.export_config_unconfigured")))
    }
}

pub fn create_router(state: SystemApiState) -> Router {
    structure_routes().merge(data_routes()).with_state(state)
}

fn structure_routes() -> Router<SystemApiState> {
    use self::endpoints::structure::{
        DASHBOARD, DEPT_REPLACE, DEPT_SORT, DEPTS_CREATE, DEPTS_EXCLUDE, DEPTS_SORT, DEPTS_TREE_SELECT, POST_REPLACE, POSTS_CREATE, POSTS_DELETE_BATCH,
        POSTS_EXPORT, POSTS_OPTIONS, ROLE_DEPT_TREE,
    };

    Router::new()
        .route(DASHBOARD.api_route_path(), get(get_server_dashboard))
        .route(DEPTS_CREATE.api_route_path(), get(list_depts).post(create_dept))
        .route(DEPTS_TREE_SELECT.api_route_path(), get(dept_tree_select))
        .route(DEPTS_EXCLUDE.api_route_path(), get(exclude_dept_tree))
        .route(DEPTS_SORT.api_route_path(), put(update_dept_sorts))
        .route(DEPT_REPLACE.api_route_path(), get(get_dept).put(replace_dept).delete(delete_dept))
        .route(DEPT_SORT.api_route_path(), put(update_dept_sort))
        .route(ROLE_DEPT_TREE.api_route_path(), get(role_dept_tree_select))
        .route(POSTS_CREATE.api_route_path(), get(list_posts).post(create_post))
        .route(POSTS_EXPORT.api_route_path(), axum::routing::post(export_posts))
        .route(POSTS_OPTIONS.api_route_path(), get(post_options))
        .route(POSTS_DELETE_BATCH.api_route_path(), axum::routing::delete(delete_posts))
        .route(POST_REPLACE.api_route_path(), get(get_post).put(replace_post).delete(delete_post))
}

fn data_routes() -> Router<SystemApiState> {
    use self::endpoints::data::{
        CONFIG_BY_KEY, CONFIG_REPLACE, CONFIGS_CACHE, CONFIGS_CREATE, CONFIGS_DELETE_BATCH, CONFIGS_EXPORT, DICT_DATA_BY_TYPE, DICT_DATA_CREATE,
        DICT_DATA_DELETE_BATCH, DICT_DATA_EXPORT, DICT_DATA_REPLACE, DICT_TYPE_REPLACE, DICT_TYPES_CACHE, DICT_TYPES_CREATE, DICT_TYPES_DELETE_BATCH,
        DICT_TYPES_EXPORT, DICT_TYPES_OPTIONS, PUBLIC_CONFIGS,
    };

    Router::new()
        .route(DICT_TYPES_CREATE.api_route_path(), get(list_dict_types).post(create_dict_type))
        .route(DICT_TYPES_EXPORT.api_route_path(), axum::routing::post(export_dict_types))
        .route(DICT_TYPES_OPTIONS.api_route_path(), get(dict_type_options))
        .route(DICT_TYPES_CACHE.api_route_path(), axum::routing::delete(refresh_dict_cache))
        .route(DICT_TYPES_DELETE_BATCH.api_route_path(), axum::routing::delete(delete_dict_types))
        .route(
            DICT_TYPE_REPLACE.api_route_path(),
            get(get_dict_type).put(replace_dict_type).delete(delete_dict_type),
        )
        .route(DICT_DATA_CREATE.api_route_path(), get(list_dict_data).post(create_dict_data))
        .route(DICT_DATA_EXPORT.api_route_path(), axum::routing::post(export_dict_data))
        .route(DICT_DATA_BY_TYPE.api_route_path(), get(dict_data_by_type))
        .route(DICT_DATA_DELETE_BATCH.api_route_path(), axum::routing::delete(delete_dict_data_batch))
        .route(
            DICT_DATA_REPLACE.api_route_path(),
            get(get_dict_data).put(replace_dict_data).delete(delete_dict_data),
        )
        .route(CONFIGS_CREATE.api_route_path(), get(list_configs).post(create_config))
        .route(CONFIGS_EXPORT.api_route_path(), axum::routing::post(export_configs))
        .route(CONFIGS_CACHE.api_route_path(), axum::routing::delete(refresh_config_cache))
        .route(CONFIGS_DELETE_BATCH.api_route_path(), axum::routing::delete(delete_configs))
        .route(CONFIG_BY_KEY.api_route_path(), get(config_by_key))
        .route(CONFIG_REPLACE.api_route_path(), get(get_config).put(replace_config).delete(delete_config))
        .route(PUBLIC_CONFIGS.api_route_path(), get(public_configs))
}
