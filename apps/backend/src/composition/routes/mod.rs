use ::rbac::application::{AuthWhitelistRule, AuthorizationConfig, PermissionRequirement, RoutePermissionRule};
use configuration::Settings;

const GET: &[&str] = &["GET"];
const POST: &[&str] = &["POST"];
const PUT: &[&str] = &["PUT"];
const DELETE: &[&str] = &["DELETE"];

#[derive(Clone, Copy)]
struct RouteRuleSpec {
    methods: &'static [&'static str],
    path_pattern: &'static str,
    requirement: PermissionRequirement,
    handler: &'static str,
}

macro_rules! rule_spec {
    ($methods:expr, $path_pattern:expr, $permission:expr, $handler:expr $(,)?) => {
        crate::composition::routes::RouteRuleSpec {
            methods: $methods,
            path_pattern: $path_pattern,
            requirement: ::rbac::application::PermissionRequirement::all_of(&[$permission]),
            handler: $handler,
        }
    };
}

mod rbac;
mod scheduler;
mod system;

pub(super) fn authorization_config(settings: &Settings) -> AuthorizationConfig {
    AuthorizationConfig {
        whitelist: auth_whitelist(settings),
        route_permissions: route_permissions(),
    }
}

pub(super) fn auth_whitelist(settings: &Settings) -> Vec<AuthWhitelistRule> {
    let mut rules = settings
        .auth
        .whitelist
        .iter()
        .map(|rule| AuthWhitelistRule {
            methods: rule.methods.clone(),
            path_pattern: rule.path_pattern.clone(),
        })
        .collect::<Vec<_>>();
    ensure_auth_whitelist_rule(&mut rules, GET, "/api/app/configs");
    ensure_auth_whitelist_rule(&mut rules, GET, "/api/auth/me");
    ensure_auth_whitelist_rule(&mut rules, GET, "/uploads/avatars/{*file}");
    ensure_auth_whitelist_rule(&mut rules, GET, "/api/captcha/config");
    ensure_auth_whitelist_rule(&mut rules, POST, "/api/captcha/challenge");
    ensure_auth_whitelist_rule(&mut rules, POST, "/api/captcha/redeem");
    rules
}

pub(super) fn ensure_auth_whitelist_rule(rules: &mut Vec<AuthWhitelistRule>, methods: &[&str], path_pattern: &str) {
    let exists = rules
        .iter()
        .any(|rule| rule.path_pattern == path_pattern && methods.iter().all(|method| rule.methods.iter().any(|item| item.eq_ignore_ascii_case(method))));
    if exists {
        return;
    }
    rules.push(AuthWhitelistRule {
        methods: methods.iter().map(|method| (*method).into()).collect(),
        path_pattern: path_pattern.into(),
    });
}

pub(super) fn route_permissions() -> Vec<RoutePermissionRule> {
    let mut rules = rbac::routes();
    rules.extend(system::routes());
    rules.extend(scheduler::routes());
    rules
}

pub(super) fn data_scope_handlers() -> Vec<&'static str> {
    rbac::data_scope_handlers()
}

fn from_specs(specs: &[RouteRuleSpec]) -> Vec<RoutePermissionRule> {
    specs.iter().map(|spec| route_rule(*spec)).collect()
}

fn route_rule(spec: RouteRuleSpec) -> RoutePermissionRule {
    RoutePermissionRule {
        methods: spec.methods.iter().map(|method| (*method).into()).collect(),
        path_pattern: spec.path_pattern.into(),
        requirement: spec.requirement,
        handler: spec.handler,
    }
}
