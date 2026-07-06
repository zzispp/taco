use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    response::Response,
};
use kernel::pagination::{Page, PageRequest};
use rbac_macros::{data_scope, require_perms};
use serde::Deserialize;
use types::{
    http::{RequestJson, xlsx_attachment},
    rbac::DataScopeFilter,
    system::{BatchIdsInput, SortBatchInput},
};

use crate::api::{
    CurrentUser, RbacApiError, RbacApiState,
    export::{export_roles_xlsx, role_export_page},
};
use crate::{
    application::{MenuListFilter, RoleListFilter, RoleUserListFilter},
    domain::{
        Menu, MenuInput, NavResponse, Role, RoleDataScopeInput, RoleDeptBindingInput, RoleInput, RoleMenuBindingInput, RoleMenuTreeSelect, RoleOption,
        RoleUser, RoleUserBindingInput,
    },
};

type ApiJson<T> = Json<T>;
type ApiResult<T> = Result<T, RbacApiError>;
const EXPORT_PAGE_SIZE: u64 = 100;

#[derive(Debug, Deserialize)]
pub struct RbacListQuery {
    pub page: u64,
    pub page_size: u64,
    pub role_name: Option<String>,
    pub role_key: Option<String>,
    pub menu_name: Option<String>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct RoleExportQuery {
    pub role_name: Option<String>,
    pub role_key: Option<String>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

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
pub async fn export_roles(
    State(state): State<RbacApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(data_scope): Extension<DataScopeFilter>,
    Query(query): Query<RoleExportQuery>,
) -> ApiResult<Response> {
    let roles = all_export_roles(&state, &current_user, data_scope, &query).await?;
    Ok(xlsx_attachment("roles.xlsx", export_roles_xlsx(&roles)?))
}

#[require_perms("system:role:list")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn list_roles(
    State(state): State<RbacApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(data_scope): Extension<DataScopeFilter>,
    Query(query): Query<RbacListQuery>,
) -> ApiResult<ApiJson<Page<Role>>> {
    let page = if current_user.admin {
        state.rbac_admin.page_roles(query.into()).await?
    } else {
        state.rbac_admin.page_roles_scoped(query.into(), data_scope).await?
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
pub async fn list_menus(State(state): State<RbacApiState>, Query(query): Query<RbacListQuery>) -> ApiResult<ApiJson<Page<Menu>>> {
    Ok(ok(state.rbac_admin.page_menus(query.into()).await?))
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
pub async fn replace_role_menus(
    State(state): State<RbacApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<RoleMenuBindingInput>,
) -> ApiResult<ApiJson<()>> {
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
pub async fn replace_role_depts(
    State(state): State<RbacApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<RoleDeptBindingInput>,
) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.replace_role_depts(&id, payload).await?;
    Ok(ok(()))
}

#[require_perms("system:role:list")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn role_users(
    State(state): State<RbacApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(data_scope): Extension<DataScopeFilter>,
    Path(id): Path<String>,
    Query(query): Query<RoleUsersQuery>,
) -> ApiResult<ApiJson<Page<RoleUser>>> {
    let scope = (!current_user.admin).then_some(data_scope);
    Ok(ok(state.rbac_admin.page_role_users(role_user_filter(id, query), scope).await?))
}

#[require_perms("system:role:edit")]
pub async fn replace_role_users(
    State(state): State<RbacApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<RoleUserBindingInput>,
) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.replace_role_users(&id, payload).await?;
    Ok(ok(()))
}

#[require_perms("system:role:remove")]
pub async fn delete_role_user(State(state): State<RbacApiState>, Path((id, user_id)): Path<(String, String)>) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.delete_role_user(&id, &user_id).await?;
    Ok(ok(()))
}

#[require_perms("system:role:remove")]
pub async fn delete_role_users(
    State(state): State<RbacApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<BatchIdsInput>,
) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.delete_role_users(&id, payload.ids).await?;
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

impl From<RbacListQuery> for RoleListFilter {
    fn from(value: RbacListQuery) -> Self {
        Self {
            page: PageRequest {
                page: value.page,
                page_size: value.page_size,
            },
            role_name: value.role_name,
            role_key: value.role_key,
            status: value.status,
            begin_time: value.begin_time,
            end_time: value.end_time,
        }
    }
}

impl From<RbacListQuery> for MenuListFilter {
    fn from(value: RbacListQuery) -> Self {
        Self {
            page: PageRequest {
                page: value.page,
                page_size: value.page_size,
            },
            menu_name: value.menu_name,
            status: value.status,
        }
    }
}

fn role_user_filter(role_id: String, query: RoleUsersQuery) -> RoleUserListFilter {
    RoleUserListFilter {
        page: PageRequest {
            page: query.page,
            page_size: query.page_size,
        },
        role_id,
        username: query.username,
        phonenumber: query.phonenumber,
        allocated: query.allocated.unwrap_or(true),
    }
}

fn menu_tree(menus: Vec<Menu>) -> Vec<types::system::TreeSelectNode> {
    menus.iter().filter(|menu| menu.parent_id == "0").map(|menu| menu_node(menu, &menus)).collect()
}

fn menu_node(menu: &Menu, menus: &[Menu]) -> types::system::TreeSelectNode {
    types::system::TreeSelectNode {
        id: menu.menu_id.clone(),
        label: menu.menu_name.clone(),
        parent_id: menu.parent_id.clone(),
        disabled: menu.status != types::rbac::STATUS_NORMAL,
        children: menus
            .iter()
            .filter(|child| child.parent_id == menu.menu_id)
            .map(|child| menu_node(child, menus))
            .collect(),
    }
}

fn checked_keys_for_tree(tree: &[types::system::TreeSelectNode], checked_keys: Vec<String>, strictly: bool) -> Vec<String> {
    if strictly {
        checked_keys.into_iter().filter(|key| tree_leaf_contains(tree, key)).collect()
    } else {
        checked_keys
    }
}

fn tree_leaf_contains(tree: &[types::system::TreeSelectNode], key: &str) -> bool {
    tree.iter().any(|node| {
        if node.id == key {
            return node.children.is_empty();
        }
        tree_leaf_contains(&node.children, key)
    })
}

async fn all_export_roles(state: &RbacApiState, current_user: &CurrentUser, data_scope: DataScopeFilter, query: &RoleExportQuery) -> ApiResult<Vec<Role>> {
    let mut page = 1;
    let mut roles = Vec::new();
    loop {
        let filter = role_export_page(query, page, EXPORT_PAGE_SIZE);
        let current = if current_user.admin {
            state.rbac_admin.page_roles(filter).await?
        } else {
            state.rbac_admin.page_roles_scoped(filter, data_scope.clone()).await?
        };
        let is_last = current.items.is_empty() || roles.len() + current.items.len() >= current.total as usize;
        roles.extend(current.items);
        if is_last {
            return Ok(roles);
        }
        page += 1;
    }
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(data)
}
