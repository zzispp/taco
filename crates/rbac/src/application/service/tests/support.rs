use super::*;
use crate::domain::{DataScope, PermissionSnapshot};

#[derive(Clone, Default)]
pub(super) struct MemoryRepository {
    users: Vec<TestUser>,
    role_ids: Vec<String>,
    installation_owner_user_ids: Vec<String>,
    installation_owner_role_ids: Vec<String>,
}

#[derive(Clone)]
struct TestUser {
    user_id: String,
    dept_id: Option<String>,
}

#[derive(Clone)]
pub(super) struct MemoryCache {
    snapshot: PermissionSnapshot,
}

pub(super) fn test_service(snapshot: PermissionSnapshot) -> RbacService<MemoryRepository, MemoryCache> {
    RbacService::new(MemoryRepository::default(), MemoryCache { snapshot })
}

pub(super) fn test_admin_service(repository: MemoryRepository) -> RbacService<MemoryRepository, MemoryCache> {
    RbacService::new(repository, MemoryCache { snapshot: snapshot(vec![]) })
}

impl MemoryRepository {
    pub(super) fn with_user(mut self, user_id: &str, dept_id: &str) -> Self {
        self.users.push(TestUser {
            user_id: user_id.into(),
            dept_id: Some(dept_id.into()),
        });
        self
    }

    pub(super) fn with_installation_owner_user(mut self, user_id: &str) -> Self {
        self.installation_owner_user_ids.push(user_id.into());
        self
    }

    pub(super) fn with_installation_owner_role(mut self, role_id: &str) -> Self {
        self.installation_owner_role_ids.push(role_id.into());
        self
    }

    pub(super) fn with_role(mut self, role_id: &str) -> Self {
        self.role_ids.push(role_id.into());
        self
    }
}

pub(super) fn config() -> AuthorizationConfig {
    config_with_requirement(crate::application::PermissionRequirement::all_of(&["system:user:list"]))
}

pub(super) fn config_with_requirement(requirement: crate::application::PermissionRequirement) -> AuthorizationConfig {
    AuthorizationConfig::compile(
        vec![],
        vec![crate::application::RoutePermissionRule {
            methods: vec!["GET".into()],
            path_pattern: "/api/system/users".into(),
            requirement,
            handler: "list_users",
        }],
    )
    .unwrap()
}

pub(super) fn auth_me_config() -> AuthorizationConfig {
    AuthorizationConfig::compile(
        vec![AuthWhitelistRule {
            methods: vec!["GET".into()],
            path_pattern: "/api/auth/me".into(),
        }],
        vec![],
    )
    .unwrap()
}

pub(super) fn request(permissions: Vec<&str>, is_installation_owner: bool) -> ApiCheckRequest {
    ApiCheckRequest {
        method: "GET".into(),
        path: "/api/system/users".into(),
        role_keys: vec!["business-role".into()],
        permissions: permissions.into_iter().map(String::from).collect(),
        is_installation_owner,
    }
}

pub(super) fn auth_me_request() -> ApiCheckRequest {
    ApiCheckRequest {
        method: "GET".into(),
        path: "/api/auth/me".into(),
        role_keys: vec!["business-role".into()],
        permissions: vec![],
        is_installation_owner: false,
    }
}

pub(super) fn current_user(role_keys: Vec<&str>, is_installation_owner: bool) -> CurrentUser {
    CurrentUser {
        id: "2".into(),
        username: "taco".into(),
        role_keys: role_keys.into_iter().map(String::from).collect(),
        permissions: vec![],
        dept_id: Some("103".into()),
        is_installation_owner,
    }
}

pub(super) fn snapshot(roles: Vec<RolePermissionSnapshot>) -> PermissionSnapshot {
    PermissionSnapshot {
        roles,
        menus: vec![],
        installation_owner_menus: vec![],
    }
}

pub(super) fn role_scope(role_key: &str, data_scope: &str, dept_ids: Vec<&str>) -> RolePermissionSnapshot {
    RolePermissionSnapshot {
        role_key: role_key.into(),
        status: "0".into(),
        permissions: vec![],
        data_scope: data_scope.into(),
        dept_ids: dept_ids.into_iter().map(String::from).collect(),
    }
}

pub(super) fn disabled_role_scope(role_key: &str, data_scope: &str) -> RolePermissionSnapshot {
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

    async fn read_nav(&self, _role_keys: &[String], _is_installation_owner: bool) -> RbacResult<NavResponse> {
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

    async fn find_role(&self, role_id: &str) -> RbacResult<Option<Role>> {
        Ok(self.role_ids.iter().any(|id| id == role_id).then(|| role(role_id)))
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

    async fn page_roles(&self, _filter: RoleListFilter) -> RbacResult<CursorPage<Role>> {
        Ok(empty_page())
    }

    async fn page_roles_scoped(&self, _filter: RoleListFilter, _scope: DataScopeFilter) -> RbacResult<CursorPage<Role>> {
        Ok(empty_page())
    }

    async fn export_roles(&self, _request: RoleExportRequest, _sink: &mut dyn RoleExportSink) -> RbacResult<()> {
        Ok(())
    }

    async fn role_options(&self) -> RbacResult<Vec<RoleOption>> {
        Ok(vec![])
    }

    async fn page_role_users(&self, _filter: RoleUserListFilter, _scope: Option<DataScopeFilter>) -> RbacResult<CursorPage<RoleUser>> {
        Ok(empty_page())
    }

    async fn scoped_user_ids(&self, user_ids: &[String], scope: DataScopeFilter) -> RbacResult<Vec<String>> {
        Ok(self
            .users
            .iter()
            .filter(|user| user_ids.contains(&user.user_id) && test_user_scope_matches(user, &scope))
            .map(|user| user.user_id.clone())
            .collect())
    }

    async fn has_installation_owner(&self, user_ids: &[String]) -> RbacResult<bool> {
        Ok(user_ids.iter().any(|user_id| self.installation_owner_user_ids.contains(user_id)))
    }

    async fn role_has_installation_owner(&self, role_id: &str) -> RbacResult<bool> {
        Ok(self.installation_owner_role_ids.iter().any(|id| id == role_id))
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

    async fn page_menus(&self, _filter: MenuListFilter) -> RbacResult<CursorPage<Menu>> {
        Ok(empty_page())
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

fn empty_page<T>() -> CursorPage<T> {
    CursorPage::new(vec![], None, None)
}

fn role(role_id: &str) -> Role {
    Role {
        role_id: role_id.into(),
        role_name: "business role".into(),
        role_key: "business-role".into(),
        role_sort: 1,
        data_scope: "1".into(),
        menu_check_strictly: true,
        dept_check_strictly: true,
        status: "0".into(),
        system: false,
        remark: None,
        create_time: String::new(),
    }
}

fn test_user_scope_matches(user: &TestUser, scope: &DataScopeFilter) -> bool {
    match scope.data_scope {
        DataScope::All => true,
        DataScope::SelfOnly => user.user_id == scope.user_id,
        _ => user.dept_id == scope.dept_id,
    }
}
