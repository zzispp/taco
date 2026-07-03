use async_trait::async_trait;
use storage::{Database, StorageError};
use crate::application::{RbacError, RbacRepository, RbacResult};
use crate::domain::{
    ApiPermission, ApiPermissionInput, ApiPermissionSnapshot, MenuItem, MenuItemInput, MenuSection, MenuSectionInput, NavItemResponse, NavSectionResponse,
    PermissionSnapshot, Role, RoleInput, RoleMenuBindingInput, RoleMenuSnapshot,
};
use kernel::pagination::{Page, PageRequest};

use super::persistence::{
    ApiPermissionRecordInput, MenuItemRecordInput, MenuSectionRecordInput, RbacStore, RoleApiBindingRecordInput, RoleMenuBindingRecordInput,
    RoleRecordInput,
};
use kernel::pagination::PageSliceRequest;

#[derive(Clone)]
pub struct StorageRbacRepository {
    store: RbacStore,
}

impl StorageRbacRepository {
    pub fn new(database: Database) -> Self {
        Self {
            store: RbacStore::new(database),
        }
    }
}

#[async_trait]
impl RbacRepository for StorageRbacRepository {
    async fn create_role(&self, input: RoleInput) -> RbacResult<Role> {
        self.store.create_role(role_record_input(input, false)).await.map_err(storage_error)
    }

    async fn create_system_role(&self, input: RoleInput) -> RbacResult<Role> {
        self.store.create_role(role_record_input(input, true)).await.map_err(storage_error)
    }

    async fn replace_role(&self, code: &str, input: RoleInput) -> RbacResult<Role> {
        self.store.replace_role(code, role_record_input(input, false)).await.map_err(storage_error)
    }

    async fn replace_system_role(&self, code: &str, input: RoleInput) -> RbacResult<Role> {
        self.store.replace_role(code, role_record_input(input, true)).await.map_err(storage_error)
    }

    async fn delete_role(&self, code: &str) -> RbacResult<()> {
        self.store.delete_role(code).await.map_err(storage_error)
    }

    async fn find_role(&self, code: &str) -> RbacResult<Option<Role>> {
        self.store.find_role(code).await.map_err(storage_error)
    }

    async fn role_has_api_bindings(&self, code: &str) -> RbacResult<bool> {
        self.store.role_has_api_bindings(code).await.map_err(storage_error)
    }

    async fn role_has_menu_bindings(&self, code: &str) -> RbacResult<bool> {
        self.store.role_has_menu_bindings(code).await.map_err(storage_error)
    }

    async fn role_has_users(&self, code: &str) -> RbacResult<bool> {
        self.store.role_has_users(code).await.map_err(storage_error)
    }

    async fn list_roles(&self) -> RbacResult<Vec<Role>> {
        self.store.list_roles().await.map_err(storage_error)
    }

    async fn page_roles(&self, page: PageRequest) -> RbacResult<Page<Role>> {
        self.store.page_roles(page_request(page)).await.map_err(storage_error)
    }

    async fn create_api(&self, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        self.store.create_api(api_record_input(input, false)).await.map_err(storage_error)
    }

    async fn replace_api(&self, id: &str, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        self.store.replace_api(id, api_record_input(input, false)).await.map_err(storage_error)
    }

    async fn delete_api(&self, id: &str) -> RbacResult<()> {
        self.store.delete_api(id).await.map_err(storage_error)
    }

    async fn find_api(&self, id: &str) -> RbacResult<Option<ApiPermission>> {
        self.store.find_api(id).await.map_err(storage_error)
    }

    async fn api_has_role_bindings(&self, id: &str) -> RbacResult<bool> {
        self.store.api_has_role_bindings(id).await.map_err(storage_error)
    }

    async fn list_apis(&self) -> RbacResult<Vec<ApiPermission>> {
        self.store.list_apis().await.map_err(storage_error)
    }

    async fn page_apis(&self, page: PageRequest) -> RbacResult<Page<ApiPermission>> {
        self.store.page_apis(page_request(page)).await.map_err(storage_error)
    }

    async fn create_menu_section(&self, input: MenuSectionInput) -> RbacResult<MenuSection> {
        self.store.create_menu_section(menu_section_record_input(input)).await.map_err(storage_error)
    }

    async fn replace_menu_section(&self, id: &str, input: MenuSectionInput) -> RbacResult<MenuSection> {
        self.store
            .replace_menu_section(id, menu_section_record_input(input))
            .await
            .map_err(storage_error)
    }

    async fn delete_menu_section(&self, id: &str) -> RbacResult<()> {
        self.store.delete_menu_section(id).await.map_err(storage_error)
    }

