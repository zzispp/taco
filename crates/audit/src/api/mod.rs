mod body_capture;
mod dto;
mod endpoint_catalog;
mod endpoints;
mod error;
mod export;
mod handlers;
mod input;
mod middleware;
#[cfg(test)]
mod middleware_tests;
mod openapi;
mod presenter;
mod sanitize;
mod state;

use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub use endpoints::endpoint_specs;
pub use error::AuditApiError;
pub use middleware::{AuditActorContext, OperationAuditState, operation_audit_middleware};
pub use openapi::AuditApiDoc;
pub use state::{AuditApiState, AuditApiStateParts};

pub fn create_router(state: AuditApiState) -> Router {
    use self::endpoints::{
        LOGIN_LOG_DELETE, LOGIN_LOGS_CLEAN, LOGIN_LOGS_DELETE_BATCH, LOGIN_LOGS_EXPORT, LOGIN_LOGS_LIST, LOGIN_UNLOCK, OPERATION_LOG_DELETE,
        OPERATION_LOGS_CLEAN, OPERATION_LOGS_DELETE_BATCH, OPERATION_LOGS_EXPORT, OPERATION_LOGS_LIST,
    };

    Router::new()
        .route(OPERATION_LOGS_LIST.api_route_path(), get(handlers::list_operation_logs))
        .route(OPERATION_LOGS_EXPORT.api_route_path(), post(handlers::export_operation_logs))
        .route(OPERATION_LOGS_CLEAN.api_route_path(), delete(handlers::clear_operation_logs))
        .route(OPERATION_LOGS_DELETE_BATCH.api_route_path(), delete(handlers::delete_operation_logs))
        .route(
            OPERATION_LOG_DELETE.api_route_path(),
            get(handlers::get_operation_log).delete(handlers::delete_operation_log),
        )
        .route(LOGIN_LOGS_LIST.api_route_path(), get(handlers::list_login_logs))
        .route(LOGIN_LOGS_EXPORT.api_route_path(), post(handlers::export_login_logs))
        .route(LOGIN_LOGS_CLEAN.api_route_path(), delete(handlers::clear_login_logs))
        .route(LOGIN_LOGS_DELETE_BATCH.api_route_path(), delete(handlers::delete_login_logs))
        .route(LOGIN_LOG_DELETE.api_route_path(), delete(handlers::delete_login_log))
        .route(LOGIN_UNLOCK.api_route_path(), put(handlers::unlock_login))
        .with_state(state)
}
