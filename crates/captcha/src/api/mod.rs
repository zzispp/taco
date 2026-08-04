mod endpoints;
mod error;
mod handlers;
mod state;

use axum::{
    Router,
    routing::{get, post},
};

pub use endpoints::endpoint_specs;
pub use error::CaptchaApiError;
pub use state::CaptchaApiState;

use self::handlers::{challenge, config, redeem};

pub fn create_router(state: CaptchaApiState) -> Result<Router, audit_contract::EndpointSpecError> {
    use self::endpoints::{CHALLENGE, CONFIG, REDEEM};

    Ok(Router::new()
        .route(CONFIG.api_route_path()?, get(config))
        .route(CHALLENGE.api_route_path()?, post(challenge))
        .route(REDEEM.api_route_path()?, post(redeem))
        .with_state(state))
}
