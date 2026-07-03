use kernel::pagination::{Page, PageRequest};

use crate::application::{ApiCheckRequest, AuthorizationConfig, RbacCache, RbacError, RbacRepository, RbacResult};
use crate::domain::{
    ApiPermission, ApiPermissionInput, MenuItem, MenuItemInput, MenuSection, MenuSectionInput, NavResponse, Role, RoleApiBindingInput, RoleInput,
    RoleMenuBindingInput,
};

use self::{
    authz::{authorize_snapshot, is_whitelisted},
    validation::{sanitize_api, sanitize_menu_item, sanitize_menu_section, sanitize_role, validate_page},
};

mod authz;
mod use_cases;
mod validation;

pub struct RbacService<R, C> {
    repository: R,
    cache: C,
}

impl<R, C> RbacService<R, C>
where
    R: RbacRepository,
    C: RbacCache,
{
    pub const fn new(repository: R, cache: C) -> Self {
        Self { repository, cache }
    }

    pub async fn create_role(&self, input: RoleInput) -> RbacResult<Role> {
        let role = self.repository.create_role(sanitize_role(input)?).await?;
        self.rebuild_cache().await?;
        Ok(role)
    }

    pub async fn ensure_system_role(&self, input: RoleInput) -> RbacResult<Role> {
        let input = sanitize_role(input)?;
        let code = input.code.clone();
        let role = match self.repository.find_role(&code).await? {
            Some(_) => self.repository.replace_system_role(&code, input).await?,
            None => self.repository.create_system_role(input).await?,
        };
        self.rebuild_cache().await?;
        Ok(role)
    }

    pub async fn replace_role(&self, code: &str, input: RoleInput) -> RbacResult<Role> {
        reject_system_role_update(&self.repository, code).await?;
        let role = self.repository.replace_role(code, sanitize_role(input)?).await?;
        self.rebuild_cache().await?;
        Ok(role)
    }

    pub async fn delete_role(&self, code: &str) -> RbacResult<()> {
        reject_system_role_update(&self.repository, code).await?;
        reject_bound_role_delete(&self.repository, code).await?;
        self.repository.delete_role(code).await?;
        self.rebuild_cache().await
    }

    pub async fn list_roles(&self) -> RbacResult<Vec<Role>> {
        self.repository.list_roles().await
    }

    pub async fn page_roles(&self, page: PageRequest) -> RbacResult<Page<Role>> {
        validate_page(page)?;
        self.repository.page_roles(page).await
    }

    pub async fn create_api(&self, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        let api = self.repository.create_api(sanitize_api(input)?).await?;
        self.rebuild_cache().await?;
        Ok(api)
    }

    pub async fn replace_api(&self, id: &str, input: ApiPermissionInput) -> RbacResult<ApiPermission> {
        let api = self.repository.replace_api(id, sanitize_api(input)?).await?;
        self.rebuild_cache().await?;
        Ok(api)
    }

    pub async fn delete_api(&self, id: &str) -> RbacResult<()> {
        ensure_api_permission_exists(&self.repository, id).await?;
        reject_bound_api_delete(&self.repository, id).await?;
        self.repository.delete_api(id).await?;
        self.rebuild_cache().await
    }

    pub async fn list_apis(&self) -> RbacResult<Vec<ApiPermission>> {
        self.repository.list_apis().await
    }

    pub async fn page_apis(&self, page: PageRequest) -> RbacResult<Page<ApiPermission>> {
        validate_page(page)?;
        self.repository.page_apis(page).await
    }

    pub async fn create_menu_section(&self, input: MenuSectionInput) -> RbacResult<MenuSection> {
        let section = self.repository.create_menu_section(sanitize_menu_section(input)?).await?;
        self.rebuild_cache().await?;
        Ok(section)
    }

    pub async fn replace_menu_section(&self, id: &str, input: MenuSectionInput) -> RbacResult<MenuSection> {
        let section = self.repository.replace_menu_section(id, sanitize_menu_section(input)?).await?;
        self.rebuild_cache().await?;
        Ok(section)
    }

    pub async fn delete_menu_section(&self, id: &str) -> RbacResult<()> {
        ensure_menu_section_exists(&self.repository, id).await?;
        reject_non_empty_menu_section_delete(&self.repository, id).await?;
        self.repository.delete_menu_section(id).await?;
        self.rebuild_cache().await
    }

    pub async fn page_menu_sections(&self, page: PageRequest) -> RbacResult<Page<MenuSection>> {
        validate_page(page)?;
        self.repository.page_menu_sections(page).await
    }

    pub async fn create_menu_item(&self, input: MenuItemInput) -> RbacResult<MenuItem> {
        let input = sanitize_menu_item(input)?;
        ensure_menu_section_exists(&self.repository, &input.section_id).await?;
        ensure_menu_parent_is_valid(&self.repository, None, &input).await?;
        let item = self.repository.create_menu_item(input).await?;
        self.rebuild_cache().await?;
        Ok(item)
    }

    pub async fn replace_menu_item(&self, id: &str, input: MenuItemInput) -> RbacResult<MenuItem> {
        let input = sanitize_menu_item(input)?;
        ensure_menu_item_exists(&self.repository, id).await?;
        ensure_menu_section_exists(&self.repository, &input.section_id).await?;
        ensure_menu_parent_is_valid(&self.repository, Some(id), &input).await?;
        let item = self.repository.replace_menu_item(id, input).await?;
        self.rebuild_cache().await?;
        Ok(item)
    }

    pub async fn delete_menu_item(&self, id: &str) -> RbacResult<()> {
        ensure_menu_item_exists(&self.repository, id).await?;
        reject_menu_item_delete_with_dependents(&self.repository, id).await?;
        self.repository.delete_menu_item(id).await?;
        self.rebuild_cache().await
    }

    pub async fn page_menu_items(&self, page: PageRequest) -> RbacResult<Page<MenuItem>> {
        validate_page(page)?;
        self.repository.page_menu_items(page).await
    }

    pub async fn replace_role_apis(&self, role_code: &str, api_permission_ids: Vec<String>) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_code).await?;
        ensure_api_permissions_exist(&self.repository, &api_permission_ids).await?;
        self.repository.replace_role_apis(role_code, api_permission_ids).await?;
        self.rebuild_cache().await
    }

    pub async fn replace_role_menus(&self, role_code: &str, input: RoleMenuBindingInput) -> RbacResult<()> {
        ensure_role_exists(&self.repository, role_code).await?;
        ensure_menu_items_exist(&self.repository, &input.menu_item_ids).await?;
        self.repository.replace_role_menus(role_code, input).await?;
        self.rebuild_cache().await
    }

    pub async fn role_api_bindings(&self, role_code: &str) -> RbacResult<RoleApiBindingInput> {
        ensure_role_exists(&self.repository, role_code).await?;
        Ok(RoleApiBindingInput {
            api_permission_ids: self.repository.role_api_ids(role_code).await?,
        })
    }

    pub async fn role_menu_bindings(&self, role_code: &str) -> RbacResult<RoleMenuBindingInput> {
        ensure_role_exists(&self.repository, role_code).await?;
        Ok(RoleMenuBindingInput {
            menu_item_ids: self.repository.role_menu_item_ids(role_code).await?,
        })
    }

    pub async fn navbar(&self, role_code: &str) -> RbacResult<NavResponse> {
        self.cache.read_nav(role_code).await
    }

    pub async fn authorize_api(&self, config: &AuthorizationConfig, request: ApiCheckRequest) -> RbacResult<()> {
        if self.is_whitelisted(config, &request.method, &request.path)? {
            return Ok(());
        }

        if request.system {
            return Ok(());
        }

        let snapshot = self.cache.read_snapshot().await?;
        authorize_snapshot(&snapshot.api_permissions, &request)
    }

    pub fn is_whitelisted(&self, config: &AuthorizationConfig, method: &str, path: &str) -> RbacResult<bool> {
        is_whitelisted(config, method, path)
    }

    pub async fn rebuild_cache(&self) -> RbacResult<()> {
        let snapshot = self.repository.permission_snapshot().await?;
        self.cache.write_snapshot(&snapshot).await
    }
}

