mod error;
mod handlers;
mod state;

use axum::{
    Router,
    routing::{get, post},
};

pub use error::CaptchaApiError;
pub use state::CaptchaApiState;

use self::handlers::{challenge, config, redeem};

pub fn create_router(state: CaptchaApiState) -> Router {
    Router::new()
        .route("/captcha/config", get(config))
        .route("/captcha/challenge", post(challenge))
        .route("/captcha/redeem", post(redeem))
        .with_state(state)
}
