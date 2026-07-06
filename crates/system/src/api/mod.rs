mod error;
mod export;
mod handlers;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, put},
};

use rbac::application::{RbacAdminUseCase, RbacUseCase};

use crate::application::SystemUseCase;

use self::handlers::{
    config_by_key, create_config, create_dept, create_dict_data, create_dict_type, create_post, delete_config, delete_configs, delete_dept, delete_dict_data,
    delete_dict_data_batch, delete_dict_type, delete_dict_types, delete_post, delete_posts, dept_tree_select, dict_data_by_type, dict_type_options,
    exclude_dept_tree, export_configs, export_dict_data, export_dict_types, export_posts, get_config, get_dept, get_dict_data, get_dict_type, get_post,
    list_configs, list_depts, list_dict_data, list_dict_types, list_posts, post_options, public_configs, refresh_config_cache, refresh_dict_cache,
    replace_config, replace_dept, replace_dict_data, replace_dict_type, replace_post, role_dept_tree_select, update_dept_sort, update_dept_sorts,
};

pub use error::SystemApiError;

#[derive(Clone)]
pub struct SystemApiState {
    pub system: Arc<dyn SystemUseCase>,
    pub rbac: Arc<dyn RbacUseCase>,
    pub rbac_admin: Arc<dyn RbacAdminUseCase>,
}

impl SystemApiState {
    pub fn new(system: Arc<dyn SystemUseCase>, rbac: Arc<dyn RbacUseCase>, rbac_admin: Arc<dyn RbacAdminUseCase>) -> Self {
        Self { system, rbac, rbac_admin }
    }
}

pub fn create_router(state: SystemApiState) -> Router {
    Router::new()
        .route("/system/depts", get(list_depts).post(create_dept))
        .route("/system/depts/tree-select", get(dept_tree_select))
        .route("/system/depts/exclude/{id}", get(exclude_dept_tree))
        .route("/system/depts/sort", put(update_dept_sorts))
        .route("/system/depts/{id}", get(get_dept).put(replace_dept).delete(delete_dept))
        .route("/system/depts/{id}/sort", put(update_dept_sort))
        .route("/system/roles/{id}/dept-tree-select", get(role_dept_tree_select))
        .route("/system/posts", get(list_posts).post(create_post))
        .route("/system/posts/export", axum::routing::post(export_posts))
        .route("/system/posts/options", get(post_options))
        .route("/system/posts/batch", axum::routing::delete(delete_posts))
        .route("/system/posts/{id}", get(get_post).put(replace_post).delete(delete_post))
        .route("/system/dict-types", get(list_dict_types).post(create_dict_type))
        .route("/system/dict-types/export", axum::routing::post(export_dict_types))
        .route("/system/dict-types/options", get(dict_type_options))
        .route("/system/dict-types/cache", axum::routing::delete(refresh_dict_cache))
        .route("/system/dict-types/batch", axum::routing::delete(delete_dict_types))
        .route("/system/dict-types/{id}", get(get_dict_type).put(replace_dict_type).delete(delete_dict_type))
        .route("/system/dict-data", get(list_dict_data).post(create_dict_data))
        .route("/system/dict-data/export", axum::routing::post(export_dict_data))
        .route("/system/dict-data/type/{dict_type}", get(dict_data_by_type))
        .route("/system/dict-data/batch", axum::routing::delete(delete_dict_data_batch))
        .route("/system/dict-data/{id}", get(get_dict_data).put(replace_dict_data).delete(delete_dict_data))
        .route("/system/configs", get(list_configs).post(create_config))
        .route("/system/configs/export", axum::routing::post(export_configs))
        .route("/system/configs/cache", axum::routing::delete(refresh_config_cache))
        .route("/system/configs/batch", axum::routing::delete(delete_configs))
        .route("/system/configs/key/{key}", get(config_by_key))
        .route("/system/configs/{id}", get(get_config).put(replace_config).delete(delete_config))
        .route("/app/configs", get(public_configs))
        .with_state(state)
}
