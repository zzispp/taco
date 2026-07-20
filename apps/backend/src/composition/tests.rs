use audit_contract::{EndpointAccess, EndpointMethod};
use rbac::application::PermissionRequirement;

use super::{
    access_catalog::EndpointCatalog,
    routes::{auth_whitelist, authorization_config, ensure_auth_whitelist_rule},
};

mod http_pipeline;
mod settings;

pub(crate) use settings::test_settings;

#[test]
fn ensure_auth_whitelist_rule_adds_rule_once() {
    let mut rules = vec![];

    ensure_auth_whitelist_rule(&mut rules, &["GET"], "/api/auth/me");
    ensure_auth_whitelist_rule(&mut rules, &["GET"], "/api/auth/me");

    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].methods, vec!["GET"]);
    assert_eq!(rules[0].path_pattern, "/api/auth/me");
}

#[test]
fn endpoint_catalog_covers_cross_context_and_captcha_public_routes() {
    let catalog = EndpointCatalog::build().unwrap();

    assert!(
        catalog
            .specs()
            .iter()
            .any(|spec| { spec.method == EndpointMethod::Get && spec.path == "/api/captcha/config" && spec.access == EndpointAccess::Public })
    );
    assert!(catalog.specs().iter().any(|spec| spec.path == "/api/system/users"));
    assert!(catalog.specs().iter().any(|spec| spec.path == "/api/system/roles"));
    assert!(catalog.specs().iter().any(|spec| spec.path == "/api/system/notices"));
    assert!(catalog.specs().iter().any(|spec| spec.path == "/api/system/jobs"));
    assert!(catalog.specs().iter().any(|spec| spec.path == "/api/system/login-logs"));
}

#[test]
fn authorization_config_is_derived_from_endpoint_permission_specs() {
    let catalog = EndpointCatalog::build().unwrap();
    let authorization = authorization_config(&catalog).unwrap();

    let detail = authorization
        .route_permissions()
        .iter()
        .find(|rule| rule.handler == "get_job_log_detail")
        .unwrap();
    let notice = authorization
        .route_permissions()
        .iter()
        .find(|rule| rule.handler == "list_notice_readers")
        .unwrap();

    assert_eq!(
        detail.requirement,
        PermissionRequirement::all_of(&["system:job:log:query", "system:job:log:detail"])
    );
    assert_eq!(notice.requirement, PermissionRequirement::all_of(&["system:notice:list"]));
    assert_permission_rules_match_registered_handlers(authorization.route_permissions());
}

#[test]
fn auth_whitelist_contains_fixed_routes_and_public_endpoint_specs() {
    let catalog = EndpointCatalog::build().unwrap();
    let rules = auth_whitelist(&catalog);

    for path in [
        "/health",
        "/ready",
        "/metrics",
        "/uploads/avatars/{*file}",
        installation::api::SETUP_STATUS_PATH,
    ] {
        assert_eq!(rule_methods(&rules, path), Some(vec!["GET".to_owned()]));
    }

    let logout_rule = rules.iter().find(|rule| rule.path_pattern == "/api/auth/logout");
    let me_rule = rules.iter().find(|rule| rule.path_pattern == "/api/auth/me");
    let docs_rule = rules.iter().find(|rule| rule.path_pattern == "/docs");

    assert_eq!(logout_rule.map(|rule| rule.methods.clone()), Some(vec!["POST".to_owned()]));
    assert_eq!(me_rule, None);
    assert_eq!(docs_rule, None);
}

fn rule_methods(rules: &[rbac::application::AuthWhitelistRule], path: &str) -> Option<Vec<String>> {
    rules.iter().find(|rule| rule.path_pattern == path).map(|rule| rule.methods.clone())
}

fn assert_permission_rules_match_registered_handlers(rules: &[rbac::application::RoutePermissionRule]) {
    let handlers = rbac::inventory::iter::<rbac::application::ProtectedHandler>.into_iter().collect::<Vec<_>>();
    for rule in rules {
        assert!(
            handlers
                .iter()
                .any(|handler| handler.function == rule.handler && handler.requirement.is_equivalent_to(rule.requirement)),
            "endpoint permission rule has no matching inventory handler: {}",
            rule.handler
        );
    }
}
