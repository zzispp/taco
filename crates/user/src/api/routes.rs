use axum::{
    Router,
    routing::{get, post, put},
};

use crate::api::{
    ApiState,
    handlers::{
        create_user, delete_user, delete_users, get_user, list_users, me, refresh, replace_user, replace_user_roles, reset_user_password, sign_in, sign_up,
        update_user_status, user_dept_tree, user_form_options, user_roles,
    },
};

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/auth/sign-up", post(sign_up))
        .route("/auth/sign-in", post(sign_in))
        .route("/auth/refresh", post(refresh))
        .route("/auth/me", get(me))
        .route("/system/users", get(list_users).post(create_user))
        .route("/system/users/dept-tree", get(user_dept_tree))
        .route("/system/users/form-options", get(user_form_options))
        .route("/system/users/batch", axum::routing::delete(delete_users))
        .route("/system/users/{id}", get(get_user).put(replace_user).delete(delete_user))
        .route("/system/users/{id}/password", put(reset_user_password))
        .route("/system/users/{id}/status", put(update_user_status))
        .route("/system/users/{id}/roles", get(user_roles).put(replace_user_roles))
        .with_state(state)
}

#[cfg(test)]
mod tests;