    async fn find_menu_section(&self, id: &str) -> RbacResult<Option<MenuSection>> {
        self.store.find_menu_section(id).await.map_err(storage_error)
    }

    async fn menu_section_has_items(&self, id: &str) -> RbacResult<bool> {
        self.store.menu_section_has_items(id).await.map_err(storage_error)
    }

    async fn page_menu_sections(&self, page: PageRequest) -> RbacResult<Page<MenuSection>> {
        self.store.page_menu_sections(page_request(page)).await.map_err(storage_error)
    }

    async fn create_menu_item(&self, input: MenuItemInput) -> RbacResult<MenuItem> {
        self.store.create_menu_item(menu_item_record_input(input)).await.map_err(storage_error)
    }

    async fn replace_menu_item(&self, id: &str, input: MenuItemInput) -> RbacResult<MenuItem> {
        self.store.replace_menu_item(id, menu_item_record_input(input)).await.map_err(storage_error)
    }

    async fn delete_menu_item(&self, id: &str) -> RbacResult<()> {
        self.store.delete_menu_item(id).await.map_err(storage_error)
    }

    async fn find_menu_item(&self, id: &str) -> RbacResult<Option<MenuItem>> {
        self.store.find_menu_item(id).await.map_err(storage_error)
    }

    async fn menu_item_has_children(&self, id: &str) -> RbacResult<bool> {
        self.store.menu_item_has_children(id).await.map_err(storage_error)
    }

    async fn menu_item_has_role_bindings(&self, id: &str) -> RbacResult<bool> {
        self.store.menu_item_has_role_bindings(id).await.map_err(storage_error)
    }

    async fn list_menu_items(&self) -> RbacResult<Vec<MenuItem>> {
        self.store.list_menu_items().await.map_err(storage_error)
    }

    async fn page_menu_items(&self, page: PageRequest) -> RbacResult<Page<MenuItem>> {
        self.store.page_menu_items(page_request(page)).await.map_err(storage_error)
    }

    async fn replace_role_apis(&self, role_code: &str, api_permission_ids: Vec<String>) -> RbacResult<()> {
        let inputs = api_permission_ids
            .into_iter()
            .map(|api_permission_id| RoleApiBindingRecordInput {
                role_code: role_code.into(),
                api_permission_id,
            })
            .collect();
        self.store.replace_role_apis(role_code, inputs).await.map_err(storage_error)
    }

    async fn replace_role_menus(&self, role_code: &str, input: RoleMenuBindingInput) -> RbacResult<()> {
        let inputs = input
            .menu_item_ids
            .into_iter()
            .map(|menu_item_id| RoleMenuBindingRecordInput {
                role_code: role_code.into(),
                menu_item_id,
            })
            .collect();
        self.store.replace_role_menus(role_code, inputs).await.map_err(storage_error)
    }

    async fn role_api_ids(&self, role_code: &str) -> RbacResult<Vec<String>> {
        self.store.role_api_ids(role_code).await.map_err(storage_error)
    }

    async fn role_menu_item_ids(&self, role_code: &str) -> RbacResult<Vec<String>> {
        self.store.role_menu_item_ids(role_code).await.map_err(storage_error)
    }

    async fn permission_snapshot(&self) -> RbacResult<PermissionSnapshot> {
        let roles = self.store.list_roles().await.map_err(storage_error)?;
        let apis = self.store.list_apis().await.map_err(storage_error)?;
        let sections = self.store.list_menu_sections().await.map_err(storage_error)?;
        let items = self.store.list_menu_items().await.map_err(storage_error)?;
        let api_bindings = self.store.list_role_api_bindings().await.map_err(storage_error)?;
        let menu_bindings = self.store.list_role_menu_bindings().await.map_err(storage_error)?;
        Ok(PermissionSnapshot {
            api_permissions: api_snapshots(apis, roles, api_bindings),
            menus: menu_snapshots(sections, items, menu_bindings),
        })
    }
}

fn page_request(page: PageRequest) -> PageSliceRequest {
    PageSliceRequest {
        offset: (page.page - 1) * page.page_size,
        limit: page.page_size,
        page: page.page,
        page_size: page.page_size,
    }
}

fn role_record_input(input: RoleInput, system: bool) -> RoleRecordInput {
    RoleRecordInput {
        code: input.code,
        name: input.name,
        description: input.description,
        enabled: input.enabled,
        system,
        sort_order: input.sort_order,
    }
}

fn api_record_input(input: ApiPermissionInput, system: bool) -> ApiPermissionRecordInput {
    ApiPermissionRecordInput {
        code: input.code,
        method: input.method,
        path_pattern: input.path_pattern,
        name: input.name,
        group: input.group,
        enabled: input.enabled,
        system,
    }
}

