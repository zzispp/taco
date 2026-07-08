use super::*;

impl From<RbacListQuery> for RoleListFilter {
    fn from(value: RbacListQuery) -> Self {
        Self {
            page: PageRequest {
                page: value.page,
                page_size: value.page_size,
            },
            role_name: value.role_name,
            role_key: value.role_key,
            status: value.status,
            system: value.system,
            begin_time: value.begin_time,
            end_time: value.end_time,
        }
    }
}

impl From<RbacListQuery> for MenuListFilter {
    fn from(value: RbacListQuery) -> Self {
        Self {
            page: PageRequest {
                page: value.page,
                page_size: value.page_size,
            },
            menu_name: value.menu_name,
            status: value.status,
        }
    }
}

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
    pub(super) query: &'a RoleExportQuery,
}

pub(super) async fn all_export_roles(input: ExportRolesInput<'_>) -> ApiResult<Vec<Role>> {
    let export_page_size = input.state.export_config.export_batch_config().await?;
    let mut page = 1;
    let mut roles = Vec::new();
    loop {
        let filter = role_export_page(input.query, page, export_page_size.page_size);
        let current = if input.current_user.admin {
            input.state.rbac_admin.page_roles(filter).await?
        } else {
            input.state.rbac_admin.page_roles_scoped(filter, input.data_scope.clone()).await?
        };
        let is_last = current.items.is_empty() || roles.len() + current.items.len() >= current.total as usize;
        roles.extend(current.items);
        if is_last {
            return Ok(roles);
        }
        page += 1;
    }
}

pub(super) fn ok<T>(data: T) -> ApiJson<T> {
    Json(data)
}
