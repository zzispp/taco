use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::api::{
    ApiState,
    endpoint_specs::{
        AUTH_LOGOUT, AUTH_ME, AUTH_REFRESH, AUTH_SIGN_IN, AUTH_SIGN_UP, ONLINE_FORCE_LOGOUT, ONLINE_LIST, PROFILE_AVATAR, PROFILE_GET, PROFILE_PASSWORD,
        USER_GET, USER_RESET_PASSWORD, USER_ROLES, USER_UPDATE_STATUS, USERS_DELETE_BATCH, USERS_DEPT_TREE, USERS_EXPORT, USERS_FORM_OPTIONS, USERS_IMPORT,
        USERS_IMPORT_TEMPLATE, USERS_LIST,
    },
    handlers::{
        account_profile, change_account_password, create_user, delete_user, delete_users, export_users, force_logout_online_session, get_user, import_users,
        list_online_sessions, list_users, logout, me, refresh, replace_user, replace_user_roles, reset_user_password, sign_in, sign_up, update_account_profile,
        update_user_status, upload_account_avatar, user_dept_tree, user_form_options, user_import_template, user_roles,
    },
};

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route(AUTH_SIGN_UP.api_route_path(), post(sign_up))
        .route(AUTH_SIGN_IN.api_route_path(), post(sign_in))
        .route(AUTH_REFRESH.api_route_path(), post(refresh))
        .route(AUTH_LOGOUT.api_route_path(), post(logout))
        .route(AUTH_ME.api_route_path(), get(me))
        .route(PROFILE_GET.api_route_path(), get(account_profile).put(update_account_profile))
        .route(PROFILE_PASSWORD.api_route_path(), put(change_account_password))
        .route(PROFILE_AVATAR.api_route_path(), post(upload_account_avatar))
        .route(ONLINE_LIST.api_route_path(), get(list_online_sessions))
        .route(ONLINE_FORCE_LOGOUT.api_route_path(), delete(force_logout_online_session))
        .route(USERS_LIST.api_route_path(), get(list_users).post(create_user))
        .route(USERS_EXPORT.api_route_path(), post(export_users))
        .route(USERS_IMPORT.api_route_path(), post(import_users))
        .route(USERS_IMPORT_TEMPLATE.api_route_path(), post(user_import_template))
        .route(USERS_DEPT_TREE.api_route_path(), get(user_dept_tree))
        .route(USERS_FORM_OPTIONS.api_route_path(), get(user_form_options))
        .route(USERS_DELETE_BATCH.api_route_path(), delete(delete_users))
        .route(USER_GET.api_route_path(), get(get_user).put(replace_user).delete(delete_user))
        .route(USER_RESET_PASSWORD.api_route_path(), put(reset_user_password))
        .route(USER_UPDATE_STATUS.api_route_path(), put(update_user_status))
        .route(USER_ROLES.api_route_path(), get(user_roles).put(replace_user_roles))
        .with_state(state)
}

#[cfg(test)]
mod tests;
