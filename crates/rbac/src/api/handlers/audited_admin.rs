use audit_contract::OperationAuditContext;
use axum::{
    Extension,
    extract::{Path, State},
};
use rbac_macros::require_perms;
use types::{
    http::RequestJson,
    system::{BatchIdsInput, SortBatchInput},
};

use crate::{
    api::RbacApiState,
    domain::{Menu, MenuInput, Role, RoleDataScopeInput, RoleDeptBindingInput, RoleInput, RoleMenuBindingInput},
};

use super::{
    ApiJson, ApiResult, SortPayload, StatusPayload,
    support::{ok, successful_operation_audit},
};

type AuditedResourceMutation<T> = (State<RbacApiState>, Path<String>, Extension<OperationAuditContext>, RequestJson<T>);
type RoleStatusRequest = AuditedResourceMutation<StatusPayload>;
type RoleDataScopeRequest = AuditedResourceMutation<RoleDataScopeInput>;
type ReplaceRoleRequest = AuditedResourceMutation<RoleInput>;
type ReplaceMenuRequest = AuditedResourceMutation<MenuInput>;
type MenuSortRequest = AuditedResourceMutation<SortPayload>;
type RoleMenuRequest = AuditedResourceMutation<RoleMenuBindingInput>;
type RoleDeptRequest = AuditedResourceMutation<RoleDeptBindingInput>;

#[require_perms("system:role:edit")]
pub async fn update_role_status((State(state), Path(id), Extension(audit_context), RequestJson(payload)): RoleStatusRequest) -> ApiResult<ApiJson<Role>> {
    let audit = successful_operation_audit(audit_context)?;
    let role = state
        .rbac_audited_admin
        .update_role_status_with_audit(&id, payload.status, audit.record())
        .await?;
    audit.mark_persisted();
    refresh_rbac_cache(&state).await?;
    Ok(ok(role))
}

#[require_perms("system:role:edit")]
pub async fn update_role_data_scope(
    (State(state), Path(id), Extension(audit_context), RequestJson(payload)): RoleDataScopeRequest,
) -> ApiResult<ApiJson<Role>> {
    let audit = successful_operation_audit(audit_context)?;
    let role = state.rbac_audited_admin.update_role_data_scope_with_audit(&id, payload, audit.record()).await?;
    audit.mark_persisted();
    refresh_rbac_cache(&state).await?;
    Ok(ok(role))
}

#[require_perms("system:role:add")]
pub async fn create_role(
    State(state): State<RbacApiState>,
    Extension(audit_context): Extension<OperationAuditContext>,
    RequestJson(payload): RequestJson<RoleInput>,
) -> ApiResult<ApiJson<Role>> {
    let audit = successful_operation_audit(audit_context)?;
    let role = state.rbac_audited_admin.create_role_with_audit(payload, audit.record()).await?;
    audit.mark_persisted();
    refresh_rbac_cache(&state).await?;
    Ok(ok(role))
}

#[require_perms("system:role:edit")]
pub async fn replace_role((State(state), Path(id), Extension(audit_context), RequestJson(payload)): ReplaceRoleRequest) -> ApiResult<ApiJson<Role>> {
    let audit = successful_operation_audit(audit_context)?;
    let role = state.rbac_audited_admin.replace_role_with_audit(&id, payload, audit.record()).await?;
    audit.mark_persisted();
    refresh_rbac_cache(&state).await?;
    Ok(ok(role))
}

#[require_perms("system:role:remove")]
pub async fn delete_role(
    State(state): State<RbacApiState>,
    Path(id): Path<String>,
    Extension(audit_context): Extension<OperationAuditContext>,
) -> ApiResult<ApiJson<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.rbac_audited_admin.delete_role_with_audit(&id, audit.record()).await?;
    audit.mark_persisted();
    refresh_rbac_cache(&state).await?;
    Ok(ok(()))
}

