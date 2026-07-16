use std::{collections::BTreeMap, sync::Arc};

use kernel::error::LocalizedError;
use matchit::Router;

use super::{AuthWhitelistRule, PermissionRequirement, RbacError, RbacResult, RoutePermissionRule};

#[derive(Clone)]
pub struct AuthorizationConfig {
    route_permissions: Arc<[RoutePermissionRule]>,
    whitelist_matcher: Arc<Router<Vec<AuthWhitelistRule>>>,
    permission_matcher: Arc<Router<Vec<RoutePermissionRule>>>,
}

impl AuthorizationConfig {
    pub fn compile(whitelist: Vec<AuthWhitelistRule>, route_permissions: Vec<RoutePermissionRule>) -> RbacResult<Self> {
        let whitelist_matcher = compile_matcher(&whitelist)?;
        let permission_matcher = compile_matcher(&route_permissions)?;
        Ok(Self {
            route_permissions: Arc::from(route_permissions),
            whitelist_matcher: Arc::new(whitelist_matcher),
            permission_matcher: Arc::new(permission_matcher),
        })
    }

    pub fn route_permissions(&self) -> &[RoutePermissionRule] {
        &self.route_permissions
    }

    pub(crate) fn is_whitelisted(&self, method: &str, path: &str) -> bool {
        matching_rules(&self.whitelist_matcher, method, path).next().is_some()
    }

    pub(crate) fn required_permission(&self, method: &str, path: &str) -> Option<PermissionRequirement> {
        matching_rules(&self.permission_matcher, method, path).next().map(|rule| rule.requirement)
    }
}

trait RoutePattern {
    fn methods(&self) -> &[String];
    fn path_pattern(&self) -> &str;
}

impl RoutePattern for AuthWhitelistRule {
    fn methods(&self) -> &[String] {
        &self.methods
    }

    fn path_pattern(&self) -> &str {
        &self.path_pattern
    }
}

impl RoutePattern for RoutePermissionRule {
    fn methods(&self) -> &[String] {
        &self.methods
    }

    fn path_pattern(&self) -> &str {
        &self.path_pattern
    }
}

fn compile_matcher<T: Clone + RoutePattern>(rules: &[T]) -> RbacResult<Router<Vec<T>>> {
    let mut grouped = BTreeMap::<&str, Vec<T>>::new();
    for rule in rules {
        grouped.entry(rule.path_pattern()).or_default().push(rule.clone());
    }
    let mut matcher = Router::new();
    for (pattern, path_rules) in grouped {
        matcher.insert(pattern, path_rules).map_err(|error| invalid_route_pattern(pattern, &error))?;
    }
    Ok(matcher)
}

fn matching_rules<'a, T: RoutePattern>(matcher: &'a Router<Vec<T>>, method: &'a str, path: &str) -> impl Iterator<Item = &'a T> {
    matcher
        .at(path)
        .ok()
        .into_iter()
        .flat_map(|matched| matched.value.iter())
        .filter(move |rule| rule.methods().iter().any(|candidate| candidate.eq_ignore_ascii_case(method)))
}

fn invalid_route_pattern(pattern: &str, error: &matchit::InsertError) -> RbacError {
    taco_tracing::error_with_fields!("RBAC route pattern compilation failed", error, path_pattern = pattern);
    RbacError::InvalidInput(LocalizedError::new("errors.rbac.invalid_route_pattern"))
}
