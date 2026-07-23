use super::*;
use crate::domain::PermissionSnapshot;

mod repository;

pub(super) use repository::MemoryRepository;

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

pub(super) fn request(permissions: Vec<&str>) -> ApiCheckRequest {
    ApiCheckRequest {
        method: "GET".into(),
        path: "/api/system/users".into(),
        role_keys: vec!["business-role".into()],
        permissions: permissions.into_iter().map(String::from).collect(),
    }
}

pub(super) fn auth_me_request() -> ApiCheckRequest {
    ApiCheckRequest {
        method: "GET".into(),
        path: "/api/auth/me".into(),
        role_keys: vec!["business-role".into()],
        permissions: vec![],
    }
}

pub(super) fn current_user(role_keys: Vec<&str>) -> CurrentUser {
    CurrentUser {
        id: "2".into(),
        username: "taco".into(),
        role_keys: role_keys.into_iter().map(String::from).collect(),
        permissions: vec![],
        dept_id: Some("103".into()),
    }
}

pub(super) fn snapshot(roles: Vec<RolePermissionSnapshot>) -> PermissionSnapshot {
    PermissionSnapshot { roles, menus: vec![] }
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

    async fn read_nav(&self, _role_keys: &[String]) -> RbacResult<NavResponse> {
        Ok(NavResponse { nav_items: vec![] })
    }
}
