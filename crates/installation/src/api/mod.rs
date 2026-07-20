mod dto;
mod error;
mod handlers;
mod state;

use axum::{
    Router,
    http::StatusCode,
    routing::{any, get, post},
};
use std::sync::Arc;

use crate::application::{InstallationStatus, SetupUseCase};

use self::{
    handlers::{get_setup_defaults, get_setup_status, get_status, install, test_postgres, test_redis},
    state::{InstallationApiState, SetupApiState},
};

pub const SETUP_STATUS_PATH: &str = "/api/setup/status";
pub const SETUP_DEFAULTS_PATH: &str = "/api/setup/defaults";
pub const SETUP_POSTGRES_TEST_PATH: &str = "/api/setup/postgres/test";
pub const SETUP_REDIS_TEST_PATH: &str = "/api/setup/redis/test";
pub const SETUP_INSTALL_PATH: &str = "/api/setup/install";

pub fn setup_router() -> Router {
    status_router(InstallationStatus::setup())
}

/// Builds the complete public setup router. It is intentionally available only
/// to setup-mode startup wiring; normal mode uses [`installed_router`].
pub fn setup_router_with_state(setup: Arc<dyn SetupUseCase>) -> Router {
    Router::new()
        .route(SETUP_STATUS_PATH, get(get_setup_status))
        .route(SETUP_DEFAULTS_PATH, get(get_setup_defaults))
        .route(SETUP_POSTGRES_TEST_PATH, post(test_postgres))
        .route(SETUP_REDIS_TEST_PATH, post(test_redis))
        .route(SETUP_INSTALL_PATH, post(install))
        .with_state(SetupApiState::new(setup))
}

pub fn installed_router() -> Router {
    status_router(InstallationStatus::installed())
        .route(SETUP_DEFAULTS_PATH, any(setup_route_not_found))
        .route(SETUP_POSTGRES_TEST_PATH, any(setup_route_not_found))
        .route(SETUP_REDIS_TEST_PATH, any(setup_route_not_found))
        .route(SETUP_INSTALL_PATH, any(setup_route_not_found))
}

fn status_router(status: InstallationStatus) -> Router {
    Router::new()
        .route(SETUP_STATUS_PATH, get(get_status))
        .with_state(InstallationApiState::new(status))
}

async fn setup_route_not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}

#[cfg(test)]
mod tests;