#[require_perms("system:role:remove")]
pub async fn delete_roles(
    State(state): State<RbacApiState>,
    Extension(audit_context): Extension<OperationAuditContext>,
    RequestJson(payload): RequestJson<BatchIdsInput>,
) -> ApiResult<ApiJson<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.rbac_audited_admin.delete_roles_with_audit(payload.ids, audit.record()).await?;
    audit.mark_persisted();
    refresh_rbac_cache(&state).await?;
    Ok(ok(()))
}

#[require_perms("system:menu:add")]
pub async fn create_menu(
    State(state): State<RbacApiState>,
    Extension(audit_context): Extension<OperationAuditContext>,
    RequestJson(payload): RequestJson<MenuInput>,
) -> ApiResult<ApiJson<Menu>> {
    let audit = successful_operation_audit(audit_context)?;
    let menu = state.rbac_audited_admin.create_menu_with_audit(payload, audit.record()).await?;
    audit.mark_persisted();
    refresh_rbac_cache(&state).await?;
    Ok(ok(menu))
}

#[require_perms("system:menu:edit")]
pub async fn replace_menu((State(state), Path(id), Extension(audit_context), RequestJson(payload)): ReplaceMenuRequest) -> ApiResult<ApiJson<Menu>> {
    let audit = successful_operation_audit(audit_context)?;
    let menu = state.rbac_audited_admin.replace_menu_with_audit(&id, payload, audit.record()).await?;
    audit.mark_persisted();
    refresh_rbac_cache(&state).await?;
    Ok(ok(menu))
}

#[require_perms("system:menu:edit")]
pub async fn update_menu_sort((State(state), Path(id), Extension(audit_context), RequestJson(payload)): MenuSortRequest) -> ApiResult<ApiJson<Menu>> {
    let audit = successful_operation_audit(audit_context)?;
    let menu = state
        .rbac_audited_admin
        .update_menu_sort_with_audit(&id, payload.order_num, audit.record())
        .await?;
    audit.mark_persisted();
    refresh_rbac_cache(&state).await?;
    Ok(ok(menu))
}

#[require_perms("system:menu:edit")]
pub async fn update_menu_sorts(
    State(state): State<RbacApiState>,
    Extension(audit_context): Extension<OperationAuditContext>,
    RequestJson(payload): RequestJson<SortBatchInput>,
) -> ApiResult<ApiJson<Vec<Menu>>> {
    let audit = successful_operation_audit(audit_context)?;
    let menus = state.rbac_audited_admin.update_menu_sorts_with_audit(payload, audit.record()).await?;
    audit.mark_persisted();
    refresh_rbac_cache(&state).await?;
    Ok(ok(menus))
}

#[require_perms("system:menu:remove")]
pub async fn delete_menu(
    State(state): State<RbacApiState>,
    Path(id): Path<String>,
    Extension(audit_context): Extension<OperationAuditContext>,
) -> ApiResult<ApiJson<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.rbac_audited_admin.delete_menu_with_audit(&id, audit.record()).await?;
    audit.mark_persisted();
    refresh_rbac_cache(&state).await?;
    Ok(ok(()))
}

#[require_perms("system:role:edit")]
pub async fn replace_role_menus((State(state), Path(id), Extension(audit_context), RequestJson(payload)): RoleMenuRequest) -> ApiResult<ApiJson<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.rbac_audited_admin.replace_role_menus_with_audit(&id, payload, audit.record()).await?;
    audit.mark_persisted();
    refresh_rbac_cache(&state).await?;
    Ok(ok(()))
}

#[require_perms("system:role:edit")]
pub async fn replace_role_depts((State(state), Path(id), Extension(audit_context), RequestJson(payload)): RoleDeptRequest) -> ApiResult<ApiJson<()>> {
    let audit = successful_operation_audit(audit_context)?;
    state.rbac_audited_admin.replace_role_depts_with_audit(&id, payload, audit.record()).await?;
    audit.mark_persisted();
    refresh_rbac_cache(&state).await?;
    Ok(ok(()))
}

async fn refresh_rbac_cache(state: &RbacApiState) -> ApiResult<()> {
    state.rbac_cache_refresher.refresh_after_audited_write().await?;
    Ok(())
}
