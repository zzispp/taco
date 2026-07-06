use axum::{
    Extension, Router,
    routing::{get, put},
};

use super::{
    CurrentUser,
    handlers::{
        create_menu, create_role, delete_menu, delete_role, delete_role_user, delete_role_users, delete_roles, export_roles, get_menu, get_role,
        list_menu_tree, list_menus, list_roles, menu_tree_select, navbar, replace_menu, replace_role, replace_role_depts, replace_role_menus,
        replace_role_users, role_dept_bindings, role_menu_bindings, role_menu_tree_select, role_options, role_users, update_menu_sort, update_menu_sorts,
        update_role_data_scope, update_role_status,
    },
    state::RbacApiState,
};

pub fn create_router(state: RbacApiState) -> Router {
    Router::new()
        .route("/navbar", get(navbar_route))
        .route("/system/roles", get(list_roles).post(create_role))
        .route("/system/roles/export", axum::routing::post(export_roles))
        .route("/system/roles/options", get(role_options))
        .route("/system/roles/batch", axum::routing::delete(delete_roles))
        .route("/system/roles/{id}", get(get_role).put(replace_role).delete(delete_role))
        .route("/system/roles/{id}/status", put(update_role_status))
        .route("/system/roles/{id}/data-scope", put(update_role_data_scope))
        .route("/system/roles/{id}/menus", get(role_menu_bindings).put(replace_role_menus))
        .route("/system/roles/{id}/depts", get(role_dept_bindings).put(replace_role_depts))
        .route("/system/roles/{id}/users", get(role_users).put(replace_role_users))
        .route("/system/roles/{id}/users/batch", axum::routing::delete(delete_role_users))
        .route("/system/roles/{id}/users/{user_id}", axum::routing::delete(delete_role_user))
        .route("/system/menus", get(list_menus).post(create_menu))
        .route("/system/menus/tree", get(list_menu_tree))
        .route("/system/menus/tree-select", get(menu_tree_select))
        .route("/system/menus/role-tree-select/{id}", get(role_menu_tree_select))
        .route("/system/menus/sort", put(update_menu_sorts))
        .route("/system/menus/{id}", get(get_menu).put(replace_menu).delete(delete_menu))
        .route("/system/menus/{id}/sort", put(update_menu_sort))
        .with_state(state)
}

async fn navbar_route(
    state: axum::extract::State<RbacApiState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<axum::Json<crate::domain::NavResponse>, super::RbacApiError> {
    navbar(state, current_user).await
}
