use async_trait::async_trait;
use kernel::pagination::CursorPage;

use super::*;
use crate::{
    application::{AuthWhitelistRule, PermissionRequirement},
    domain::{DataScope, MenuInput, RolePermissionSnapshot},
};

mod support;

use support::{
    MemoryRepository, auth_me_config, auth_me_request, config, config_with_requirement, current_user, disabled_role_scope, request, role_scope, snapshot,
    test_admin_service, test_service,
};

#[tokio::test]
async fn authorize_api_allows_declared_permission() {
    let service = test_service(snapshot(vec![]));
    let result = service.authorize_api(&config(), request(vec!["system:user:list"])).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn authorize_api_rejects_missing_permission() {
    let service = test_service(snapshot(vec![]));
    let result = service.authorize_api(&config(), request(vec!["system:role:list"])).await;
    assert!(matches!(result, Err(RbacError::Forbidden)));
}

#[tokio::test]
async fn authorize_api_accepts_any_declared_permission() {
    let service = test_service(snapshot(vec![]));
    let authorization = config_with_requirement(PermissionRequirement::any_of(&["system:user:import", "system:user:edit"]));

    let imported = service.authorize_api(&authorization, request(vec!["system:user:import"])).await;
    let edited = service.authorize_api(&authorization, request(vec!["system:user:edit"])).await;
    let rejected = service.authorize_api(&authorization, request(vec!["system:user:list"])).await;

    assert!(imported.is_ok());
    assert!(edited.is_ok());
    assert!(matches!(rejected, Err(RbacError::Forbidden)));
}

#[tokio::test]
async fn authorize_api_rejects_reserved_wildcard_permission_for_business_roles() {
    let service = test_service(snapshot(vec![]));
    let result = service
        .authorize_api(&config(), request(vec![constants::system::RESERVED_WILDCARD_PERMISSION]))
        .await;
    assert!(matches!(result, Err(RbacError::Forbidden)));
}

#[tokio::test]
async fn menu_creation_rejects_the_reserved_wildcard_permission() {
    let service = test_admin_service(MemoryRepository::default());
    let result = service
        .create_menu(MenuInput {
            menu_name: "all permissions".into(),
            parent_id: "0".into(),
            order_num: 1,
            path: "permissions".into(),
            component: None,
            query: None,
            route_name: "all-permissions".into(),
            is_frame: false,
            is_cache: false,
            menu_type: "F".into(),
            visible: "0".into(),
            status: "0".into(),
            perms: Some(format!("  {}  ", constants::system::RESERVED_WILDCARD_PERMISSION)),
            icon: "shield".into(),
            remark: None,
        })
        .await;

    assert!(matches!(result, Err(RbacError::InvalidInput(error)) if error.key() == "errors.rbac.wildcard_permission_reserved"));
}

#[tokio::test]
async fn authorize_api_allows_whitelisted_me_without_permission() {
    let service = test_service(snapshot(vec![]));
    let result = service.authorize_api(&auth_me_config(), auth_me_request()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn data_scope_uses_the_assigned_role_binding() {
    let service = test_service(snapshot(vec![role_scope("business-admin", "5", vec!["103"])]));
    let filter = service.data_scope_filter(&current_user(vec!["business-admin"])).await.unwrap();
    assert_eq!(filter.data_scope, DataScope::SelfOnly);
}

#[tokio::test]
async fn data_scope_uses_most_permissive_role_scope() {
    let service = test_service(snapshot(vec![role_scope("a", "5", vec![]), role_scope("b", "3", vec!["103"])]));
    let filter = service.data_scope_filter(&current_user(vec!["a", "b"])).await.unwrap();
    assert_eq!(filter.data_scope, DataScope::Department);
    assert_eq!(filter.dept_id, Some("103".into()));
    assert_eq!(filter.dept_ids, Vec::<String>::new());
}

#[tokio::test]
async fn data_scope_collects_departments_only_from_custom_roles() {
    let service = test_service(snapshot(vec![
        role_scope("custom", "2", vec!["104"]),
        role_scope("department", "3", vec!["stale"]),
    ]));

    let filter = service.data_scope_filter(&current_user(vec!["custom", "department"])).await.unwrap();

    assert_eq!(filter.data_scope, DataScope::Custom);
    assert_eq!(filter.dept_ids, vec!["104"]);
}

#[tokio::test]
async fn data_scope_ignores_disabled_roles() {
    let service = test_service(snapshot(vec![disabled_role_scope("wide", "1"), role_scope("narrow", "5", vec![])]));
    let filter = service.data_scope_filter(&current_user(vec!["wide", "narrow"])).await.unwrap();

    assert_eq!(filter.data_scope, DataScope::SelfOnly);
}

#[tokio::test]
async fn data_scope_rejects_unknown_role_scope() {
    let service = test_service(snapshot(vec![role_scope("invalid", "unknown", vec![])]));

    let result = service.data_scope_filter(&current_user(vec!["invalid"])).await;

    let Err(RbacError::InvalidInput(error)) = result else {
        panic!("unknown role data scope must fail explicitly");
    };
    assert_eq!(error.key(), "errors.rbac.invalid_data_scope");
    assert_eq!(error.params(), []);
}

#[test]
fn authorization_config_rejects_invalid_patterns_without_exposing_provider_error() {
    let result = AuthorizationConfig::compile(
        vec![AuthWhitelistRule {
            methods: vec!["GET".into()],
            path_pattern: "/api/{invalid".into(),
        }],
        vec![],
    );

    let Err(RbacError::InvalidInput(error)) = result else {
        panic!("invalid authorization pattern must fail during compilation");
    };
    assert_eq!(error.key(), "errors.rbac.invalid_route_pattern");
    assert_eq!(error.params(), []);
}

#[tokio::test]
async fn ensure_user_ids_scoped_rejects_out_of_scope_role_user() {
    let service = test_admin_service(MemoryRepository::default().with_user("2", "104"));

    let result = service.ensure_user_ids_scoped(vec!["2".into()], self_scope("1", "103")).await;

    assert!(matches!(result, Err(RbacError::Forbidden)));
}

#[tokio::test]
async fn ensure_user_ids_scoped_allows_visible_role_user() {
    let service = test_admin_service(MemoryRepository::default().with_user("2", "104"));

    let result = service.ensure_user_ids_scoped(vec!["2".into()], self_scope("2", "104")).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn system_role_menu_bindings_cannot_be_changed() {
    let service = test_admin_service(MemoryRepository::default().with_system_role("admin-role"));

    let result = service
        .replace_role_menus(
            "admin-role",
            crate::domain::RoleMenuBindingInput {
                menu_ids: vec!["menu-1".into()],
            },
        )
        .await;

    assert!(matches!(result, Err(RbacError::Conflict(error)) if error.key() == "errors.rbac.system_role_immutable"));
}

#[tokio::test]
async fn system_role_department_bindings_cannot_be_changed() {
    let service = test_admin_service(MemoryRepository::default().with_system_role("admin-role"));

    let result = service
        .replace_role_depts(
            "admin-role",
            crate::domain::RoleDeptBindingInput {
                dept_ids: vec!["dept-1".into()],
            },
        )
        .await;

    assert!(matches!(result, Err(RbacError::Conflict(error)) if error.key() == "errors.rbac.system_role_immutable"));
}

#[tokio::test]
async fn role_user_unbinding_cannot_remove_the_last_enabled_admin() {
    let repository = MemoryRepository::default().with_admin_role_user("admin-role", "admin-user");
    let service = test_admin_service(repository);

    let result = service.delete_role_user("admin-role", "admin-user").await;

    assert!(matches!(result, Err(RbacError::Conflict(error)) if error.key() == "errors.rbac.last_enabled_admin_required"));
}

#[tokio::test]
async fn role_user_unbinding_allows_an_admin_when_another_enabled_admin_remains() {
    let repository = MemoryRepository::default()
        .with_admin_role_user("admin-role", "admin-user")
        .with_admin_role_user("admin-role", "other-admin");
    let service = test_admin_service(repository);

    let result = service.delete_role_user("admin-role", "admin-user").await;

    assert!(result.is_ok());
}

fn self_scope(user_id: &str, dept_id: &str) -> DataScopeFilter {
    DataScopeFilter {
        data_scope: DataScope::SelfOnly,
        user_id: user_id.into(),
        dept_id: Some(dept_id.into()),
        dept_ids: vec![],
    }
}
