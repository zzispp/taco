use audit_contract::OperationAuditContext;
use axum::{
    Extension,
    extract::{Path, State},
};
use rbac::domain::DataScopeFilter;
use rbac_macros::require_perms;
use types::{http::RequestJson, system::BatchIdsInput};

use crate::{
    api::{
        ApiState,
        dto::{CreateUserPayload, ReplaceUserPayload, ResetPasswordPayload, StatusPayload, UserFormOptionsResponse, UserResponse, UserRolesPayload},
    },
    domain::UserId,
};

use super::{
    ApiJson, ApiResult,
    support::{ok, successful_operation_audit},
};

type AdminPathRequest = (State<ApiState>, Extension<DataScopeFilter>, Path<String>);
type AuditedAdminPathRequest = (
    State<ApiState>,
    Extension<DataScopeFilter>,
    Option<Extension<OperationAuditContext>>,
    Path<String>,
);
type AuditedAdminJsonRequest<T> = (
    State<ApiState>,
    Extension<DataScopeFilter>,
    Option<Extension<OperationAuditContext>>,
    Path<String>,
    T,
);
type AuditedAdminBatchRequest = (
    State<ApiState>,
    Extension<DataScopeFilter>,
    Option<Extension<OperationAuditContext>>,
    RequestJson<BatchIdsInput>,
);

#[require_perms("system:user:add")]
pub async fn create_user(
    State(state): State<ApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestJson(payload): RequestJson<CreateUserPayload>,
) -> ApiResult<ApiJson<UserResponse>> {
    let audit = successful_operation_audit(audit_context)?;
    let user = state.users.create_user_with_audit(payload.into(), audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(user.into()))
}

#[require_perms("system:user:edit")]
pub async fn replace_user(request: AuditedAdminJsonRequest<RequestJson<ReplaceUserPayload>>) -> ApiResult<ApiJson<UserResponse>> {
    let (State(state), Extension(data_scope), audit_context, Path(id), RequestJson(payload)) = request;
    UserScopeGuard::new(&state, data_scope).ensure_one(&id).await?;
    let audit = successful_operation_audit(audit_context)?;
    let user = state.users.replace_user_with_audit(UserId(id), payload.into(), audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(user.into()))
}

#[require_perms("system:user:remove")]
pub async fn delete_user(request: AuditedAdminPathRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(data_scope), audit_context, Path(id)) = request;
    UserScopeGuard::new(&state, data_scope).ensure_one(&id).await?;
    let audit = successful_operation_audit(audit_context)?;
    state.users.delete_user_with_audit(UserId(id), audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(()))
}

#[require_perms("system:user:remove")]
pub async fn delete_users(request: AuditedAdminBatchRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(data_scope), audit_context, RequestJson(payload)) = request;
    let ids = user_ids(payload.ids);
    UserScopeGuard::new(&state, data_scope).ensure_many(ids.clone()).await?;
    let audit = successful_operation_audit(audit_context)?;
    state.users.delete_users_with_audit(ids, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(()))
}

#[require_perms("system:user:query")]
pub async fn get_user(request: AdminPathRequest) -> ApiResult<ApiJson<UserResponse>> {
    let (State(state), Extension(data_scope), Path(id)) = request;
    UserScopeGuard::new(&state, data_scope).ensure_one(&id).await?;
    let user = state.users.get_user(UserId(id)).await?;
    Ok(ok(user.into()))
}

#[require_perms("system:user:resetPwd")]
pub async fn reset_user_password(request: AuditedAdminJsonRequest<RequestJson<ResetPasswordPayload>>) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(data_scope), audit_context, Path(id), RequestJson(payload)) = request;
    UserScopeGuard::new(&state, data_scope).ensure_one(&id).await?;
    let audit = successful_operation_audit(audit_context)?;
    state.users.reset_password_with_audit(UserId(id), payload.password, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(()))
}

#[require_perms("system:user:edit")]
pub async fn update_user_status(request: AuditedAdminJsonRequest<RequestJson<StatusPayload>>) -> ApiResult<ApiJson<UserResponse>> {
    let (State(state), Extension(data_scope), audit_context, Path(id), RequestJson(payload)) = request;
    UserScopeGuard::new(&state, data_scope).ensure_one(&id).await?;
    let audit = successful_operation_audit(audit_context)?;
    let user = state.users.update_status_with_audit(UserId(id), payload.status, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(user.into()))
}

#[require_perms("system:user:query")]
pub async fn user_roles(request: AdminPathRequest) -> ApiResult<ApiJson<UserRolesPayload>> {
    let (State(state), Extension(data_scope), Path(id)) = request;
    UserScopeGuard::new(&state, data_scope).ensure_one(&id).await?;
    let user = state.users.get_user(UserId(id)).await?;
    Ok(ok(UserRolesPayload { role_ids: user.role_ids }))
}

#[require_perms("system:user:edit")]
pub async fn replace_user_roles(request: AuditedAdminJsonRequest<RequestJson<UserRolesPayload>>) -> ApiResult<ApiJson<UserResponse>> {
    let (State(state), Extension(data_scope), audit_context, Path(id), RequestJson(payload)) = request;
    UserScopeGuard::new(&state, data_scope).ensure_one(&id).await?;
    let audit = successful_operation_audit(audit_context)?;
    let user = state.users.replace_roles_with_audit(UserId(id), payload.role_ids, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(user.into()))
}

#[require_perms("system:user:list")]
pub async fn user_form_options(State(state): State<ApiState>) -> ApiResult<ApiJson<UserFormOptionsResponse>> {
    let response: UserFormOptionsResponse = state.users.form_options().await?.into();
    Ok(ok(response))
}

#[require_perms("system:user:list")]
pub async fn user_dept_tree(State(state): State<ApiState>) -> ApiResult<ApiJson<Vec<types::system::TreeSelectNode>>> {
    Ok(ok(state.users.form_options().await?.depts))
}

struct UserScopeGuard<'a> {
    state: &'a ApiState,
    data_scope: DataScopeFilter,
}

impl<'a> UserScopeGuard<'a> {
    const fn new(state: &'a ApiState, data_scope: DataScopeFilter) -> Self {
        Self { state, data_scope }
    }

    async fn ensure_one(&self, id: &str) -> ApiResult<()> {
        self.ensure_many(vec![UserId(id.into())]).await
    }

    async fn ensure_many(&self, ids: Vec<UserId>) -> ApiResult<()> {
        self.state
            .users
            .ensure_user_ids_scoped(ids, self.data_scope.clone())
            .await
            .map_err(super::ApiError)
    }
}

fn user_ids(ids: Vec<String>) -> Vec<UserId> {
    ids.into_iter().map(UserId).collect()
}
