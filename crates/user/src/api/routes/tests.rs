mod admin;
mod auth;
mod auth_audit;
mod avatar;
mod captcha;
mod locale;
mod logout;
mod online;
mod online_force_logout;
mod scope;
mod support;
mod user_filters;
mod zero_roles;

use audit_contract::{EndpointAccess, EndpointAudit, EndpointMethod, validate_endpoint_specs};

#[test]
fn user_endpoint_specs_classify_every_route_and_keep_template_generation_read_only() {
    let specs = crate::api::endpoint_specs();

    assert_eq!(validate_endpoint_specs(specs), Ok(()));
    assert_eq!(specs.len(), 27);
    assert_eq!(specs.iter().filter(|spec| matches!(spec.audit, EndpointAudit::Operation(_))).count(), 13);
    assert_eq!(
        specs
            .iter()
            .filter(|spec| matches!(spec.access, EndpointAccess::DataScopedPermission(_)))
            .count(),
        12
    );
    assert!(specs.iter().any(|spec| {
        spec.method == EndpointMethod::Post && spec.path == "/api/system/users/import-template" && spec.audit == EndpointAudit::ExplicitReadOnly
    }));
    assert!(specs.iter().any(|spec| {
        spec.method == EndpointMethod::Post && spec.path == "/api/account/profile/avatar" && matches!(spec.audit, EndpointAudit::Operation(_))
    }));
    assert!(
        specs
            .iter()
            .any(|spec| { spec.method == EndpointMethod::Get && spec.path == "/api/avatars/{user_id}/{version}" && spec.access == EndpointAccess::Public })
    );
}
