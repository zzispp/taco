use super::*;
use crate::application::RbacError;
use audit_contract::{AuditOutboxRecord, OperationAuditContext};

const MISSING_OPERATION_AUDIT_ACTOR: &str = "authenticated operation audit actor is missing";

pub(super) struct SuccessfulOperationAudit {
    context: OperationAuditContext,
    record: AuditOutboxRecord,
}

impl SuccessfulOperationAudit {
    pub(super) fn record(&self) -> AuditOutboxRecord {
        self.record.clone()
    }

    pub(super) fn mark_persisted(&self) {
        self.context.mark_persisted();
    }
}

pub(super) fn successful_operation_audit(context: OperationAuditContext) -> ApiResult<SuccessfulOperationAudit> {
    let record = context
        .success_record()
        .map_err(|error| RbacError::Infrastructure(error.to_string()))?
        .ok_or_else(|| RbacError::Infrastructure(MISSING_OPERATION_AUDIT_ACTOR.into()))?;
    Ok(SuccessfulOperationAudit { context, record })
}

pub(super) fn role_user_filter(role_id: String, query: RoleUsersQuery) -> RoleUserListFilter {
    RoleUserListFilter {
        page: CursorPageRequest {
            limit: query.limit,
            cursor: query.cursor,
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

pub(super) fn ok<T>(data: T) -> ApiJson<T> {
    Json(data)
}