async fn reject_system_role_update<R: RbacRepository>(repository: &R, code: &str) -> RbacResult<()> {
    let role = repository.find_role(code).await?.ok_or(RbacError::NotFound)?;
    if role.system {
        return Err(RbacError::Conflict("system role cannot be changed".into()));
    }
    Ok(())
}

async fn reject_bound_role_delete<R: RbacRepository>(repository: &R, code: &str) -> RbacResult<()> {
    if repository.role_has_api_bindings(code).await? || repository.role_has_menu_bindings(code).await? {
        return Err(RbacError::Conflict("role is still bound to API permissions or menu items".into()));
    }
    if repository.role_has_users(code).await? {
        return Err(RbacError::Conflict("role is still assigned to users".into()));
    }
    Ok(())
}

async fn reject_bound_api_delete<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    if repository.api_has_role_bindings(id).await? {
        return Err(RbacError::Conflict("API permission is still bound to roles".into()));
    }
    Ok(())
}

async fn reject_non_empty_menu_section_delete<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    if repository.menu_section_has_items(id).await? {
        return Err(RbacError::Conflict("menu section still contains menu items".into()));
    }
    Ok(())
}

async fn reject_menu_item_delete_with_dependents<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    if repository.menu_item_has_children(id).await? {
        return Err(RbacError::Conflict("menu item still has child menu items".into()));
    }
    if repository.menu_item_has_role_bindings(id).await? {
        return Err(RbacError::Conflict("menu item is still bound to roles".into()));
    }
    Ok(())
}

