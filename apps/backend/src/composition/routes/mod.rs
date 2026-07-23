use ::rbac::application::{AuthWhitelistRule, AuthorizationConfig};

use super::access_catalog::EndpointCatalog;
use crate::BackendResult;

const GET: &[&str] = &["GET"];
const HEALTH_PATH: &str = "/health";
const READY_PATH: &str = "/ready";
const METRICS_PATH: &str = "/metrics";

pub(super) fn authorization_config(endpoints: &EndpointCatalog) -> BackendResult<AuthorizationConfig> {
    Ok(AuthorizationConfig::compile(auth_whitelist(endpoints), endpoints.permission_rules())?)
}

pub(super) fn auth_whitelist(endpoints: &EndpointCatalog) -> Vec<AuthWhitelistRule> {
    let mut rules = fixed_auth_whitelist();
    for spec in endpoints.public_specs() {
        ensure_auth_whitelist_rule(&mut rules, &[spec.method.as_str()], spec.path);
    }
    rules
}

fn fixed_auth_whitelist() -> Vec<AuthWhitelistRule> {
    [HEALTH_PATH, READY_PATH, METRICS_PATH]
        .into_iter()
        .map(|path_pattern| AuthWhitelistRule {
            methods: GET.iter().map(|method| (*method).into()).collect(),
            path_pattern: path_pattern.into(),
        })
        .collect()
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
