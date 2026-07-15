use super::{
    EndpointAccess, EndpointAudit, EndpointManifest, EndpointMethod, EndpointPermission, EndpointPermissionRequirement, EndpointSpec, EndpointSpecError,
    OperationEndpointAudit, RequestCapture, validate_endpoint_specs,
};
use crate::BusinessType;

const VALID: EndpointSpec = EndpointSpec {
    method: EndpointMethod::Post,
    path: "/api/system/example",
    access: EndpointAccess::Permission(EndpointPermission {
        handler: "create_example",
        requirement: EndpointPermissionRequirement::all_of(&["system:example:add"]),
    }),
    audit: EndpointAudit::Operation(OperationEndpointAudit {
        title_key: "audit.module.example",
        business_type: BusinessType::Insert,
        handler: "system::create_example",
        request_capture: RequestCapture::Sanitized,
    }),
};

#[test]
fn endpoint_specs_expose_the_nested_axum_path() {
    assert_eq!(VALID.api_route_path(), "/system/example");
    assert_eq!(validate_endpoint_specs(&[VALID]), Ok(()));
}

#[test]
fn endpoint_specs_distinguish_data_scoped_permissions() {
    let scoped = EndpointSpec {
        access: EndpointAccess::DataScopedPermission(EndpointPermission {
            handler: "list_scoped_examples",
            requirement: EndpointPermissionRequirement::all_of(&["system:example:list"]),
        }),
        ..VALID
    };

    assert_eq!(validate_endpoint_specs(&[scoped]), Ok(()));
    assert!(matches!(scoped.access, EndpointAccess::DataScopedPermission(_)));
}

#[test]
fn endpoint_specs_reject_duplicate_method_and_path() {
    assert_eq!(
        validate_endpoint_specs(&[VALID, VALID]),
        Err(EndpointSpecError::Duplicate {
            method: "POST",
            path: "/api/system/example",
        })
    );
}

#[test]
fn endpoint_specs_require_non_get_read_only_routes_to_be_explicit() {
    let accidental = EndpointSpec {
        method: EndpointMethod::Post,
        audit: EndpointAudit::ReadOnly,
        ..VALID
    };
    let intentional = EndpointSpec {
        method: EndpointMethod::Post,
        audit: EndpointAudit::ExplicitReadOnly,
        ..VALID
    };

    assert_eq!(
        validate_endpoint_specs(&[accidental]),
        Err(EndpointSpecError::NonGetReadOnly {
            method: "POST",
            path: "/api/system/example",
        })
    );
    assert_eq!(validate_endpoint_specs(&[intentional]), Ok(()));
}

#[test]
fn endpoint_specs_reject_operation_audit_for_get() {
    let invalid = EndpointSpec {
        method: EndpointMethod::Get,
        ..VALID
    };

    assert_eq!(
        validate_endpoint_specs(&[invalid]),
        Err(EndpointSpecError::OperationOnRead { path: "/api/system/example" })
    );
}

#[test]
fn endpoint_manifests_validate_across_static_segments() {
    const FIRST: &[EndpointSpec] = &[VALID];
    const SECOND: &[EndpointSpec] = &[EndpointSpec {
        method: EndpointMethod::Get,
        path: "/api/system/example/summary",
        access: EndpointAccess::Public,
        audit: EndpointAudit::ReadOnly,
    }];
    const SEGMENTS: &[&[EndpointSpec]] = &[FIRST, SECOND];

    assert_eq!(EndpointManifest::new(SEGMENTS).validate(), Ok(()));
}

#[test]
fn operation_endpoint_audit_declares_request_capture_policy() {
    let EndpointAudit::Operation(operation) = VALID.audit else {
        panic!("valid endpoint must declare an operation audit");
    };

    assert_eq!(operation.request_capture, RequestCapture::Sanitized);
}