fn api_snapshots(apis: Vec<ApiPermission>, roles: Vec<Role>, bindings: Vec<RoleApiBindingRecordInput>) -> Vec<ApiPermissionSnapshot> {
    apis.into_iter()
        .filter(|api| api.enabled)
        .map(|api| ApiPermissionSnapshot {
            role_codes: role_codes_for_api(&api.id, &roles, &bindings),
            method: api.method,
            path_pattern: api.path_pattern,
        })
        .collect()
}

fn role_codes_for_api(id: &str, roles: &[Role], bindings: &[RoleApiBindingRecordInput]) -> Vec<String> {
    bindings
        .iter()
        .filter(|binding| binding.api_permission_id == id)
        .filter(|binding| role_enabled(roles, &binding.role_code))
        .map(|binding| binding.role_code.clone())
        .collect()
}

fn menu_snapshots(sections: Vec<MenuSection>, items: Vec<MenuItem>, bindings: Vec<RoleMenuBindingRecordInput>) -> Vec<RoleMenuSnapshot> {
    role_codes_for_menus(&bindings)
        .into_iter()
        .map(|role_code| RoleMenuSnapshot {
            sections: sections_for_role(&role_code, &sections, &items, &bindings),
            role_code,
        })
        .collect()
}

fn role_codes_for_menus(bindings: &[RoleMenuBindingRecordInput]) -> Vec<String> {
    let mut role_codes = bindings.iter().map(|binding| binding.role_code.clone()).collect::<Vec<_>>();
    role_codes.sort();
    role_codes.dedup();
    role_codes
}

fn sections_for_role(role_code: &str, sections: &[MenuSection], items: &[MenuItem], bindings: &[RoleMenuBindingRecordInput]) -> Vec<NavSectionResponse> {
    sections
        .iter()
        .filter(|section| section.enabled)
        .filter_map(|section| nav_section_for_role(role_code, section, items, bindings))
        .collect()
}

fn nav_section_for_role(role_code: &str, section: &MenuSection, items: &[MenuItem], bindings: &[RoleMenuBindingRecordInput]) -> Option<NavSectionResponse> {
    let nav_items = child_items(role_code, &section.id, None, items, bindings);
    if nav_items.is_empty() {
        return None;
    }
    Some(NavSectionResponse {
        code: section.code.clone(),
        subheader: section.subheader.clone(),
        items: nav_items,
    })
}

fn child_items(
    role_code: &str,
    section_id: &str,
    parent_id: Option<&str>,
    items: &[MenuItem],
    bindings: &[RoleMenuBindingRecordInput],
) -> Vec<NavItemResponse> {
    items
        .iter()
        .filter(|item| item.section_id == section_id && item.parent_id.as_deref() == parent_id)
        .filter(|item| item.enabled && menu_bound(role_code, &item.id, bindings))
        .map(|item| nav_item(role_code, item, items, bindings))
        .collect()
}

fn nav_item(role_code: &str, item: &MenuItem, items: &[MenuItem], bindings: &[RoleMenuBindingRecordInput]) -> NavItemResponse {
    NavItemResponse {
        code: item.code.clone(),
        title: item.title.clone(),
        path: item.path.clone(),
        icon: item.icon.clone(),
        caption: item.caption.clone(),
        deep_match: item.deep_match,
        children: child_items(role_code, &item.section_id, Some(item.id.as_str()), items, bindings),
    }
}

fn role_enabled(roles: &[Role], code: &str) -> bool {
    roles.iter().any(|role| role.code == code && role.enabled)
}

fn menu_bound(role_code: &str, item_id: &str, bindings: &[RoleMenuBindingRecordInput]) -> bool {
    bindings.iter().any(|binding| binding.role_code == role_code && binding.menu_item_id == item_id)
}

fn storage_error(error: StorageError) -> RbacError {
    match error {
        StorageError::NotFound => RbacError::NotFound,
        StorageError::Conflict(message) => RbacError::Conflict(message),
        StorageError::Database(message) => RbacError::Infrastructure(message),
    }
}

fn menu_section_record_input(input: MenuSectionInput) -> MenuSectionRecordInput {
    MenuSectionRecordInput {
        code: input.code,
        subheader: input.subheader,
        sort_order: input.sort_order,
        enabled: input.enabled,
    }
}

fn menu_item_record_input(input: MenuItemInput) -> MenuItemRecordInput {
    MenuItemRecordInput {
        section_id: input.section_id,
        parent_id: input.parent_id,
        code: input.code,
        title: input.title,
        path: input.path,
        icon: input.icon,
        caption: input.caption,
        deep_match: input.deep_match,
        sort_order: input.sort_order,
        enabled: input.enabled,
    }
}
