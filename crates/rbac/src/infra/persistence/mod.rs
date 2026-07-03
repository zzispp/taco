mod apis;
mod bindings;
mod menus;
mod record;
mod repository;
mod roles;
mod types;

pub(super) use repository::RbacStore;
pub(super) use types::{
    ApiPermissionRecordInput, MenuItemRecordInput, MenuSectionRecordInput, RoleApiBindingRecordInput, RoleMenuBindingRecordInput, RoleRecordInput,
};

pub(super) use record::{ApiPermissionRecord, MenuItemRecord, MenuSectionRecord, RoleApiPermissionRecord, RoleMenuPermissionRecord, RoleRecord};

#[cfg(test)]
mod integration_tests;
