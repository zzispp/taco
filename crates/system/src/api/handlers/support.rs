use super::*;
use kernel::error::LocalizedError;

use crate::api::input::DEPT_TREE_PAGE_SIZE;
use crate::application::SystemError;

pub(super) async fn all_export_posts(state: &SystemApiState, filter: PostExportFilter) -> ApiResult<Vec<Post>> {
    let mut items = Vec::new();
    let page_size = state.export_config.export_batch_config().await?.page_size;
    let mut page = PageRequest { page: 1, page_size };
    loop {
        let current = state.system.page_posts(filter.page_filter(page)).await?;
        let is_last = is_last_page(items.len(), current.items.len(), current.total)?;
        items.extend(current.items);
        if is_last {
            return Ok(items);
        }
        advance_page(&mut page)?;
    }
}

pub(super) async fn all_export_dict_types(state: &SystemApiState, filter: DictTypeExportFilter) -> ApiResult<Vec<DictType>> {
    let mut items = Vec::new();
    let page_size = state.export_config.export_batch_config().await?.page_size;
    let mut page = PageRequest { page: 1, page_size };
    loop {
        let current = state.system.page_dict_types(filter.page_filter(page)).await?;
        let is_last = is_last_page(items.len(), current.items.len(), current.total)?;
        items.extend(current.items);
        if is_last {
            return Ok(items);
        }
        advance_page(&mut page)?;
    }
}

pub(super) async fn all_export_dict_data(state: &SystemApiState, filter: DictDataExportFilter) -> ApiResult<Vec<DictData>> {
    let mut items = Vec::new();
    let page_size = state.export_config.export_batch_config().await?.page_size;
    let mut page = PageRequest { page: 1, page_size };
    loop {
        let current = state.system.page_dict_data(filter.page_filter(page)).await?;
        let is_last = is_last_page(items.len(), current.items.len(), current.total)?;
        items.extend(current.items);
        if is_last {
            return Ok(items);
        }
        advance_page(&mut page)?;
    }
}

pub(super) async fn all_export_configs(state: &SystemApiState, filter: ConfigExportFilter) -> ApiResult<Vec<ConfigItem>> {
    let mut items = Vec::new();
    let page_size = state.export_config.export_batch_config().await?.page_size;
    let mut page = PageRequest { page: 1, page_size };
    loop {
        let current = state.system.page_configs(filter.page_filter(page)).await?;
        let is_last = is_last_page(items.len(), current.items.len(), current.total)?;
        items.extend(current.items);
        if is_last {
            return Ok(items);
        }
        advance_page(&mut page)?;
    }
}

pub(super) fn all_depts_filter() -> DeptListFilter {
    DeptListFilter {
        page: PageRequest {
            page: 1,
            page_size: DEPT_TREE_PAGE_SIZE,
        },
        dept_name: None,
        leader: None,
        phone: None,
        email: None,
        status: None,
        begin_time: None,
        end_time: None,
    }
}

fn is_last_page(existing: usize, current: usize, total: u64) -> ApiResult<bool> {
    let loaded = existing.checked_add(current).ok_or_else(invalid_pagination)?;
    let total = usize::try_from(total).map_err(|_| invalid_pagination())?;
    Ok(current == 0 || loaded >= total)
}

fn advance_page(page: &mut PageRequest) -> ApiResult<()> {
    page.page = page.page.checked_add(1).ok_or_else(invalid_pagination)?;
    Ok(())
}

fn invalid_pagination() -> SystemApiError {
    SystemError::InvalidInput(LocalizedError::new("errors.common.invalid_input")).into()
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
