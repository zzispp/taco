use audit_contract::OperationAuditContext;
use axum::{
    Extension,
    extract::{Path, State},
};
use kernel::pagination::CursorPage;
use rbac_macros::require_perms;
use types::{
    http::{RequestJson, RequestQuery},
    system::BatchIdsInput,
};

use crate::{
    api::{CurrentUser, RbacApiState},
    domain::{DataScopeFilter, RoleUser, RoleUserBindingInput},
};

use super::{
    ApiJson, ApiResult, RoleUsersQuery,
    support::{ok, role_user_filter, successful_operation_audit},
};

type RoleUsersRequest = (
    State<RbacApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    Path<String>,
    RequestQuery<RoleUsersQuery>,
);
type RoleUserReplaceRequest = (
    State<RbacApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    Path<String>,
    Extension<OperationAuditContext>,
    RequestJson<RoleUserBindingInput>,
);
type DeleteRoleUserRequest = (
    State<RbacApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    Path<(String, String)>,
    Extension<OperationAuditContext>,
);
type DeleteRoleUsersRequest = (
    State<RbacApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    Path<String>,
    Extension<OperationAuditContext>,
    RequestJson<BatchIdsInput>,
);

#[require_perms("system:role:list")]
pub async fn role_users(request: RoleUsersRequest) -> ApiResult<ApiJson<CursorPage<RoleUser>>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id), RequestQuery(query)) = request;
    let scope = (!current_user.admin).then_some(data_scope);
    Ok(ok(state.rbac_admin.page_role_users(role_user_filter(id, query), scope).await?))
}

#[require_perms("system:role:edit")]
pub async fn replace_role_users(request: RoleUserReplaceRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id), Extension(audit_context), RequestJson(payload)) = request;
    RoleUserScopeGuard::new(&state, &current_user, data_scope)
        .ensure_many(payload.user_ids.clone())
        .await?;
    let audit = successful_operation_audit(audit_context)?;
    state.rbac_audited_admin.replace_role_users_with_audit(&id, payload, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(()))
}

#[require_perms("system:role:remove")]
pub async fn delete_role_user(request: DeleteRoleUserRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path((id, user_id)), Extension(audit_context)) = request;
    RoleUserScopeGuard::new(&state, &current_user, data_scope).ensure_one(&user_id).await?;
    let audit = successful_operation_audit(audit_context)?;
    state.rbac_audited_admin.delete_role_user_with_audit(&id, &user_id, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(()))
}

#[require_perms("system:role:remove")]
pub async fn delete_role_users(request: DeleteRoleUsersRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id), Extension(audit_context), RequestJson(payload)) = request;
    RoleUserScopeGuard::new(&state, &current_user, data_scope)
        .ensure_many(payload.ids.clone())
        .await?;
    let audit = successful_operation_audit(audit_context)?;
    state.rbac_audited_admin.delete_role_users_with_audit(&id, payload.ids, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(()))
}

struct RoleUserScopeGuard<'a> {
    state: &'a RbacApiState,
    current_user: &'a CurrentUser,
    data_scope: DataScopeFilter,
}

impl<'a> RoleUserScopeGuard<'a> {
    const fn new(state: &'a RbacApiState, current_user: &'a CurrentUser, data_scope: DataScopeFilter) -> Self {
        Self {
            state,
            current_user,
            data_scope,
        }
    }

    async fn ensure_one(&self, user_id: &str) -> ApiResult<()> {
        self.ensure_many(vec![user_id.into()]).await
    }

    async fn ensure_many(&self, user_ids: Vec<String>) -> ApiResult<()> {
        if self.current_user.admin {
            return Ok(());
        }
        let user_ids = clean_user_ids(user_ids);
        if user_ids.is_empty() {
            return Ok(());
        }
        self.state.rbac_admin.ensure_user_ids_scoped(user_ids, self.data_scope.clone()).await?;
        Ok(())
    }
}

fn clean_user_ids(user_ids: Vec<String>) -> Vec<String> {
    user_ids.into_iter().map(|id| id.trim().into()).filter(|id: &String| !id.is_empty()).collect()
}
