use axum::{
    Extension, Router,
    routing::{get, put},
};

use super::{
    CurrentUser,
    handlers::{
        create_api, create_menu_item, create_menu_section, create_role, delete_api, delete_menu_item, delete_menu_section, delete_role, list_apis,
        list_menu_items, list_menu_sections, list_roles, navbar, replace_api, replace_menu_item, replace_menu_section, replace_role, replace_role_apis,
        replace_role_menus, role_api_bindings, role_menu_bindings,
    },
    state::RbacApiState,
};

pub fn create_router(state: RbacApiState) -> Router {
    Router::new()
        .route("/navbar", get(navbar_route))
        .route("/rbac/roles", get(list_roles).post(create_role))
        .route("/rbac/roles/{code}", put(replace_role).delete(delete_role))
        .route("/rbac/roles/{code}/apis", get(role_api_bindings).put(replace_role_apis))
        .route("/rbac/roles/{code}/menus", get(role_menu_bindings).put(replace_role_menus))
        .route("/rbac/apis", get(list_apis).post(create_api))
        .route("/rbac/apis/{id}", put(replace_api).delete(delete_api))
        .route("/rbac/menu-sections", get(list_menu_sections).post(create_menu_section))
        .route("/rbac/menu-sections/{id}", put(replace_menu_section).delete(delete_menu_section))
        .route("/rbac/menu-items", get(list_menu_items).post(create_menu_item))
        .route("/rbac/menu-items/{id}", put(replace_menu_item).delete(delete_menu_item))
        .with_state(state)
}

async fn navbar_route(
    state: axum::extract::State<RbacApiState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<axum::Json<crate::domain::NavResponse>, super::RbacApiError> {
    navbar(state, current_user.role).await
}
