use super::*;
use kernel::error::LocalizedError;

use crate::application::RbacError;

pub(super) fn role_user_filter(role_id: String, query: RoleUsersQuery) -> RoleUserListFilter {
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

pub(super) fn menu_tree(menus: Vec<Menu>) -> Vec<types::system::TreeSelectNode> {
    menus.iter().filter(|menu| menu.parent_id == "0").map(|menu| menu_node(menu, &menus)).collect()
}

pub(super) fn menu_node(menu: &Menu, menus: &[Menu]) -> types::system::TreeSelectNode {
    types::system::TreeSelectNode {
        id: menu.menu_id.clone(),
        label: menu.menu_name.clone(),
        parent_id: menu.parent_id.clone(),
        disabled: menu.status != constants::system::STATUS_NORMAL,
        children: menus
            .iter()
            .filter(|child| child.parent_id == menu.menu_id)
            .map(|child| menu_node(child, menus))
            .collect(),
    }
}

pub(super) fn checked_keys_for_tree(tree: &[types::system::TreeSelectNode], checked_keys: Vec<String>, strictly: bool) -> Vec<String> {
    if strictly {
        checked_keys.into_iter().filter(|key| tree_leaf_contains(tree, key)).collect()
    } else {
        checked_keys
    }
}

pub(super) fn tree_leaf_contains(tree: &[types::system::TreeSelectNode], key: &str) -> bool {
    tree.iter().any(|node| {
        if node.id == key {
            return node.children.is_empty();
        }
        tree_leaf_contains(&node.children, key)
    })
}

pub(super) struct ExportRolesInput<'a> {
    pub(super) state: &'a RbacApiState,
    pub(super) current_user: &'a CurrentUser,
    pub(super) data_scope: DataScopeFilter,
    pub(super) filter: RoleExportFilter,
}

pub(super) async fn all_export_roles(input: ExportRolesInput<'_>) -> ApiResult<Vec<Role>> {
    let page_size = input.state.export_config.export_batch_config().await?.page_size;
    let mut page = PageRequest { page: 1, page_size };
    let mut roles = Vec::new();
    loop {
        let filter = input.filter.page_filter(page);
        let current = if input.current_user.admin {
            input.state.rbac_admin.page_roles(filter).await?
        } else {
            input.state.rbac_admin.page_roles_scoped(filter, input.data_scope.clone()).await?
        };
        let is_last = is_last_page(roles.len(), current.items.len(), current.total)?;
        roles.extend(current.items);
        if is_last {
            return Ok(roles);
        }
        advance_page(&mut page)?;
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

fn invalid_pagination() -> RbacApiError {
    RbacError::InvalidInput(LocalizedError::new("errors.validation.page_overflow")).into()
}

pub(super) fn ok<T>(data: T) -> ApiJson<T> {
    Json(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_pagination_overflow_is_explicit() {
        let Err(RbacApiError(RbacError::InvalidInput(loaded_error))) = is_last_page(usize::MAX, 1, u64::MAX) else {
            panic!("expected loaded row overflow");
        };
        assert_eq!(loaded_error.key(), "errors.validation.page_overflow");

        let mut page = PageRequest {
            page: u64::MAX,
            page_size: 100,
        };
        let Err(RbacApiError(RbacError::InvalidInput(page_error))) = advance_page(&mut page) else {
            panic!("expected page overflow");
        };
        assert_eq!(page_error.key(), "errors.validation.page_overflow");
    }
}