async fn ensure_role_exists<R: RbacRepository>(repository: &R, code: &str) -> RbacResult<()> {
    repository.find_role(code).await?.map(|_| ()).ok_or(RbacError::NotFound)
}

async fn ensure_api_permission_exists<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    repository.find_api(id).await?.map(|_| ()).ok_or(RbacError::NotFound)
}

async fn ensure_api_permissions_exist<R: RbacRepository>(repository: &R, ids: &[String]) -> RbacResult<()> {
    for id in unique_ids(ids) {
        if repository.find_api(id).await?.is_none() {
            return Err(RbacError::InvalidInput(format!("api permission does not exist: {id}")));
        }
    }
    Ok(())
}

async fn ensure_menu_items_exist<R: RbacRepository>(repository: &R, ids: &[String]) -> RbacResult<()> {
    for id in unique_ids(ids) {
        if repository.find_menu_item(id).await?.is_none() {
            return Err(RbacError::InvalidInput(format!("menu item does not exist: {id}")));
        }
    }
    Ok(())
}

async fn ensure_menu_section_exists<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    repository
        .find_menu_section(id)
        .await?
        .map(|_| ())
        .ok_or_else(|| RbacError::InvalidInput(format!("menu section does not exist: {id}")))
}

async fn ensure_menu_item_exists<R: RbacRepository>(repository: &R, id: &str) -> RbacResult<()> {
    repository.find_menu_item(id).await?.map(|_| ()).ok_or(RbacError::NotFound)
}

async fn ensure_menu_parent_is_valid<R: RbacRepository>(repository: &R, current_id: Option<&str>, input: &MenuItemInput) -> RbacResult<()> {
    let Some(parent_id) = input.parent_id.as_deref() else {
        return Ok(());
    };

    if current_id == Some(parent_id) {
        return Err(RbacError::InvalidInput("menu item cannot be its own parent".into()));
    }

    let parent = repository
        .find_menu_item(parent_id)
        .await?
        .ok_or_else(|| RbacError::InvalidInput(format!("parent menu item does not exist: {parent_id}")))?;
    if parent.section_id != input.section_id {
        return Err(RbacError::InvalidInput("parent menu item must belong to the same section".into()));
    }

    if let Some(current_id) = current_id {
        ensure_menu_parent_does_not_create_cycle(repository, current_id, parent_id).await?;
    }

    Ok(())
}

async fn ensure_menu_parent_does_not_create_cycle<R: RbacRepository>(repository: &R, current_id: &str, parent_id: &str) -> RbacResult<()> {
    let items = repository.list_menu_items().await?;
    let mut cursor = Some(parent_id);
    while let Some(id) = cursor {
        if id == current_id {
            return Err(RbacError::InvalidInput("menu parent cannot be a descendant of itself".into()));
        }
        cursor = items.iter().find(|item| item.id == id).and_then(|item| item.parent_id.as_deref());
    }
    Ok(())
}

fn unique_ids(ids: &[String]) -> Vec<&str> {
    let mut ids = ids.iter().map(String::as_str).collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    ids
}

#[cfg(test)]
mod test_fixtures;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
