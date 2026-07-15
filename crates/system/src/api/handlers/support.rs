use super::*;

pub(super) fn all_depts_filter() -> DeptListFilter {
    DeptListFilter {
        page: CursorPageRequest::default(),
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
