use audit_contract::{EndpointAccess, EndpointAudit, EndpointManifest, EndpointMethod, EndpointSpec};

const fn read(method: EndpointMethod, path: &'static str) -> EndpointSpec {
    EndpointSpec {
        method,
        path,
        access: EndpointAccess::Public,
        audit: EndpointAudit::read_only_for(method),
    }
}

pub(in crate::api) const CONFIG: EndpointSpec = read(EndpointMethod::Get, "/api/captcha/config");
pub(in crate::api) const CHALLENGE: EndpointSpec = read(EndpointMethod::Post, "/api/captcha/challenge");
pub(in crate::api) const REDEEM: EndpointSpec = read(EndpointMethod::Post, "/api/captcha/redeem");

const ENDPOINTS: &[EndpointSpec] = &[CONFIG, CHALLENGE, REDEEM];
const SEGMENTS: &[&[EndpointSpec]] = &[ENDPOINTS];

pub fn endpoint_specs() -> EndpointManifest {
    EndpointManifest::new(SEGMENTS)
}

#[cfg(test)]
mod tests {
    use audit_contract::{EndpointAccess, EndpointAudit, EndpointMethod};

    use super::endpoint_specs;

    #[test]
    fn endpoint_specs_classify_captcha_routes_as_public_read_only() {
        let specs = endpoint_specs();
        let entries = specs.iter().collect::<Vec<_>>();

        specs.validate().unwrap();
        assert_eq!(entries.len(), 3);
        assert!(entries.iter().all(|spec| spec.access == EndpointAccess::Public));
        assert_eq!(entries[0].audit, EndpointAudit::ReadOnly);
        assert!(entries[1..].iter().all(|spec| spec.audit == EndpointAudit::ExplicitReadOnly));
        assert!(
            entries
                .iter()
                .any(|spec| spec.method == EndpointMethod::Post && spec.path == "/api/captcha/challenge")
        );
        assert!(
            entries
                .iter()
                .any(|spec| spec.method == EndpointMethod::Post && spec.path == "/api/captcha/redeem")
        );
    }
}
