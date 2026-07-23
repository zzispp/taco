use axum::{
    Extension, Json,
    extract::{Path, State},
    response::Response,
};
use kernel::pagination::{CursorPage, CursorPageRequest};
use rbac_macros::require_perms;
use serde::Deserialize;
use types::http::{RequestQuery, current_locale, xlsx_file_attachment};
use utoipa::IntoParams;

use crate::api::{
    CurrentUser, RbacApiError, RbacApiState,
    export::RoleXlsxExport,
    input::{MenuListQuery, RoleExportQuery, RoleListQuery, menu_list_filter, role_export_filter, role_list_filter},
};
use crate::{
    application::{RoleExportRequest, RoleUserListFilter},
    domain::{DataScopeFilter, Menu, NavResponse, Role, RoleDeptBindingInput, RoleMenuBindingInput, RoleMenuTreeSelect, RoleOption},
};

type ApiJson<T> = Json<T>;

mod audited_admin;
mod role_user_handlers;
mod support;

pub use audited_admin::{
    create_menu, create_role, delete_menu, delete_role, delete_roles, replace_menu, replace_role, replace_role_depts, replace_role_menus, update_menu_sort,
    update_menu_sorts, update_role_data_scope, update_role_status,
};
pub use role_user_handlers::{delete_role_user, delete_role_users, replace_role_users, role_users};

use self::support::{checked_keys_for_tree, menu_tree, ok};

type ExportRolesRequest = (State<RbacApiState>, Extension<DataScopeFilter>, RequestQuery<RoleExportQuery>);
type ListRolesRequest = (State<RbacApiState>, Extension<DataScopeFilter>, RequestQuery<RoleListQuery>);
type ApiResult<T> = Result<T, RbacApiError>;

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
#[serde(deny_unknown_fields)]
pub struct RoleUsersQuery {
    #[serde(default = "default_limit")]
    #[param(default = 20, minimum = 1, maximum = 100)]
    pub limit: u64,
    #[serde(default)]
    pub cursor: Option<String>,
    pub username: Option<String>,
    pub phonenumber: Option<String>,
    pub allocated: Option<bool>,
}

const fn default_limit() -> u64 {
    kernel::pagination::DEFAULT_CURSOR_LIMIT
}

#[derive(Debug, Deserialize)]
pub struct StatusPayload {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct SortPayload {
    pub order_num: i64,
}

pub async fn navbar(State(state): State<RbacApiState>, current_user: CurrentUser) -> ApiResult<ApiJson<NavResponse>> {
    Ok(ok(state.rbac.navbar(&current_user).await?))
}

#[require_perms("system:role:export")]
pub async fn export_roles(request: ExportRolesRequest) -> ApiResult<Response> {
    let (State(state), Extension(data_scope), RequestQuery(query)) = request;
    let filter = role_export_filter(query)?;
    let batch_size = state.export_config.export_batch_config().await?.page_size;
    let mut export = RoleXlsxExport::new(current_locale())?;
    state
        .rbac_admin
        .export_roles(
            RoleExportRequest {
                filter: filter.page_filter(CursorPageRequest::default()),
                scope: Some(data_scope),
                batch_size,
            },
            &mut export,
        )
        .await?;
    Ok(xlsx_file_attachment("roles.xlsx", export.finish()?))
}

#[require_perms("system:role:list")]
pub async fn list_roles(request: ListRolesRequest) -> ApiResult<ApiJson<CursorPage<Role>>> {
    let (State(state), Extension(data_scope), RequestQuery(query)) = request;
    let filter = role_list_filter(query)?;
    let page = state.rbac_admin.page_roles_scoped(filter, data_scope).await?;
    Ok(ok(page))
}

#[require_perms("system:role:query")]
pub async fn get_role(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<Role>> {
    Ok(ok(state.rbac_admin.get_role(&id).await?))
}

#[require_perms("system:role:list")]
pub async fn role_options(State(state): State<RbacApiState>) -> ApiResult<ApiJson<Vec<RoleOption>>> {
    Ok(ok(state.rbac_admin.role_options().await?))
}

#[require_perms("system:menu:list")]
pub async fn list_menus(State(state): State<RbacApiState>, RequestQuery(query): RequestQuery<MenuListQuery>) -> ApiResult<ApiJson<CursorPage<Menu>>> {
    Ok(ok(state.rbac_admin.page_menus(menu_list_filter(query)?).await?))
}

#[require_perms("system:menu:query")]
pub async fn get_menu(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<Menu>> {
    Ok(ok(state.rbac_admin.get_menu(&id).await?))
}

#[require_perms("system:menu:list")]
pub async fn list_menu_tree(State(state): State<RbacApiState>) -> ApiResult<ApiJson<Vec<Menu>>> {
    Ok(ok(state.rbac_admin.list_menus().await?))
}

#[require_perms("system:role:query")]
pub async fn role_menu_bindings(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<RoleMenuBindingInput>> {
    Ok(ok(RoleMenuBindingInput {
        menu_ids: state.rbac_admin.role_menu_ids(&id).await?,
    }))
}

#[require_perms("system:role:query")]
pub async fn role_dept_bindings(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<RoleDeptBindingInput>> {
    Ok(ok(RoleDeptBindingInput {
        dept_ids: state.rbac_admin.role_dept_ids(&id).await?,
    }))
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        routing::get,
    };
    use serde_json::Value;
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn legacy_role_user_page_parameters_are_rejected_by_route() {
        let app = Router::new().route("/roles/{id}/users", get(|RequestQuery(_): RequestQuery<RoleUsersQuery>| async {}));

        for uri in ["/roles/1/users?page=1", "/roles/1/users?page_size=20"] {
            let response = app.clone().oneshot(Request::get(uri).body(Body::empty()).unwrap()).await.unwrap();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            let bytes = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let body = serde_json::from_slice::<Value>(&bytes).unwrap();
            assert_eq!(body["code"], "invalid_input");
        }
    }
}

#[require_perms("system:menu:list")]
pub async fn menu_tree_select(State(state): State<RbacApiState>) -> ApiResult<ApiJson<Vec<types::system::TreeSelectNode>>> {
    Ok(ok(menu_tree(state.rbac_admin.list_menus().await?)))
}

#[require_perms("system:role:query")]
pub async fn role_menu_tree_select(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<RoleMenuTreeSelect>> {
    let menus = menu_tree(state.rbac_admin.list_menus().await?);
    let role = state.rbac_admin.get_role(&id).await?;
    let checked_keys = state.rbac_admin.role_menu_ids(&id).await?;
    let checked_keys = checked_keys_for_tree(&menus, checked_keys, role.menu_check_strictly);
    Ok(ok(RoleMenuTreeSelect { menus, checked_keys }))
}
