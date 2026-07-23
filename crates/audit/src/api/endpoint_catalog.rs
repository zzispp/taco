use std::collections::HashMap;

use audit_contract::{EndpointAudit, EndpointSpec, OperationEndpointAudit, validate_endpoint_specs};
use matchit::Router;

use crate::application::{AuditError, AuditResult};

/// Resolves operation-audit metadata directly from the endpoint contract.
///
/// The catalog is deliberately built from the same declarations that own route
/// access control.  It does not contain a second, hand-maintained endpoint
/// list.
pub(super) struct EndpointAuditCatalog {
    routers: HashMap<&'static str, Router<OperationEndpointAudit>>,
}

impl EndpointAuditCatalog {
    pub(super) fn new(specs: Vec<EndpointSpec>) -> AuditResult<Self> {
        validate_endpoint_specs(&specs).map_err(endpoint_error)?;
        let mut routers = HashMap::new();
        for spec in specs {
            let operation = match spec.audit {
                EndpointAudit::Operation(operation) | EndpointAudit::Download(operation) => operation,
                EndpointAudit::ReadOnly | EndpointAudit::ExplicitReadOnly | EndpointAudit::Security => continue,
            };
            routers
                .entry(spec.method.as_str())
                .or_insert_with(Router::new)
                .insert(spec.path, operation)
                .map_err(|error| AuditError::Infrastructure(format!("invalid operation endpoint pattern: {error}")))?;
        }
        Ok(Self { routers })
    }

    pub(super) fn find(&self, method: &str, path: &str) -> Option<OperationEndpointAudit> {
        self.routers.get(method)?.at(path).ok().map(|matched| *matched.value)
    }
}

fn endpoint_error(error: audit_contract::EndpointSpecError) -> AuditError {
    AuditError::Infrastructure(format!("invalid audit endpoint contract: {error}"))
}

#[cfg(test)]
mod tests {
    use audit_contract::{BusinessType, EndpointAccess, EndpointAudit, EndpointMethod, EndpointSpec, OperationEndpointAudit, RequestCapture};

    use super::EndpointAuditCatalog;

    #[test]
    fn catalog_resolves_dynamic_operation_endpoint_from_contract() {
        let catalog = EndpointAuditCatalog::new(vec![EndpointSpec {
            method: EndpointMethod::Put,
            path: "/api/system/examples/{id}",
            access: EndpointAccess::Authenticated,
            audit: EndpointAudit::Operation(OperationEndpointAudit {
                title_key: "audit.module.example",
                business_type: BusinessType::Update,
                handler: "example::replace",
                request_capture: RequestCapture::Sanitized,
            }),
        }])
        .unwrap();

        assert_eq!(
            catalog.find("PUT", "/api/system/examples/example-1").map(|entry| entry.handler),
            Some("example::replace")
        );
        assert_eq!(catalog.find("GET", "/api/system/examples/example-1"), None);
    }

    #[test]
    fn catalog_resolves_explicit_download_endpoint_from_contract() {
        let catalog = EndpointAuditCatalog::new(vec![EndpointSpec {
            method: EndpointMethod::Get,
            path: "/api/system/examples/{id}/download",
            access: EndpointAccess::Authenticated,
            audit: EndpointAudit::Download(OperationEndpointAudit {
                title_key: "audit.module.example",
                business_type: BusinessType::Export,
                handler: "example::download",
                request_capture: RequestCapture::None,
            }),
        }])
        .unwrap();

        assert_eq!(
            catalog.find("GET", "/api/system/examples/example-1/download").map(|entry| entry.handler),
            Some("example::download")
        );
    }
}
