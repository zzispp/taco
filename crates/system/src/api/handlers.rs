use audit_contract::OperationAuditContext;
use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    response::Response,
};
use kernel::pagination::{CursorPage, CursorPageRequest};
use rbac::domain::DataScopeFilter;
use rbac_macros::require_perms;
use serde::Deserialize;
use types::http::{RequestJson, RequestQuery, current_locale, xlsx_file_attachment};
use types::rbac::RoleDeptTreeSelect;
use types::system::BatchIdsInput;

use crate::{
    api::{
        SystemApiError, SystemApiState,
        export::SystemXlsxExport,
        input::{
            DeptTreeQuery, SystemExportQuery, SystemListQuery, config_export_filter, config_list_filter, dept_list_filter, dept_tree_filter,
            dict_data_export_filter, dict_data_list_filter, dict_type_export_filter, dict_type_list_filter, post_export_filter, post_list_filter,
        },
    },
    application::{DeptListFilter, SystemExportKind, SystemExportRequest},
    domain::{ConfigInput, ConfigItem, Dept, DeptInput, DictData, DictDataInput, DictType, DictTypeInput, Post, PostInput, SortBatchInput, TreeSelectNode},
};

type ApiJson<T> = Json<T>;
type AuditedResourceJsonRequest<T> = (State<SystemApiState>, Option<Extension<OperationAuditContext>>, Path<String>, RequestJson<T>);

mod resources;
mod support;

pub use resources::{
    config_by_key, create_config, create_dict_data, create_dict_type, create_post, delete_config, delete_configs, delete_dict_data, delete_dict_data_batch,
    delete_dict_type, delete_dict_types, delete_post, delete_posts, dict_data_by_type, dict_type_options, export_configs, export_dict_data, export_dict_types,
    export_posts, get_config, get_dict_data, get_dict_type, get_post, list_configs, list_dict_data, list_dict_types, list_posts, post_options,
    refresh_config_cache, refresh_dict_cache, replace_config, replace_dict_data, replace_dict_type, replace_post,
};

use self::support::{all_depts_filter, checked_keys_for_tree, config_keys, ok};
use crate::api::operation_audit::successful_operation_audit;

type ListDeptsRequest = (State<SystemApiState>, Extension<DataScopeFilter>, RequestQuery<SystemListQuery>);
type DeptTreeRequest = (State<SystemApiState>, Extension<DataScopeFilter>, RequestQuery<DeptTreeQuery>);
type DeptPathRequest = (State<SystemApiState>, Extension<DataScopeFilter>, Path<String>);
type AuditedDeptPathRequest = (
    State<SystemApiState>,
    Extension<DataScopeFilter>,
    Option<Extension<OperationAuditContext>>,
    Path<String>,
);
type AuditedDeptJsonRequest<T> = (
    State<SystemApiState>,
    Extension<DataScopeFilter>,
    Option<Extension<OperationAuditContext>>,
    Path<String>,
    T,
);
type AuditedDeptSortsRequest = (
    State<SystemApiState>,
    Extension<DataScopeFilter>,
    Option<Extension<OperationAuditContext>>,
    RequestJson<SortBatchInput>,
);
type ApiResult<T> = Result<T, SystemApiError>;

#[derive(Debug, Deserialize)]
pub struct SortPayload {
    pub order_num: i64,
}

#[derive(Debug, Default, Deserialize)]
pub struct PublicConfigQuery {
    pub keys: Option<String>,
}

pub async fn public_configs(
    State(state): State<SystemApiState>,
    Query(query): Query<PublicConfigQuery>,
) -> ApiResult<ApiJson<std::collections::BTreeMap<String, String>>> {
    Ok(ok(state.system.public_configs(config_keys(query)).await?))
}

#[require_perms("system:dept:list")]
pub async fn list_depts(request: ListDeptsRequest) -> ApiResult<ApiJson<CursorPage<Dept>>> {
    let (State(state), Extension(data_scope), RequestQuery(query)) = request;
    let filter = dept_list_filter(query)?;
    let page = state.system.page_depts_scoped(filter, data_scope).await?;
    Ok(ok(page))
}

