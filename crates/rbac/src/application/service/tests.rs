use async_trait::async_trait;
use kernel::pagination::{Page, PageRequest};

use super::*;
use crate::domain::RolePermissionSnapshot;

#[derive(Clone, Default)]
struct MemoryRepository;

#[derive(Clone)]
struct MemoryCache {
    snapshot: PermissionSnapshot,
}

#[tokio::test]
async fn authorize_api_allows_declared_permission() {
    let service = test_service(snapshot(vec![]));
    let result = service.authorize_api(&config(), request(vec!["system:user:list"], false)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn authorize_api_rejects_missing_permission() {
    let service = test_service(snapshot(vec![]));
    let result = service.authorize_api(&config(), request(vec!["system:role:list"], false)).await;
    assert!(matches!(result, Err(RbacError::Forbidden)));
}

#[tokio::test]
async fn authorize_api_allows_taco_wildcard_permission() {
    let service = test_service(snapshot(vec![]));
    let result = service.authorize_api(&config(), request(vec![constants::system::ALL_PERMISSION], false)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn authorize_api_allows_admin_without_permission() {
    let service = test_service(snapshot(vec![]));
    let result = service.authorize_api(&config(), request(vec![], true)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn data_scope_uses_admin_all_scope() {
    let service = test_service(snapshot(vec![role_scope("common", "5", vec!["103"])]));
    let filter = service.data_scope_filter(&current_user(vec!["common"], true)).await.unwrap();
    assert_eq!(filter.data_scope, "1");
}

#[tokio::test]
async fn data_scope_uses_most_permissive_role_scope() {
    let service = test_service(snapshot(vec![role_scope("a", "5", vec![]), role_scope("b", "3", vec!["103"])]));
    let filter = service.data_scope_filter(&current_user(vec!["a", "b"], false)).await.unwrap();
    assert_eq!(filter.data_scope, "3");
    assert_eq!(filter.dept_id, Some("103".into()));
    assert_eq!(filter.dept_ids, vec!["103"]);
}

#[tokio::test]
async fn data_scope_ignores_disabled_roles() {
    let service = test_service(snapshot(vec![disabled_role_scope("wide", "1"), role_scope("narrow", "5", vec![])]));
    let filter = service.data_scope_filter(&current_user(vec!["wide", "narrow"], false)).await.unwrap();

    assert_eq!(filter.data_scope, "5");
}

#[test]
fn validate_data_scope_handlers_rejects_missing_macro_registration() {
    let service = test_service(snapshot(vec![]));
    let result = service.validate_data_scope_handlers(&["definitely_missing_data_scope_handler"]);

    assert!(matches!(result, Err(RbacError::InvalidInput(_))));
}

fn test_service(snapshot: PermissionSnapshot) -> RbacService<MemoryRepository, MemoryCache> {
    RbacService::new(MemoryRepository, MemoryCache { snapshot })
}

fn config() -> AuthorizationConfig {
    AuthorizationConfig {
        whitelist: vec![],
        route_permissions: vec![types::rbac::RoutePermissionRule {
            methods: vec!["GET".into()],
            path_pattern: "/api/system/users".into(),
            permission: "system:user:list".into(),
            handler: "list_users",
        }],
    }
}

fn request(permissions: Vec<&str>, admin: bool) -> ApiCheckRequest {
    ApiCheckRequest {
        method: "GET".into(),
        path: "/api/system/users".into(),
        role_keys: vec!["common".into()],
        permissions: permissions.into_iter().map(String::from).collect(),
        admin,
    }
}

fn current_user(role_keys: Vec<&str>, admin: bool) -> CurrentUser {
    CurrentUser {
        id: "2".into(),
        username: "taco".into(),
        role_keys: role_keys.into_iter().map(String::from).collect(),
        permissions: vec![],
        dept_id: Some("103".into()),
        admin,
        system: admin,
    }
}

fn snapshot(roles: Vec<RolePermissionSnapshot>) -> PermissionSnapshot {
    PermissionSnapshot { roles, menus: vec![] }
}

fn role_scope(role_key: &str, data_scope: &str, dept_ids: Vec<&str>) -> RolePermissionSnapshot {
    RolePermissionSnapshot {
        role_key: role_key.into(),
        status: "0".into(),
        permissions: vec![],
        data_scope: data_scope.into(),
        dept_ids: dept_ids.into_iter().map(String::from).collect(),
    }
}

fn disabled_role_scope(role_key: &str, data_scope: &str) -> RolePermissionSnapshot {
    RolePermissionSnapshot {
        role_key: role_key.into(),
        status: "1".into(),
        permissions: vec![],
        data_scope: data_scope.into(),
        dept_ids: vec![],
    }
}

#[async_trait]
impl RbacCache for MemoryCache {
    async fn write_snapshot(&self, _snapshot: &PermissionSnapshot) -> RbacResult<()> {
        Ok(())
    }

    async fn read_snapshot(&self) -> RbacResult<PermissionSnapshot> {
        Ok(self.snapshot.clone())
    }

    async fn read_nav(&self, _role_keys: &[String], _admin: bool) -> RbacResult<NavResponse> {
        Ok(NavResponse { nav_items: vec![] })
    }
}

#[async_trait]
impl RbacRepository for MemoryRepository {
    async fn create_role(&self, _input: RoleInput) -> RbacResult<Role> {
        Err(RbacError::NotFound)
    }

    async fn replace_role(&self, _role_id: &str, _input: RoleInput) -> RbacResult<Role> {
        Err(RbacError::NotFound)
    }

    async fn update_role_status(&self, _role_id: &str, _status: String) -> RbacResult<Role> {
        Err(RbacError::NotFound)
    }

    async fn update_role_data_scope(&self, _role_id: &str, _input: RoleDataScopeInput) -> RbacResult<Role> {
        Err(RbacError::NotFound)
    }

    async fn delete_role(&self, _role_id: &str) -> RbacResult<()> {
        Ok(())
    }

    async fn delete_roles(&self, _role_ids: &[String]) -> RbacResult<()> {
        Ok(())
    }

    async fn find_role(&self, _role_id: &str) -> RbacResult<Option<Role>> {
        Ok(None)
    }

    async fn role_name_exists(&self, _name: &str, _current_id: Option<&str>) -> RbacResult<bool> {
        Ok(false)
    }

    async fn role_key_exists(&self, _key: &str, _current_id: Option<&str>) -> RbacResult<bool> {
        Ok(false)
    }

    async fn role_has_users(&self, _role_id: &str) -> RbacResult<bool> {
        Ok(false)
    }

    async fn page_roles(&self, filter: RoleListFilter) -> RbacResult<Page<Role>> {
        Ok(empty_page(filter.page))
    }

    async fn page_roles_scoped(&self, filter: RoleListFilter, _scope: DataScopeFilter) -> RbacResult<Page<Role>> {
        Ok(empty_page(filter.page))
    }

    async fn role_options(&self) -> RbacResult<Vec<RoleOption>> {
        Ok(vec![])
    }

    async fn page_role_users(&self, filter: RoleUserListFilter, _scope: Option<DataScopeFilter>) -> RbacResult<Page<RoleUser>> {
        Ok(empty_page(filter.page))
    }

    async fn replace_role_users(&self, _role_id: &str, _input: RoleUserBindingInput) -> RbacResult<()> {
        Ok(())
    }

    async fn delete_role_user(&self, _role_id: &str, _user_id: &str) -> RbacResult<()> {
        Ok(())
    }

    async fn delete_role_users(&self, _role_id: &str, _user_ids: &[String]) -> RbacResult<()> {
        Ok(())
    }

    async fn create_menu(&self, _input: MenuInput) -> RbacResult<Menu> {
        Err(RbacError::NotFound)
    }

    async fn replace_menu(&self, _menu_id: &str, _input: MenuInput) -> RbacResult<Menu> {
        Err(RbacError::NotFound)
    }

    async fn update_menu_sort(&self, _menu_id: &str, _order_num: i64) -> RbacResult<Menu> {
        Err(RbacError::NotFound)
    }

    async fn update_menu_sorts(&self, _input: types::system::SortBatchInput) -> RbacResult<Vec<Menu>> {
        Ok(vec![])
    }

    async fn delete_menu(&self, _menu_id: &str) -> RbacResult<()> {
        Ok(())
    }

    async fn find_menu(&self, _menu_id: &str) -> RbacResult<Option<Menu>> {
        Ok(None)
    }

    async fn menu_has_children(&self, _menu_id: &str) -> RbacResult<bool> {
        Ok(false)
    }

    async fn menu_has_role_bindings(&self, _menu_id: &str) -> RbacResult<bool> {
        Ok(false)
    }

    async fn list_menus(&self) -> RbacResult<Vec<Menu>> {
        Ok(vec![])
    }

    async fn page_menus(&self, filter: MenuListFilter) -> RbacResult<Page<Menu>> {
        Ok(empty_page(filter.page))
    }

    async fn replace_role_menus(&self, _role_id: &str, _input: RoleMenuBindingInput) -> RbacResult<()> {
        Ok(())
    }

    async fn replace_role_depts(&self, _role_id: &str, _input: RoleDeptBindingInput) -> RbacResult<()> {
        Ok(())
    }

    async fn role_menu_ids(&self, _role_id: &str) -> RbacResult<Vec<String>> {
        Ok(vec![])
    }

    async fn role_dept_ids(&self, _role_id: &str) -> RbacResult<Vec<String>> {
        Ok(vec![])
    }

    async fn permission_snapshot(&self) -> RbacResult<PermissionSnapshot> {
        Ok(snapshot(vec![]))
    }
}

fn empty_page<T>(page: PageRequest) -> Page<T> {
    Page {
        items: vec![],
        total: 0,
        page: page.page,
        page_size: page.page_size,
    }
}
