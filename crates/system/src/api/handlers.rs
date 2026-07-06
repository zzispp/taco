use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    response::Response,
};
use kernel::pagination::{Page, PageRequest};
use rbac::api::CurrentUser;
use rbac_macros::{data_scope, require_perms};
use serde::Deserialize;
use types::http::{RequestJson, xlsx_attachment};
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
type ApiResult<T> = Result<T, SystemApiError>;
const EXPORT_PAGE_SIZE: u64 = 100;

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
pub async fn list_depts(
    State(state): State<SystemApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(data_scope): Extension<DataScopeFilter>,
    Query(query): Query<SystemListQuery>,
) -> ApiResult<ApiJson<Page<Dept>>> {
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
pub async fn get_dept(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<Dept>> {
    Ok(ok(state.system.get_dept(&id).await?))
}

#[require_perms("system:dept:list")]
#[data_scope(dept_alias = "d", user_alias = "u")]
pub async fn dept_tree_select(
    State(state): State<SystemApiState>,
    Extension(current_user): Extension<CurrentUser>,
    Extension(data_scope): Extension<DataScopeFilter>,
    Query(query): Query<DeptTreeQuery>,
) -> ApiResult<ApiJson<Vec<TreeSelectNode>>> {
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
pub async fn replace_dept(
    State(state): State<SystemApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<DeptInput>,
) -> ApiResult<ApiJson<Dept>> {
    Ok(ok(state.system.replace_dept(&id, payload).await?))
}
#[require_perms("system:dept:edit")]
pub async fn update_dept_sort(
    State(state): State<SystemApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<SortPayload>,
) -> ApiResult<ApiJson<Dept>> {
    Ok(ok(state.system.update_dept_sort(&id, payload.order_num).await?))
}

#[require_perms("system:dept:edit")]
pub async fn update_dept_sorts(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<SortBatchInput>) -> ApiResult<ApiJson<Vec<Dept>>> {
    Ok(ok(state.system.update_dept_sorts(payload).await?))
}

#[require_perms("system:dept:remove")]
pub async fn delete_dept(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.system.delete_dept(&id).await?;
    Ok(ok(()))
}

#[require_perms("system:post:export")]
pub async fn export_posts(State(state): State<SystemApiState>, Query(query): Query<SystemExportQuery>) -> ApiResult<Response> {
    let items = all_export_posts(&state, &query).await?;
    Ok(xlsx_attachment("posts.xlsx", export_posts_xlsx(&items)?))
}

#[require_perms("system:post:list")]
pub async fn list_posts(State(state): State<SystemApiState>, Query(query): Query<SystemListQuery>) -> ApiResult<ApiJson<Page<Post>>> {
    Ok(ok(state.system.page_posts(query.into()).await?))
}
#[require_perms("system:post:query")]
pub async fn get_post(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<Post>> {
    Ok(ok(state.system.get_post(&id).await?))
}

#[require_perms("system:post:list")]
pub async fn post_options(State(state): State<SystemApiState>) -> ApiResult<ApiJson<Vec<Post>>> {
    Ok(ok(state.system.post_options().await?))
}

#[require_perms("system:post:add")]
pub async fn create_post(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<PostInput>) -> ApiResult<ApiJson<Post>> {
    Ok(ok(state.system.create_post(payload).await?))
}

#[require_perms("system:post:edit")]
pub async fn replace_post(
    State(state): State<SystemApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<PostInput>,
) -> ApiResult<ApiJson<Post>> {
    Ok(ok(state.system.replace_post(&id, payload).await?))
}

#[require_perms("system:post:remove")]
pub async fn delete_post(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.system.delete_post(&id).await?;
    Ok(ok(()))
}

#[require_perms("system:post:remove")]
pub async fn delete_posts(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<BatchIdsInput>) -> ApiResult<ApiJson<()>> {
    state.system.delete_posts(payload.ids).await?;
    Ok(ok(()))
}

#[require_perms("system:dict:export")]
pub async fn export_dict_types(State(state): State<SystemApiState>, Query(query): Query<SystemExportQuery>) -> ApiResult<Response> {
    let items = all_export_dict_types(&state, &query).await?;
    Ok(xlsx_attachment("dict_types.xlsx", export_dict_types_xlsx(&items)?))
}

#[require_perms("system:dict:list")]
pub async fn list_dict_types(State(state): State<SystemApiState>, Query(query): Query<SystemListQuery>) -> ApiResult<ApiJson<Page<DictType>>> {
    Ok(ok(state.system.page_dict_types(query.into()).await?))
}
#[require_perms("system:dict:query")]
pub async fn get_dict_type(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<DictType>> {
    Ok(ok(state.system.get_dict_type(&id).await?))
}

#[require_perms("system:dict:list")]
pub async fn dict_type_options(State(state): State<SystemApiState>) -> ApiResult<ApiJson<Vec<DictType>>> {
    Ok(ok(state.system.dict_type_options().await?))
}

#[require_perms("system:dict:remove")]
pub async fn refresh_dict_cache(State(state): State<SystemApiState>) -> ApiResult<ApiJson<()>> {
    state.system.refresh_dict_cache().await?;
    Ok(ok(()))
}

#[require_perms("system:dict:add")]
pub async fn create_dict_type(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<DictTypeInput>) -> ApiResult<ApiJson<DictType>> {
    Ok(ok(state.system.create_dict_type(payload).await?))
}

#[require_perms("system:dict:edit")]
pub async fn replace_dict_type(
    State(state): State<SystemApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<DictTypeInput>,
) -> ApiResult<ApiJson<DictType>> {
    Ok(ok(state.system.replace_dict_type(&id, payload).await?))
}

#[require_perms("system:dict:remove")]
pub async fn delete_dict_type(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.system.delete_dict_type(&id).await?;
    Ok(ok(()))
}

#[require_perms("system:dict:remove")]
pub async fn delete_dict_types(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<BatchIdsInput>) -> ApiResult<ApiJson<()>> {
    state.system.delete_dict_types(payload.ids).await?;
    Ok(ok(()))
}

#[require_perms("system:dict:export")]
pub async fn export_dict_data(State(state): State<SystemApiState>, Query(query): Query<SystemExportQuery>) -> ApiResult<Response> {
    let items = all_export_dict_data(&state, &query).await?;
    Ok(xlsx_attachment("dict_data.xlsx", export_dict_data_xlsx(&items)?))
}

#[require_perms("system:dict:list")]
pub async fn list_dict_data(State(state): State<SystemApiState>, Query(query): Query<SystemListQuery>) -> ApiResult<ApiJson<Page<DictData>>> {
    Ok(ok(state.system.page_dict_data(query.into()).await?))
}
#[require_perms("system:dict:query")]
pub async fn get_dict_data(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<DictData>> {
    Ok(ok(state.system.get_dict_data(&id).await?))
}

#[require_perms("system:dict:list")]
pub async fn dict_data_by_type(State(state): State<SystemApiState>, Path(dict_type): Path<String>) -> ApiResult<ApiJson<Vec<DictData>>> {
    Ok(ok(state.system.dict_data_by_type(&dict_type).await?))
}

#[require_perms("system:dict:add")]
pub async fn create_dict_data(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<DictDataInput>) -> ApiResult<ApiJson<DictData>> {
    Ok(ok(state.system.create_dict_data(payload).await?))
}

#[require_perms("system:dict:edit")]
pub async fn replace_dict_data(
    State(state): State<SystemApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<DictDataInput>,
) -> ApiResult<ApiJson<DictData>> {
    Ok(ok(state.system.replace_dict_data(&id, payload).await?))
}

#[require_perms("system:dict:remove")]
pub async fn delete_dict_data(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.system.delete_dict_data(&id).await?;
    Ok(ok(()))
}

#[require_perms("system:dict:remove")]
pub async fn delete_dict_data_batch(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<BatchIdsInput>) -> ApiResult<ApiJson<()>> {
    state.system.delete_dict_data_batch(payload.ids).await?;
    Ok(ok(()))
}

#[require_perms("system:config:export")]
pub async fn export_configs(State(state): State<SystemApiState>, Query(query): Query<SystemExportQuery>) -> ApiResult<Response> {
    let items = all_export_configs(&state, &query).await?;
    Ok(xlsx_attachment("configs.xlsx", export_configs_xlsx(&items)?))
}

#[require_perms("system:config:list")]
pub async fn list_configs(State(state): State<SystemApiState>, Query(query): Query<SystemListQuery>) -> ApiResult<ApiJson<Page<ConfigItem>>> {
    Ok(ok(state.system.page_configs(query.into()).await?))
}
#[require_perms("system:config:query")]
pub async fn get_config(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<ConfigItem>> {
    Ok(ok(state.system.get_config(&id).await?))
}

#[require_perms("system:config:query")]
pub async fn config_by_key(State(state): State<SystemApiState>, Path(key): Path<String>) -> ApiResult<ApiJson<String>> {
    Ok(ok(state.system.config_by_key(&key).await?))
}

#[require_perms("system:config:remove")]
pub async fn refresh_config_cache(State(state): State<SystemApiState>) -> ApiResult<ApiJson<()>> {
    state.system.refresh_config_cache().await?;
    Ok(ok(()))
}

#[require_perms("system:config:add")]
pub async fn create_config(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<ConfigInput>) -> ApiResult<ApiJson<ConfigItem>> {
    Ok(ok(state.system.create_config(payload).await?))
}

#[require_perms("system:config:edit")]
pub async fn replace_config(
    State(state): State<SystemApiState>,
    Path(id): Path<String>,
    RequestJson(payload): RequestJson<ConfigInput>,
) -> ApiResult<ApiJson<ConfigItem>> {
    Ok(ok(state.system.replace_config(&id, payload).await?))
}

#[require_perms("system:config:remove")]
pub async fn delete_config(State(state): State<SystemApiState>, Path(id): Path<String>) -> ApiResult<ApiJson<()>> {
    state.system.delete_config(&id).await?;
    Ok(ok(()))
}

#[require_perms("system:config:remove")]
pub async fn delete_configs(State(state): State<SystemApiState>, RequestJson(payload): RequestJson<BatchIdsInput>) -> ApiResult<ApiJson<()>> {
    state.system.delete_configs(payload.ids).await?;
    Ok(ok(()))
}

impl From<SystemListQuery> for DeptListFilter {
    fn from(value: SystemListQuery) -> Self {
        Self {
            page: page(value.page, value.page_size),
            dept_name: value.dept_name,
            status: value.status,
            begin_time: value.begin_time,
            end_time: value.end_time,
        }
    }
}

impl From<DeptTreeQuery> for DeptListFilter {
    fn from(value: DeptTreeQuery) -> Self {
        Self {
            page: PageRequest { page: 1, page_size: 100_000 },
            dept_name: value.dept_name,
            status: value.status,
            begin_time: value.begin_time,
            end_time: value.end_time,
        }
    }
}

impl From<SystemListQuery> for PostListFilter {
    fn from(value: SystemListQuery) -> Self {
        Self {
            page: page(value.page, value.page_size),
            post_code: value.post_code,
            post_name: value.post_name,
            status: value.status,
        }
    }
}

impl From<SystemListQuery> for DictTypeListFilter {
    fn from(value: SystemListQuery) -> Self {
        Self {
            page: page(value.page, value.page_size),
            dict_name: value.dict_name,
            dict_type: value.dict_type,
            status: value.status,
            begin_time: value.begin_time,
            end_time: value.end_time,
        }
    }
}

impl From<SystemListQuery> for DictDataListFilter {
    fn from(value: SystemListQuery) -> Self {
        Self {
            page: page(value.page, value.page_size),
            dict_type: value.dict_type,
            dict_label: value.dict_label,
            status: value.status,
        }
    }
}

impl From<SystemListQuery> for ConfigListFilter {
    fn from(value: SystemListQuery) -> Self {
        Self {
            page: page(value.page, value.page_size),
            config_name: value.config_name,
            config_key: value.config_key,
            config_type: value.config_type,
            begin_time: value.begin_time,
            end_time: value.end_time,
        }
    }
}

async fn all_export_posts(state: &SystemApiState, query: &SystemExportQuery) -> ApiResult<Vec<Post>> {
    let mut page = 1;
    let mut items = Vec::new();
    loop {
        let current = state.system.page_posts(post_export_page(query, page, EXPORT_PAGE_SIZE)).await?;
        let is_last = current.items.is_empty() || items.len() + current.items.len() >= current.total as usize;
        items.extend(current.items);
        if is_last {
            return Ok(items);
        }
        page += 1;
    }
}

async fn all_export_dict_types(state: &SystemApiState, query: &SystemExportQuery) -> ApiResult<Vec<DictType>> {
    let mut page = 1;
    let mut items = Vec::new();
    loop {
        let current = state.system.page_dict_types(dict_type_export_page(query, page, EXPORT_PAGE_SIZE)).await?;
        let is_last = current.items.is_empty() || items.len() + current.items.len() >= current.total as usize;
        items.extend(current.items);
        if is_last {
            return Ok(items);
        }
        page += 1;
    }
}

async fn all_export_dict_data(state: &SystemApiState, query: &SystemExportQuery) -> ApiResult<Vec<DictData>> {
    let mut page = 1;
    let mut items = Vec::new();
    loop {
        let current = state.system.page_dict_data(dict_data_export_page(query, page, EXPORT_PAGE_SIZE)).await?;
        let is_last = current.items.is_empty() || items.len() + current.items.len() >= current.total as usize;
        items.extend(current.items);
        if is_last {
            return Ok(items);
        }
        page += 1;
    }
}

async fn all_export_configs(state: &SystemApiState, query: &SystemExportQuery) -> ApiResult<Vec<ConfigItem>> {
    let mut page = 1;
    let mut items = Vec::new();
    loop {
        let current = state.system.page_configs(config_export_page(query, page, EXPORT_PAGE_SIZE)).await?;
        let is_last = current.items.is_empty() || items.len() + current.items.len() >= current.total as usize;
        items.extend(current.items);
        if is_last {
            return Ok(items);
        }
        page += 1;
    }
}

fn page(page: u64, page_size: u64) -> PageRequest {
    PageRequest { page, page_size }
}

fn all_depts_filter() -> DeptListFilter {
    DeptListFilter {
        page: PageRequest { page: 1, page_size: 100_000 },
        dept_name: None,
        status: None,
        begin_time: None,
        end_time: None,
    }
}

fn checked_keys_for_tree(tree: &[TreeSelectNode], checked_keys: Vec<String>, strictly: bool) -> Vec<String> {
    if strictly {
        checked_keys.into_iter().filter(|key| tree_leaf_contains(tree, key)).collect()
    } else {
        checked_keys
    }
}

fn tree_leaf_contains(tree: &[TreeSelectNode], key: &str) -> bool {
    tree.iter().any(|node| {
        if node.id == key {
            return node.children.is_empty();
        }
        tree_leaf_contains(&node.children, key)
    })
}

fn config_keys(query: PublicConfigQuery) -> Vec<String> {
    query.keys.unwrap_or_default().split(',').map(str::to_owned).collect()
}

fn ok<T>(data: T) -> ApiJson<T> {
    Json(data)
}
