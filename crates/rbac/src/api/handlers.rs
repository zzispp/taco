use axum::{
    Extension, Json,
    extract::{Path, State},
    response::Response,
};
use kernel::pagination::{Page, PageRequest};
use rbac_macros::{data_scope, require_perms};
use serde::Deserialize;
use types::{
    http::{RequestJson, RequestQuery, current_locale, xlsx_attachment},
    rbac::DataScopeFilter,
    system::{BatchIdsInput, SortBatchInput},
};

use crate::api::{
    CurrentUser, RbacApiError, RbacApiState,
    export::export_roles_xlsx,
    input::{MenuListQuery, RoleExportFilter, RoleExportQuery, RoleListQuery, menu_list_filter, role_export_filter, role_list_filter},
};
use crate::{
    application::RoleUserListFilter,
    domain::{
        Menu, MenuInput, NavResponse, Role, RoleDataScopeInput, RoleDeptBindingInput, RoleInput, RoleMenuBindingInput, RoleMenuTreeSelect, RoleOption,
        RoleUser, RoleUserBindingInput,
    },
};

type ApiJson<T> = Json<T>;

mod role_user_handlers;
mod support;

pub use role_user_handlers::{delete_role_user, delete_role_users, replace_role_users, role_users};

use self::support::{ExportRolesInput, all_export_roles, checked_keys_for_tree, menu_tree, ok, role_user_filter};

type ExportRolesRequest = (
    State<RbacApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    RequestQuery<RoleExportQuery>,
);
type ListRolesRequest = (
    State<RbacApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    RequestQuery<RoleListQuery>,
);
type RoleMenuRequest = (State<RbacApiState>, Path<String>, RequestJson<RoleMenuBindingInput>);
type RoleDeptRequest = (State<RbacApiState>, Path<String>, RequestJson<RoleDeptBindingInput>);
type ApiResult<T> = Result<T, RbacApiError>;

#[derive(Debug, Deserialize)]
pub struct RoleUsersQuery {
    pub page: u64,
    pub page_size: u64,
    pub username: Option<String>,
    pub phonenumber: Option<String>,
    pub allocated: Option<bool>,
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
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn export_roles(request: ExportRolesRequest) -> ApiResult<Response> {
    let (State(state), Extension(current_user), Extension(data_scope), RequestQuery(query)) = request;
    let filter = role_export_filter(query)?;
    let roles = all_export_roles(ExportRolesInput {
        state: &state,
        current_user: &current_user,
        data_scope,
        filter,
    })
    .await?;
    Ok(xlsx_attachment("roles.xlsx", export_roles_xlsx(&roles, current_locale())?))
}

#[require_perms("system:role:list")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn list_roles(request: ListRolesRequest) -> ApiResult<ApiJson<Page<Role>>> {
    let (State(state), Extension(current_user), Extension(data_scope), RequestQuery(query)) = request;
    let filter = role_list_filter(query)?;
    let page = if current_user.admin {
        state.rbac_admin.page_roles(filter).await?
    } else {
        state.rbac_admin.page_roles_scoped(filter, data_scope).await?
    };
    Ok(ok(page))
}

#[require_perms("system:role:query")]
pub async fn get_role(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<Role>> {
    Ok(ok(state.rbac_admin.get_role(&id).await?))
}

#[require_perms("system:role:edit")]
pub async fn update_role_status(
    State(state): State<RbacApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<StatusPayload>,
) -> ApiResult<ApiJson<Role>> {
    Ok(ok(state.rbac_admin.update_role_status(&id, payload.status).await?))
}

#[require_perms("system:role:edit")]
pub async fn update_role_data_scope(
    State(state): State<RbacApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<RoleDataScopeInput>,
) -> ApiResult<ApiJson<Role>> {
    Ok(ok(state.rbac_admin.update_role_data_scope(&id, payload).await?))
}

#[require_perms("system:role:list")]
pub async fn role_options(State(state): State<RbacApiState>) -> ApiResult<ApiJson<Vec<RoleOption>>> {
    Ok(ok(state.rbac_admin.role_options().await?))
}

#[require_perms("system:role:add")]
pub async fn create_role(State(state): State<RbacApiState>, RequestJson(payload): RequestJson<RoleInput>) -> ApiResult<ApiJson<Role>> {
    Ok(ok(state.rbac_admin.create_role(payload).await?))
}

#[require_perms("system:role:edit")]
pub async fn replace_role(State(state): State<RbacApiState>, Path(id): Path<String>, RequestJson(payload): RequestJson<RoleInput>) -> ApiResult<ApiJson<Role>> {
    Ok(ok(state.rbac_admin.replace_role(&id, payload).await?))
}

#[require_perms("system:role:remove")]
pub async fn delete_role(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.delete_role(&id).await?;
    Ok(ok(()))
}

#[require_perms("system:role:remove")]
pub async fn delete_roles(State(state): State<RbacApiState>, RequestJson(payload): RequestJson<BatchIdsInput>) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.delete_roles(payload.ids).await?;
    Ok(ok(()))
}

#[require_perms("system:menu:list")]
pub async fn list_menus(State(state): State<RbacApiState>, RequestQuery(query): RequestQuery<MenuListQuery>) -> ApiResult<ApiJson<Page<Menu>>> {
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

#[require_perms("system:menu:add")]
pub async fn create_menu(State(state): State<RbacApiState>, RequestJson(payload): RequestJson<MenuInput>) -> ApiResult<ApiJson<Menu>> {
    Ok(ok(state.rbac_admin.create_menu(payload).await?))
}

#[require_perms("system:menu:edit")]
pub async fn replace_menu(State(state): State<RbacApiState>, Path(id): Path<String>, RequestJson(payload): RequestJson<MenuInput>) -> ApiResult<ApiJson<Menu>> {
    Ok(ok(state.rbac_admin.replace_menu(&id, payload).await?))
}

#[require_perms("system:menu:edit")]
pub async fn update_menu_sort(
    State(state): State<RbacApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<SortPayload>,
) -> ApiResult<ApiJson<Menu>> {
    Ok(ok(state.rbac_admin.update_menu_sort(&id, payload.order_num).await?))
}

#[require_perms("system:menu:edit")]
pub async fn update_menu_sorts(State(state): State<RbacApiState>, RequestJson(payload): RequestJson<SortBatchInput>) -> ApiResult<ApiJson<Vec<Menu>>> {
    Ok(ok(state.rbac_admin.update_menu_sorts(payload).await?))
}

#[require_perms("system:menu:remove")]
pub async fn delete_menu(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.delete_menu(&id).await?;
    Ok(ok(()))
}

#[require_perms("system:role:query")]
pub async fn role_menu_bindings(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<RoleMenuBindingInput>> {
    Ok(ok(RoleMenuBindingInput {
        menu_ids: state.rbac_admin.role_menu_ids(&id).await?,
    }))
}

#[require_perms("system:role:edit")]
pub async fn replace_role_menus((State(state), Path(id), RequestJson(payload)): RoleMenuRequest) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.replace_role_menus(&id, payload).await?;
    Ok(ok(()))
}

#[require_perms("system:role:query")]
pub async fn role_dept_bindings(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<RoleDeptBindingInput>> {
    Ok(ok(RoleDeptBindingInput {
        dept_ids: state.rbac_admin.role_dept_ids(&id).await?,
    }))
}

#[require_perms("system:role:edit")]
pub async fn replace_role_depts((State(state), Path(id), RequestJson(payload)): RoleDeptRequest) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.replace_role_depts(&id, payload).await?;
    Ok(ok(()))
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