#[require_perms("system:dept:add")]
pub async fn create_dept(
    State(state): State<SystemApiState>,
    audit_context: Option<Extension<OperationAuditContext>>,
    RequestJson(payload): RequestJson<DeptInput>,
) -> ApiResult<ApiJson<Dept>> {
    let audit = successful_operation_audit(audit_context)?;
    let dept = state.system_audited.create_dept_with_audit(payload, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(dept))
}
#[require_perms("system:dept:query")]
pub async fn get_dept(request: DeptPathRequest) -> ApiResult<ApiJson<Dept>> {
    let (State(state), Extension(data_scope), Path(id)) = request;
    DeptScopeGuard::new(&state, data_scope).ensure_one(&id).await?;
    Ok(ok(state.system.get_dept(&id).await?))
}

#[require_perms("system:dept:list")]
pub async fn dept_tree_select(request: DeptTreeRequest) -> ApiResult<ApiJson<Vec<TreeSelectNode>>> {
    let (State(state), Extension(data_scope), RequestQuery(query)) = request;
    Ok(ok(state.system.dept_tree(dept_tree_filter(query)?, Some(data_scope)).await?))
}

#[require_perms("system:dept:list")]
pub async fn exclude_dept_tree(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<Vec<TreeSelectNode>>> {
    Ok(ok(state.system.exclude_dept_tree(&id).await?))
}

#[require_perms("system:role:query")]
pub async fn role_dept_tree_select(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<RoleDeptTreeSelect>> {
    let role = state.rbac_admin.get_role(&id).await.map_err(SystemApiError::from)?;
    let checked_keys = state.rbac_admin.role_dept_ids(&id).await.map_err(SystemApiError::from)?;
    let tree = state.system.dept_tree(all_depts_filter(), None).await?;
    Ok(ok(RoleDeptTreeSelect {
        depts: tree.clone(),
        checked_keys: checked_keys_for_tree(&tree, checked_keys, role.dept_check_strictly),
    }))
}

#[require_perms("system:dept:edit")]
pub async fn replace_dept(request: AuditedDeptJsonRequest<RequestJson<DeptInput>>) -> ApiResult<ApiJson<Dept>> {
    let (State(state), Extension(data_scope), audit_context, Path(id), RequestJson(payload)) = request;
    DeptScopeGuard::new(&state, data_scope).ensure_one(&id).await?;
    let audit = successful_operation_audit(audit_context)?;
    let dept = state.system_audited.replace_dept_with_audit(&id, payload, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(dept))
}
#[require_perms("system:dept:edit")]
pub async fn update_dept_sort(request: AuditedDeptJsonRequest<RequestJson<SortPayload>>) -> ApiResult<ApiJson<Dept>> {
    let (State(state), Extension(data_scope), audit_context, Path(id), RequestJson(payload)) = request;
    DeptScopeGuard::new(&state, data_scope).ensure_one(&id).await?;
    let audit = successful_operation_audit(audit_context)?;
    let dept = state.system_audited.update_dept_sort_with_audit(&id, payload.order_num, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(dept))
}

#[require_perms("system:dept:edit")]
pub async fn update_dept_sorts(request: AuditedDeptSortsRequest) -> ApiResult<ApiJson<Vec<Dept>>> {
    let (State(state), Extension(data_scope), audit_context, RequestJson(payload)) = request;
    let ids = payload.items.iter().map(|item| item.id.clone()).collect();
    DeptScopeGuard::new(&state, data_scope).ensure_many(ids).await?;
    let audit = successful_operation_audit(audit_context)?;
    let departments = state.system_audited.update_dept_sorts_with_audit(payload, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(departments))
}

#[require_perms("system:dept:remove")]
pub async fn delete_dept(request: AuditedDeptPathRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(data_scope), audit_context, Path(id)) = request;
    DeptScopeGuard::new(&state, data_scope).ensure_one(&id).await?;
    let audit = successful_operation_audit(audit_context)?;
    state.system_audited.delete_dept_with_audit(&id, audit.record()).await?;
    audit.mark_persisted();
    Ok(ok(()))
}

struct DeptScopeGuard<'a> {
    state: &'a SystemApiState,
    data_scope: DataScopeFilter,
}

impl<'a> DeptScopeGuard<'a> {
    const fn new(state: &'a SystemApiState, data_scope: DataScopeFilter) -> Self {
        Self { state, data_scope }
    }

    async fn ensure_one(&self, id: &str) -> ApiResult<()> {
        self.ensure_many(vec![id.into()]).await
    }

    async fn ensure_many(&self, ids: Vec<String>) -> ApiResult<()> {
        self.state.system.ensure_dept_ids_scoped(ids, self.data_scope.clone()).await?;
        Ok(())
    }
}
