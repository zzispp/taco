use super::*;

impl From<SystemListQuery> for DeptListFilter {
    fn from(value: SystemListQuery) -> Self {
        Self {
            page: page(value.page, value.page_size),
            dept_name: value.dept_name,
            leader: value.leader,
            phone: value.phone,
            email: value.email,
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
            leader: value.leader,
            phone: value.phone,
            email: value.email,
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
            remark: value.remark,
            begin_time: value.begin_time,
            end_time: value.end_time,
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
            public_read: value.public_read,
            begin_time: value.begin_time,
            end_time: value.end_time,
        }
    }
}

pub(super) async fn all_export_posts(state: &SystemApiState, query: &SystemExportQuery) -> ApiResult<Vec<Post>> {
    let mut page = 1;
    let mut items = Vec::new();
    let page_size = state.export_config.export_batch_config().await?.page_size;
    loop {
        let current = state.system.page_posts(post_export_page(query, page, page_size)).await?;
        let is_last = current.items.is_empty() || items.len() + current.items.len() >= current.total as usize;
        items.extend(current.items);
        if is_last {
            return Ok(items);
        }
        page += 1;
    }
}

pub(super) async fn all_export_dict_types(state: &SystemApiState, query: &SystemExportQuery) -> ApiResult<Vec<DictType>> {
    let mut page = 1;
    let mut items = Vec::new();
    let page_size = state.export_config.export_batch_config().await?.page_size;
    loop {
        let current = state.system.page_dict_types(dict_type_export_page(query, page, page_size)).await?;
        let is_last = current.items.is_empty() || items.len() + current.items.len() >= current.total as usize;
        items.extend(current.items);
        if is_last {
            return Ok(items);
        }
        page += 1;
    }
}

pub(super) async fn all_export_dict_data(state: &SystemApiState, query: &SystemExportQuery) -> ApiResult<Vec<DictData>> {
    let mut page = 1;
    let mut items = Vec::new();
    let page_size = state.export_config.export_batch_config().await?.page_size;
    loop {
        let current = state.system.page_dict_data(dict_data_export_page(query, page, page_size)).await?;
        let is_last = current.items.is_empty() || items.len() + current.items.len() >= current.total as usize;
        items.extend(current.items);
        if is_last {
            return Ok(items);
        }
        page += 1;
    }
}

pub(super) async fn all_export_configs(state: &SystemApiState, query: &SystemExportQuery) -> ApiResult<Vec<ConfigItem>> {
    let mut page = 1;
    let mut items = Vec::new();
    let page_size = state.export_config.export_batch_config().await?.page_size;
    loop {
        let current = state.system.page_configs(config_export_page(query, page, page_size)).await?;
        let is_last = current.items.is_empty() || items.len() + current.items.len() >= current.total as usize;
        items.extend(current.items);
        if is_last {
            return Ok(items);
        }
        page += 1;
    }
}

pub(super) fn page(page: u64, page_size: u64) -> PageRequest {
    PageRequest { page, page_size }
}

pub(super) fn all_depts_filter() -> DeptListFilter {
    DeptListFilter {
        page: PageRequest { page: 1, page_size: 100_000 },
        dept_name: None,
        leader: None,
        phone: None,
        email: None,
        status: None,
        begin_time: None,
        end_time: None,
    }
}

pub(super) fn checked_keys_for_tree(tree: &[TreeSelectNode], checked_keys: Vec<String>, strictly: bool) -> Vec<String> {
    if strictly {
        checked_keys.into_iter().filter(|key| tree_leaf_contains(tree, key)).collect()
    } else {
        checked_keys
    }
}

pub(super) fn tree_leaf_contains(tree: &[TreeSelectNode], key: &str) -> bool {
    tree.iter().any(|node| {
        if node.id == key {
            return node.children.is_empty();
        }
        tree_leaf_contains(&node.children, key)
    })
}

pub(super) fn config_keys(query: PublicConfigQuery) -> Vec<String> {
    query.keys.unwrap_or_default().split(',').map(str::to_owned).collect()
}

pub(super) fn ok<T>(data: T) -> ApiJson<T> {
    Json(data)
}
