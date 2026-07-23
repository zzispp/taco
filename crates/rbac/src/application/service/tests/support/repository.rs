use super::*;
use crate::domain::{DataScope, PermissionSnapshot};
use kernel::error::LocalizedError;

#[derive(Clone, Default)]
pub(in super::super) struct MemoryRepository {
    users: Vec<TestUser>,
    role_ids: Vec<String>,
    system_role_ids: Vec<String>,
    admin_role_users: Vec<(String, String)>,
}

#[derive(Clone)]
struct TestUser {
    user_id: String,
    dept_id: Option<String>,
}

impl MemoryRepository {
    pub(in super::super) fn with_user(mut self, user_id: &str, dept_id: &str) -> Self {
        self.users.push(TestUser {
            user_id: user_id.into(),
            dept_id: Some(dept_id.into()),
        });
        self
    }

    pub(super) fn with_role(mut self, role_id: &str) -> Self {
        self.role_ids.push(role_id.into());
        self
    }

    pub(in super::super) fn with_system_role(mut self, role_id: &str) -> Self {
        self = self.with_role(role_id);
        self.system_role_ids.push(role_id.into());
        self
    }

    pub(in super::super) fn with_admin_role_user(mut self, role_id: &str, user_id: &str) -> Self {
        self = self.with_system_role(role_id);
        self.admin_role_users.push((role_id.into(), user_id.into()));
        self
    }

    fn removes_last_enabled_admin(&self, role_id: &str, user_ids: &[String]) -> bool {
        let targets_admin = self
            .admin_role_users
            .iter()
            .any(|(admin_role_id, user_id)| admin_role_id == role_id && user_ids.contains(user_id));
        targets_admin
            && self
                .admin_role_users
                .iter()
                .filter(|(admin_role_id, user_id)| admin_role_id != role_id || !user_ids.contains(user_id))
                .count()
                == 0
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
        Ok(self.role_ids.iter().any(|id| id == role_id).then(|| {
            role(
                role_id,
                self.system_role_ids.iter().any(|id| id == role_id),
                self.admin_role_users.iter().any(|(id, _)| id == role_id),
            )
        }))
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

    async fn replace_role_users(&self, _role_id: &str, _input: RoleUserBindingInput) -> RbacResult<()> {
        Ok(())
    }

    async fn delete_role_user(&self, role_id: &str, user_id: &str) -> RbacResult<()> {
        if self.removes_last_enabled_admin(role_id, &[user_id.into()]) {
            return Err(last_enabled_admin_error());
        }
        Ok(())
    }

    async fn delete_role_users(&self, role_id: &str, user_ids: &[String]) -> RbacResult<()> {
        if self.removes_last_enabled_admin(role_id, user_ids) {
            return Err(last_enabled_admin_error());
        }
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
        Ok(PermissionSnapshot { roles: vec![], menus: vec![] })
    }
}

fn empty_page<T>() -> CursorPage<T> {
    CursorPage::new(vec![], None, None)
}

fn last_enabled_admin_error() -> RbacError {
    RbacError::Conflict(LocalizedError::new("errors.rbac.last_enabled_admin_required"))
}

fn role(role_id: &str, system: bool, admin: bool) -> Role {
    Role {
        role_id: role_id.into(),
        role_name: "business role".into(),
        role_key: if admin {
            constants::system::ADMIN_ROLE_KEY.into()
        } else {
            "business-role".into()
        },
        role_sort: 1,
        data_scope: "1".into(),
        menu_check_strictly: true,
        dept_check_strictly: true,
        status: "0".into(),
        system,
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
