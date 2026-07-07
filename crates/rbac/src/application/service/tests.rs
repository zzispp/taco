use async_trait::async_trait;
use kernel::pagination::{Page, PageRequest};

use super::*;
use crate::{application::AuthWhitelistRule, domain::RolePermissionSnapshot};

mod support;

use support::{auth_me_config, auth_me_request, config, current_user, disabled_role_scope, request, role_scope, snapshot, test_service};

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
async fn authorize_api_allows_whitelisted_me_without_permission() {
    let service = test_service(snapshot(vec![]));
    let result = service.authorize_api(&auth_me_config(), auth_me_request()).await;
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
