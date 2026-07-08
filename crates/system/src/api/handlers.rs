use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    response::Response,
};
use kernel::pagination::{Page, PageRequest};
use rbac::api::CurrentUser;
use rbac_macros::{data_scope, require_perms};
use serde::Deserialize;
use types::http::{RequestJson, current_locale, xlsx_attachment};
use types::rbac::{DataScopeFilter, RoleDeptTreeSelect};
use types::system::BatchIdsInput;

use crate::{
    api::{
        SystemApiError, SystemApiState,
        export::{
            config_export_page, dict_data_export_page, dict_type_export_page, export_configs_xlsx, export_dict_data_xlsx, export_dict_types_xlsx,
            export_posts_xlsx, post_export_page,
        },
    },
    application::{ConfigListFilter, DeptListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter},
    domain::{ConfigInput, ConfigItem, Dept, DeptInput, DictData, DictDataInput, DictType, DictTypeInput, Post, PostInput, SortBatchInput, TreeSelectNode},
};

type ApiJson<T> = Json<T>;

mod resources;
mod support;

pub use resources::{
    config_by_key, create_config, create_dict_data, create_dict_type, create_post, delete_config, delete_configs, delete_dict_data, delete_dict_data_batch,
    delete_dict_type, delete_dict_types, delete_post, delete_posts, dict_data_by_type, dict_type_options, export_configs, export_dict_data, export_dict_types,
    export_posts, get_config, get_dict_data, get_dict_type, get_post, list_configs, list_dict_data, list_dict_types, list_posts, post_options,
    refresh_config_cache, refresh_dict_cache, replace_config, replace_dict_data, replace_dict_type, replace_post,
};

use self::support::{
    all_depts_filter, all_export_configs, all_export_dict_data, all_export_dict_types, all_export_posts, checked_keys_for_tree, config_keys, ok,
};

type ListDeptsRequest = (
    State<SystemApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    Query<SystemListQuery>,
);
type DeptTreeRequest = (State<SystemApiState>, Extension<CurrentUser>, Extension<DataScopeFilter>, Query<DeptTreeQuery>);
type DeptPathRequest = (State<SystemApiState>, Extension<CurrentUser>, Extension<DataScopeFilter>, Path<String>);
type DeptJsonRequest<T> = (State<SystemApiState>, Extension<CurrentUser>, Extension<DataScopeFilter>, Path<String>, T);
type DeptSortsRequest = (
    State<SystemApiState>,
    Extension<CurrentUser>,
    Extension<DataScopeFilter>,
    RequestJson<SortBatchInput>,
);
type ApiResult<T> = Result<T, SystemApiError>;

#[derive(Debug, Deserialize)]
pub struct SystemListQuery {
    pub page: u64,
    pub page_size: u64,
    pub dept_name: Option<String>,
    pub post_code: Option<String>,
    pub post_name: Option<String>,
    pub dict_name: Option<String>,
    pub dict_type: Option<String>,
    pub dict_label: Option<String>,
    pub config_name: Option<String>,
    pub config_key: Option<String>,
    pub config_type: Option<String>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SystemExportQuery {
    pub post_code: Option<String>,
    pub post_name: Option<String>,
    pub dict_name: Option<String>,
    pub dict_type: Option<String>,
    pub dict_label: Option<String>,
    pub config_name: Option<String>,
    pub config_key: Option<String>,
    pub config_type: Option<String>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct DeptTreeQuery {
    pub dept_name: Option<String>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

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
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn list_depts(request: ListDeptsRequest) -> ApiResult<ApiJson<Page<Dept>>> {
    let (State(state), Extension(current_user), Extension(data_scope), Query(query)) = request;
    let page = if current_user.admin {
        state.system.page_depts(query.into()).await?
    } else {
        state.system.page_depts_scoped(query.into(), data_scope).await?
    };
    Ok(ok(page))
}

#[require_perms("system:dept:add")]
pub async fn create_dept(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<DeptInput>) -> ApiResult<ApiJson<Dept>> {
    Ok(ok(state.system.create_dept(payload).await?))
}
#[require_perms("system:dept:query")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn get_dept(request: DeptPathRequest) -> ApiResult<ApiJson<Dept>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id)) = request;
    DeptScopeGuard::new(&state, &current_user, data_scope).ensure_one(&id).await?;
    Ok(ok(state.system.get_dept(&id).await?))
}

#[require_perms("system:dept:list")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn dept_tree_select(request: DeptTreeRequest) -> ApiResult<ApiJson<Vec<TreeSelectNode>>> {
    let (State(state), Extension(current_user), Extension(data_scope), Query(query)) = request;
    let scope = (!current_user.admin).then_some(data_scope);
    Ok(ok(state.system.dept_tree(query.into(), scope).await?))
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
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn replace_dept(request: DeptJsonRequest<RequestJson<DeptInput>>) -> ApiResult<ApiJson<Dept>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id), RequestJson(payload)) = request;
    DeptScopeGuard::new(&state, &current_user, data_scope).ensure_one(&id).await?;
    Ok(ok(state.system.replace_dept(&id, payload).await?))
}
#[require_perms("system:dept:edit")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn update_dept_sort(request: DeptJsonRequest<RequestJson<SortPayload>>) -> ApiResult<ApiJson<Dept>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id), RequestJson(payload)) = request;
    DeptScopeGuard::new(&state, &current_user, data_scope).ensure_one(&id).await?;
    Ok(ok(state.system.update_dept_sort(&id, payload.order_num).await?))
}

#[require_perms("system:dept:edit")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn update_dept_sorts(request: DeptSortsRequest) -> ApiResult<ApiJson<Vec<Dept>>> {
    let (State(state), Extension(current_user), Extension(data_scope), RequestJson(payload)) = request;
    let ids = payload.items.iter().map(|item| item.id.clone()).collect();
    DeptScopeGuard::new(&state, &current_user, data_scope).ensure_many(ids).await?;
    Ok(ok(state.system.update_dept_sorts(payload).await?))
}

#[require_perms("system:dept:remove")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn delete_dept(request: DeptPathRequest) -> ApiResult<ApiJson<()>> {
    let (State(state), Extension(current_user), Extension(data_scope), Path(id)) = request;
    DeptScopeGuard::new(&state, &current_user, data_scope).ensure_one(&id).await?;
    state.system.delete_dept(&id).await?;
    Ok(ok(()))
}

struct DeptScopeGuard<'a> {
    state: &'a SystemApiState,
    current_user: &'a CurrentUser,
    data_scope: DataScopeFilter,
}

impl<'a> DeptScopeGuard<'a> {
    const fn new(state: &'a SystemApiState, current_user: &'a CurrentUser, data_scope: DataScopeFilter) -> Self {
        Self {
            state,
            current_user,
            data_scope,
        }
    }

    async fn ensure_one(&self, id: &str) -> ApiResult<()> {
        self.ensure_many(vec![id.into()]).await
    }

    async fn ensure_many(&self, ids: Vec<String>) -> ApiResult<()> {
        if self.current_user.admin {
            return Ok(());
        }
        self.state.system.ensure_dept_ids_scoped(ids, self.data_scope.clone()).await?;
        Ok(())
    }
}
