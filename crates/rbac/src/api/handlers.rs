use axum::{
    Json,
    extract::{Path, Query, State},
};
use kernel::pagination::{Page, PageRequest};
use serde::Deserialize;
use types::http::RequestJson;

use crate::api::{RbacApiError, RbacApiState};
use crate::domain::{
    ApiPermission, ApiPermissionInput, MenuItem, MenuItemInput, MenuSection, MenuSectionInput, NavResponse, Role, RoleApiBindingInput, RoleInput,
    RoleMenuBindingInput,
};

type ApiJson<T> = Json<T>;
type ApiResult<T> = Result<T, RbacApiError>;

#[derive(Debug, Deserialize)]
pub struct RbacListQuery {
    pub page: u64,
    pub page_size: u64,
}

pub async fn navbar(State(state): State<RbacApiState>, role: String) -> ApiResult<ApiJson<NavResponse>> {
    let nav = state.rbac.navbar(&role).await?;
    Ok(ok(nav))
}

pub async fn list_roles(State(state): State<RbacApiState>, Query(query): Query<RbacListQuery>) -> ApiResult<ApiJson<Page<Role>>> {
    Ok(ok(state.rbac_admin.page_roles(query.into()).await?))
}

pub async fn create_role(State(state): State<RbacApiState>, RequestJson(payload): RequestJson<RoleInput>) -> ApiResult<ApiJson<Role>> {
    Ok(ok(state.rbac_admin.create_role(payload).await?))
}

pub async fn replace_role(
    State(state): State<RbacApiState>,
    Path(code): Path<String>,
    RequestJson(payload): RequestJson<RoleInput>,
) -> ApiResult<ApiJson<Role>> {
    Ok(ok(state.rbac_admin.replace_role(&code, payload).await?))
}

pub async fn delete_role(State(state): State<RbacApiState>, Path(code): Path<String>) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.delete_role(&code).await?;
    Ok(ok(()))
}

pub async fn list_apis(State(state): State<RbacApiState>, Query(query): Query<RbacListQuery>) -> ApiResult<ApiJson<Page<ApiPermission>>> {
    Ok(ok(state.rbac_admin.page_apis(query.into()).await?))
}

pub async fn create_api(State(state): State<RbacApiState>, RequestJson(payload): RequestJson<ApiPermissionInput>) -> ApiResult<ApiJson<ApiPermission>> {
    Ok(ok(state.rbac_admin.create_api(payload).await?))
}

pub async fn replace_api(
    State(state): State<RbacApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<ApiPermissionInput>,
) -> ApiResult<ApiJson<ApiPermission>> {
    Ok(ok(state.rbac_admin.replace_api(&id, payload).await?))
}

pub async fn delete_api(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.delete_api(&id).await?;
    Ok(ok(()))
}

pub async fn replace_role_apis(
    State(state): State<RbacApiState>,
    Path(code): Path<String>,
    RequestJson(payload): RequestJson<RoleApiBindingInput>,
) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.replace_role_apis(&code, payload).await?;
    Ok(ok(()))
}

pub async fn role_api_bindings(State(state): State<RbacApiState>, Path(code): Path<String>) -> ApiResult<ApiJson<RoleApiBindingInput>> {
    let api_permission_ids = state.rbac_admin.role_api_ids(&code).await?;
    Ok(ok(RoleApiBindingInput { api_permission_ids }))
}

pub async fn list_menu_sections(State(state): State<RbacApiState>, Query(query): Query<RbacListQuery>) -> ApiResult<ApiJson<Page<MenuSection>>> {
    Ok(ok(state.rbac_admin.page_menu_sections(query.into()).await?))
}

pub async fn create_menu_section(State(state): State<RbacApiState>, RequestJson(payload): RequestJson<MenuSectionInput>) -> ApiResult<ApiJson<MenuSection>> {
    Ok(ok(state.rbac_admin.create_menu_section(payload).await?))
}

pub async fn replace_menu_section(
    State(state): State<RbacApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<MenuSectionInput>,
) -> ApiResult<ApiJson<MenuSection>> {
    Ok(ok(state.rbac_admin.replace_menu_section(&id, payload).await?))
}

pub async fn delete_menu_section(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.delete_menu_section(&id).await?;
    Ok(ok(()))
}

pub async fn list_menu_items(State(state): State<RbacApiState>, Query(query): Query<RbacListQuery>) -> ApiResult<ApiJson<Page<MenuItem>>> {
    Ok(ok(state.rbac_admin.page_menu_items(query.into()).await?))
}

pub async fn create_menu_item(State(state): State<RbacApiState>, RequestJson(payload): RequestJson<MenuItemInput>) -> ApiResult<ApiJson<MenuItem>> {
    Ok(ok(state.rbac_admin.create_menu_item(payload).await?))
}

pub async fn replace_menu_item(
    State(state): State<RbacApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<MenuItemInput>,
) -> ApiResult<ApiJson<MenuItem>> {
    Ok(ok(state.rbac_admin.replace_menu_item(&id, payload).await?))
}

pub async fn delete_menu_item(State(state): State<RbacApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.delete_menu_item(&id).await?;
    Ok(ok(()))
}

pub async fn replace_role_menus(
    State(state): State<RbacApiState>,
    Path(code): Path<String>,
    RequestJson(payload): RequestJson<RoleMenuBindingInput>,
) -> ApiResult<ApiJson<()>> {
    state.rbac_admin.replace_role_menus(&code, payload).await?;
    Ok(ok(()))
}

pub async fn role_menu_bindings(State(state): State<RbacApiState>, Path(code): Path<String>) -> ApiResult<ApiJson<RoleMenuBindingInput>> {
    let menu_item_ids = state.rbac_admin.role_menu_item_ids(&code).await?;
    Ok(ok(RoleMenuBindingInput { menu_item_ids }))
}

impl From<RbacListQuery> for PageRequest {
    fn from(value: RbacListQuery) -> Self {
        Self {
            page: value.page,
            page_size: value.page_size,
        }
    }
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(data)
}
