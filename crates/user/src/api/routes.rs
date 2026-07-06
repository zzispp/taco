use axum::{
    Router,
    routing::{get, post, put},
};

use crate::api::{
    ApiState,
    handlers::{
        account_profile, change_account_password, create_user, delete_user, delete_users, export_users, get_user, import_users, list_users, me, refresh,
        replace_user, replace_user_roles, reset_user_password, sign_in, sign_up, update_account_profile, update_user_status, upload_account_avatar,
        user_dept_tree, user_form_options, user_import_template, user_roles,
    },
};

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/auth/sign-up", post(sign_up))
        .route("/auth/sign-in", post(sign_in))
        .route("/auth/refresh", post(refresh))
        .route("/auth/me", get(me))
        .route("/account/profile", get(account_profile).put(update_account_profile))
        .route("/account/profile/password", put(change_account_password))
        .route("/account/profile/avatar", post(upload_account_avatar))
        .route("/system/users", get(list_users).post(create_user))
        .route("/system/users/export", axum::routing::post(export_users))
        .route("/system/users/import", axum::routing::post(import_users))
        .route("/system/users/import-template", axum::routing::post(user_import_template))
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
