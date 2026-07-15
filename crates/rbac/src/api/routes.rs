use axum::{
    Extension, Router,
    routing::{get, put},
};

use super::endpoints::{
    MENU_REPLACE, MENU_SORT, MENUS_CREATE, MENUS_SORT, MENUS_TREE, MENUS_TREE_SELECT, NAVBAR, ROLE_DATA_SCOPE, ROLE_DEPTS_REPLACE, ROLE_MENU_TREE,
    ROLE_MENUS_REPLACE, ROLE_REPLACE, ROLE_STATUS, ROLE_USER_DELETE, ROLE_USERS_DELETE_BATCH, ROLE_USERS_REPLACE, ROLES_CREATE, ROLES_DELETE_BATCH,
    ROLES_EXPORT, ROLES_OPTIONS,
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
        .route(NAVBAR.api_route_path(), get(navbar_route))
        .route(ROLES_CREATE.api_route_path(), get(list_roles).post(create_role))
        .route(ROLES_EXPORT.api_route_path(), axum::routing::post(export_roles))
        .route(ROLES_OPTIONS.api_route_path(), get(role_options))
        .route(ROLES_DELETE_BATCH.api_route_path(), axum::routing::delete(delete_roles))
        .route(ROLE_REPLACE.api_route_path(), get(get_role).put(replace_role).delete(delete_role))
        .route(ROLE_STATUS.api_route_path(), put(update_role_status))
        .route(ROLE_DATA_SCOPE.api_route_path(), put(update_role_data_scope))
        .route(ROLE_MENUS_REPLACE.api_route_path(), get(role_menu_bindings).put(replace_role_menus))
        .route(ROLE_DEPTS_REPLACE.api_route_path(), get(role_dept_bindings).put(replace_role_depts))
        .route(ROLE_USERS_REPLACE.api_route_path(), get(role_users).put(replace_role_users))
        .route(ROLE_USERS_DELETE_BATCH.api_route_path(), axum::routing::delete(delete_role_users))
        .route(ROLE_USER_DELETE.api_route_path(), axum::routing::delete(delete_role_user))
        .route(MENUS_CREATE.api_route_path(), get(list_menus).post(create_menu))
        .route(MENUS_TREE.api_route_path(), get(list_menu_tree))
        .route(MENUS_TREE_SELECT.api_route_path(), get(menu_tree_select))
        .route(ROLE_MENU_TREE.api_route_path(), get(role_menu_tree_select))
        .route(MENUS_SORT.api_route_path(), put(update_menu_sorts))
        .route(MENU_REPLACE.api_route_path(), get(get_menu).put(replace_menu).delete(delete_menu))
        .route(MENU_SORT.api_route_path(), put(update_menu_sort))
        .with_state(state)
}

async fn navbar_route(
    state: axum::extract::State<RbacApiState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<axum::Json<crate::domain::NavResponse>, super::RbacApiError> {
    navbar(state, current_user).await
}
