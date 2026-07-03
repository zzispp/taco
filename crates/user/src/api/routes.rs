use axum::{
    Router,
    routing::{get, post, put},
};

use crate::api::{
    ApiState,
    handlers::{create_user, delete_user, list_users, me, refresh, replace_user, sign_in, sign_up},
};

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/auth/sign-up", post(sign_up))
        .route("/auth/sign-in", post(sign_in))
        .route("/auth/refresh", post(refresh))
        .route("/auth/me", get(me))
        .route("/users", get(list_users).post(create_user))
        .route("/users/{id}", put(replace_user).delete(delete_user))
        .with_state(state)
}

#[cfg(test)]
mod tests;
