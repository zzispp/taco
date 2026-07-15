use ::rbac::application::{AuthWhitelistRule, AuthorizationConfig};
use configuration::Settings;

use super::access_catalog::EndpointCatalog;
use crate::BackendResult;

const GET: &[&str] = &["GET"];
const AVATAR_FILES_PATH: &str = "/uploads/avatars/{*file}";

pub(super) fn authorization_config(settings: &Settings, endpoints: &EndpointCatalog) -> BackendResult<AuthorizationConfig> {
    Ok(AuthorizationConfig::compile(auth_whitelist(settings, endpoints), endpoints.permission_rules())?)
}

pub(super) fn auth_whitelist(settings: &Settings, endpoints: &EndpointCatalog) -> Vec<AuthWhitelistRule> {
    let mut rules = settings
        .auth
        .whitelist
        .iter()
        .map(|rule| AuthWhitelistRule {
            methods: rule.methods.clone(),
            path_pattern: rule.path_pattern.clone(),
        })
        .collect::<Vec<_>>();
    ensure_auth_whitelist_rule(&mut rules, GET, AVATAR_FILES_PATH);
    for spec in endpoints.public_specs() {
        ensure_auth_whitelist_rule(&mut rules, &[spec.method.as_str()], spec.path);
    }
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
