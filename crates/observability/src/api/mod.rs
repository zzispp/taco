mod dto;
mod endpoints;
mod error;
mod export;
mod handlers;
mod input;
mod openapi;
mod presenter;
mod state;
mod support;

#[cfg(test)]
mod tests;

use axum::{
    Router,
    routing::{delete, get, post},
};

pub use endpoints::endpoint_specs;
pub use error::SystemLogApiError;
pub use openapi::SystemLogApiDoc;
pub use state::{SystemLogApiState, SystemLogApiStateParts};

pub fn create_router(state: SystemLogApiState) -> Router {
    use endpoints::{
        SYSTEM_LOG_DELETE, SYSTEM_LOGS_CLEAN, SYSTEM_LOGS_CLEAN_COUNT, SYSTEM_LOGS_CLEAN_EXECUTION, SYSTEM_LOGS_DELETE_BATCH, SYSTEM_LOGS_EXPORT,
        SYSTEM_LOGS_LIST,
    };

    Router::new()
        .route(SYSTEM_LOGS_LIST.api_route_path(), get(handlers::list_system_logs))
        .route(SYSTEM_LOGS_EXPORT.api_route_path(), post(handlers::export_system_logs))
        .route(SYSTEM_LOGS_CLEAN_COUNT.api_route_path(), get(handlers::count_system_logs_for_cleanup))
        .route(SYSTEM_LOGS_CLEAN.api_route_path(), delete(handlers::clean_system_logs))
        .route(SYSTEM_LOGS_CLEAN_EXECUTION.api_route_path(), get(handlers::get_system_log_cleanup_execution))
        .route(SYSTEM_LOGS_DELETE_BATCH.api_route_path(), delete(handlers::delete_system_logs))
        .route(
            SYSTEM_LOG_DELETE.api_route_path(),
            get(handlers::get_system_log).delete(handlers::delete_system_log),
        )
        .with_state(state)
}
