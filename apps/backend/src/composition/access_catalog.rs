use std::{collections::BTreeMap, sync::Arc};

use audit_contract::{EndpointAccess, EndpointPermissionRequirement, EndpointSpec, validate_endpoint_specs};
use matchit::Router;
use rbac::application::{PermissionRequirement, RoutePermissionRule};

use crate::BackendResult;

#[derive(Clone)]
pub(crate) struct EndpointCatalog {
    specs: Arc<[EndpointSpec]>,
    matcher: Arc<Router<Vec<EndpointSpec>>>,
}

impl EndpointCatalog {
    pub(crate) fn build() -> BackendResult<Self> {
        Self::from_specs(collect_endpoint_specs())
    }

    pub(crate) fn specs(&self) -> &[EndpointSpec] {
        &self.specs
    }

    pub(crate) fn access(&self, method: &str, path: &str) -> Option<EndpointAccess> {
        self.matcher
            .at(path)
            .ok()?
            .value
            .iter()
            .find(|spec| spec.method.as_str().eq_ignore_ascii_case(method))
            .map(|spec| spec.access)
    }

    pub(crate) fn permission_rules(&self) -> Vec<RoutePermissionRule> {
        self.specs.iter().filter_map(permission_rule).collect()
    }

    pub(crate) fn public_specs(&self) -> impl Iterator<Item = &EndpointSpec> {
        self.specs.iter().filter(|spec| is_public(spec.access))
    }

    fn from_specs(specs: Vec<EndpointSpec>) -> BackendResult<Self> {
        validate_endpoint_specs(&specs)?;
        let matcher = build_matcher(&specs)?;
        Ok(Self {
            specs: Arc::from(specs),
            matcher: Arc::new(matcher),
        })
    }
}

fn collect_endpoint_specs() -> Vec<EndpointSpec> {
    let mut specs = captcha::api::endpoint_specs().iter().copied().collect::<Vec<_>>();
    specs.extend(user::api::endpoint_specs().iter().copied());
    specs.extend(rbac::api::endpoint_specs().iter().copied());
    specs.extend(::system::api::endpoint_specs().iter().copied());
    specs.extend(::system::notice::endpoint_specs().iter().copied());
    specs.extend(scheduler::api::endpoint_specs().iter().copied());
    specs.extend(audit::api::endpoint_specs().iter().copied());
    specs.extend(observability::api::endpoint_specs().iter().copied());
    specs.extend(file::api::endpoint_specs().iter().copied());
    specs
}

fn build_matcher(specs: &[EndpointSpec]) -> BackendResult<Router<Vec<EndpointSpec>>> {
    let mut grouped = BTreeMap::<&str, Vec<EndpointSpec>>::new();
    for spec in specs {
        grouped.entry(spec.path).or_default().push(*spec);
    }

    let mut matcher = Router::new();
    for (path, path_specs) in grouped {
        matcher.insert(path, path_specs)?;
    }
    Ok(matcher)
}

fn permission_rule(spec: &EndpointSpec) -> Option<RoutePermissionRule> {
    let permission = match spec.access {
        EndpointAccess::Permission(permission) | EndpointAccess::DataScopedPermission(permission) => permission,
        EndpointAccess::Public | EndpointAccess::SelfAuthenticated | EndpointAccess::Authenticated => return None,
    };
    Some(RoutePermissionRule {
        methods: vec![spec.method.as_str().into()],
        path_pattern: spec.path.into(),
        requirement: permission_requirement(permission.requirement),
        handler: permission.handler,
    })
}

fn permission_requirement(value: EndpointPermissionRequirement) -> PermissionRequirement {
    match value {
        EndpointPermissionRequirement::AllOf(values) => PermissionRequirement::all_of(values),
        EndpointPermissionRequirement::AnyOf(values) => PermissionRequirement::any_of(values),
    }
}

fn is_public(access: EndpointAccess) -> bool {
    matches!(access, EndpointAccess::Public)
}

#[cfg(test)]
mod tests {
    use audit_contract::{EndpointAccess, EndpointAudit, EndpointMethod, EndpointPermission, EndpointPermissionRequirement, EndpointSpec};

    use super::{EndpointCatalog, collect_endpoint_specs};

    const EXPECTED_OPERATION_ENDPOINTS: usize = 99;

    #[test]
    fn matcher_distinguishes_methods_on_the_same_dynamic_route() {
        let catalog = EndpointCatalog::from_specs(vec![
            read(EndpointMethod::Get, "/api/resources/{id}", EndpointAccess::Authenticated),
            read(
                EndpointMethod::Delete,
                "/api/resources/{id}",
                EndpointAccess::Permission(EndpointPermission {
                    handler: "delete_resource",
                    requirement: EndpointPermissionRequirement::all_of(&["system:resource:remove"]),
                }),
            ),
        ])
        .unwrap();

        assert_eq!(catalog.access("GET", "/api/resources/item-1"), Some(EndpointAccess::Authenticated));
        assert!(matches!(catalog.access("DELETE", "/api/resources/item-1"), Some(EndpointAccess::Permission(_))));
        assert_eq!(catalog.access("POST", "/api/resources/item-1"), None);
    }

    #[test]
    fn permission_rules_preserve_any_of_requirements() {
        let catalog = EndpointCatalog::from_specs(vec![read(
            EndpointMethod::Post,
            "/api/resources/import",
            EndpointAccess::Permission(EndpointPermission {
                handler: "import_resource",
                requirement: EndpointPermissionRequirement::any_of(&["system:resource:import", "system:resource:edit"]),
            }),
        )])
        .unwrap();

        let rules = catalog.permission_rules();

        assert_eq!(rules.len(), 1);
        assert!(rules[0].requirement.is_satisfied_by(&["system:resource:edit".into()]));
        assert!(!rules[0].requirement.is_satisfied_by(&["system:resource:list".into()]));
    }

    #[test]
    fn public_specs_exclude_self_authenticated_routes() {
        let catalog = EndpointCatalog::from_specs(vec![
            read(EndpointMethod::Get, "/api/public", EndpointAccess::Public),
            read(EndpointMethod::Get, "/api/self", EndpointAccess::SelfAuthenticated),
        ])
        .unwrap();

        let paths = catalog.public_specs().map(|spec| spec.path).collect::<Vec<_>>();

        assert_eq!(paths, vec!["/api/public"]);
    }

    #[test]
    fn aggregate_manifest_covers_all_management_operations_and_explicit_read_only_posts() {
        let specs = collect_endpoint_specs();
        let operations = specs
            .iter()
            .filter(|spec| matches!(spec.audit, EndpointAudit::Operation(_) | EndpointAudit::Download(_)))
            .count();

        assert_eq!(operations, EXPECTED_OPERATION_ENDPOINTS);
        assert_read_only_post(&specs, "/api/system/users/import-template");
        assert_read_only_post(&specs, "/api/system/jobs/cron/next-times");
    }

    fn assert_read_only_post(specs: &[EndpointSpec], path: &'static str) {
        assert!(specs.iter().any(|spec| {
            spec.method == EndpointMethod::Post
                && spec.path == path
                && matches!(spec.access, EndpointAccess::Permission(_) | EndpointAccess::DataScopedPermission(_))
                && spec.audit == EndpointAudit::ExplicitReadOnly
        }));
    }

    fn read(method: EndpointMethod, path: &'static str, access: EndpointAccess) -> EndpointSpec {
        EndpointSpec {
            method,
            path,
            access,
            audit: EndpointAudit::read_only_for(method),
        }
    }
}
