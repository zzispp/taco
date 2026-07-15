pub(super) mod data;
pub(super) mod structure;

use audit_contract::{
    BusinessType, EndpointAccess, EndpointAudit, EndpointManifest, EndpointPermission, EndpointPermissionRequirement, EndpointSpec, OperationEndpointAudit,
    RequestCapture,
};

const fn permission(handler: &'static str, requirement: EndpointPermissionRequirement) -> EndpointAccess {
    EndpointAccess::Permission(EndpointPermission { handler, requirement })
}

const fn scoped_permission(handler: &'static str, requirement: EndpointPermissionRequirement) -> EndpointAccess {
    EndpointAccess::DataScopedPermission(EndpointPermission { handler, requirement })
}

const fn read(method: audit_contract::EndpointMethod, path: &'static str, access: EndpointAccess) -> EndpointSpec {
    EndpointSpec {
        method,
        path,
        access,
        audit: EndpointAudit::read_only_for(method),
    }
}

const fn operation(title_key: &'static str, business_type: BusinessType, handler: &'static str) -> EndpointAudit {
    EndpointAudit::Operation(OperationEndpointAudit {
        title_key,
        business_type,
        handler,
        request_capture: RequestCapture::Sanitized,
    })
}

const fn operation_without_request(title_key: &'static str, business_type: BusinessType, handler: &'static str) -> EndpointAudit {
    EndpointAudit::Operation(OperationEndpointAudit {
        title_key,
        business_type,
        handler,
        request_capture: RequestCapture::None,
    })
}

const SEGMENTS: &[&[EndpointSpec]] = &[structure::ENDPOINTS, data::ENDPOINTS];

pub fn endpoint_specs() -> EndpointManifest {
    EndpointManifest::new(SEGMENTS)
}

#[cfg(test)]
mod tests {
    use audit_contract::{EndpointAccess, EndpointAudit, RequestCapture};

    use super::endpoint_specs;

    #[test]
    fn endpoint_specs_cover_system_core_routes_and_management_actions() {
        let specs = endpoint_specs();
        let entries = specs.iter().collect::<Vec<_>>();

        specs.validate().unwrap();
        assert_eq!(entries.len(), 46);
        assert_eq!(entries.iter().filter(|spec| matches!(spec.audit, EndpointAudit::Operation(_))).count(), 27);
        assert_eq!(
            entries
                .iter()
                .filter(|spec| matches!(spec.access, EndpointAccess::DataScopedPermission(_)))
                .count(),
            7
        );
    }

    #[test]
    fn config_management_operations_do_not_capture_request_payloads() {
        let config_operations = endpoint_specs()
            .iter()
            .filter(|spec| spec.path.starts_with("/api/system/configs"))
            .filter_map(|spec| match spec.audit {
                EndpointAudit::Operation(operation) => Some(operation),
                EndpointAudit::ReadOnly | EndpointAudit::ExplicitReadOnly | EndpointAudit::Security => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(config_operations.len(), 6);
        assert!(config_operations.iter().all(|operation| operation.request_capture == RequestCapture::None));
    }
}
