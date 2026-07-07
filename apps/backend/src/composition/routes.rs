use rbac::{
    application::{AuthWhitelistRule, AuthorizationConfig},
    domain::RoutePermissionRule,
};

use configuration::Settings;

const GET: &[&str] = &["GET"];
const POST: &[&str] = &["POST"];
const PUT: &[&str] = &["PUT"];
const DELETE: &[&str] = &["DELETE"];

#[derive(Clone, Copy)]
struct RouteRuleSpec {
    methods: &'static [&'static str],
    path_pattern: &'static str,
    permission: &'static str,
    handler: &'static str,
}

macro_rules! rule_spec {
    ($methods:expr, $path_pattern:expr, $permission:expr, $handler:expr $(,)?) => {
        RouteRuleSpec {
            methods: $methods,
            path_pattern: $path_pattern,
            permission: $permission,
            handler: $handler,
        }
    };
}

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
    let mut rules = dashboard_routes();
    rules.extend(user_routes());
    rules.extend(role_routes());
    rules.extend(menu_routes());
    rules.extend(dept_routes());
    rules.extend(post_routes());
    rules.extend(dict_routes());
    rules.extend(config_routes());
    rules
}

pub(super) fn data_scope_handlers() -> Vec<&'static str> {
    vec![
        "list_users",
        "export_users",
        "list_roles",
        "export_roles",
        "role_users",
        "list_depts",
        "dept_tree_select",
    ]
}

fn dashboard_routes() -> Vec<RoutePermissionRule> {
    from_specs(&[RouteRuleSpec {
        methods: GET,
        path_pattern: "/api/system/dashboard",
        permission: "system:dashboard:view",
        handler: "get_server_dashboard",
    }])
}

fn user_routes() -> Vec<RoutePermissionRule> {
    from_specs(&[
        rule_spec!(GET, "/api/system/users", "system:user:list", "list_users"),
        rule_spec!(POST, "/api/system/users", "system:user:add", "create_user"),
        rule_spec!(POST, "/api/system/users/export", "system:user:export", "export_users"),
        rule_spec!(POST, "/api/system/users/import", "system:user:import", "import_users"),
        rule_spec!(POST, "/api/system/users/import-template", "system:user:import", "user_import_template"),
        rule_spec!(GET, "/api/system/users/dept-tree", "system:user:list", "user_dept_tree"),
        rule_spec!(GET, "/api/system/users/form-options", "system:user:list", "user_form_options"),
        rule_spec!(GET, "/api/system/users/{id}", "system:user:query", "get_user"),
        rule_spec!(PUT, "/api/system/users/{id}", "system:user:edit", "replace_user"),
        rule_spec!(DELETE, "/api/system/users/{id}", "system:user:remove", "delete_user"),
        rule_spec!(DELETE, "/api/system/users/batch", "system:user:remove", "delete_users"),
        rule_spec!(PUT, "/api/system/users/{id}/password", "system:user:resetPwd", "reset_user_password"),
        rule_spec!(PUT, "/api/system/users/{id}/status", "system:user:edit", "update_user_status"),
        rule_spec!(GET, "/api/system/users/{id}/roles", "system:user:query", "user_roles"),
        rule_spec!(PUT, "/api/system/users/{id}/roles", "system:user:edit", "replace_user_roles"),
    ])
}

fn role_routes() -> Vec<RoutePermissionRule> {
    from_specs(&[
        rule_spec!(GET, "/api/system/roles", "system:role:list", "list_roles"),
        rule_spec!(POST, "/api/system/roles", "system:role:add", "create_role"),
        rule_spec!(POST, "/api/system/roles/export", "system:role:export", "export_roles"),
        rule_spec!(GET, "/api/system/roles/options", "system:role:list", "role_options"),
        rule_spec!(GET, "/api/system/roles/{id}", "system:role:query", "get_role"),
        rule_spec!(PUT, "/api/system/roles/{id}", "system:role:edit", "replace_role"),
        rule_spec!(DELETE, "/api/system/roles/{id}", "system:role:remove", "delete_role"),
        rule_spec!(DELETE, "/api/system/roles/batch", "system:role:remove", "delete_roles"),
        rule_spec!(PUT, "/api/system/roles/{id}/status", "system:role:edit", "update_role_status"),
        rule_spec!(PUT, "/api/system/roles/{id}/data-scope", "system:role:edit", "update_role_data_scope"),
        rule_spec!(GET, "/api/system/roles/{id}/menus", "system:role:query", "role_menu_bindings"),
        rule_spec!(PUT, "/api/system/roles/{id}/menus", "system:role:edit", "replace_role_menus"),
        rule_spec!(GET, "/api/system/roles/{id}/depts", "system:role:query", "role_dept_bindings"),
        rule_spec!(PUT, "/api/system/roles/{id}/depts", "system:role:edit", "replace_role_depts"),
        rule_spec!(GET, "/api/system/roles/{id}/users", "system:role:list", "role_users"),
        rule_spec!(PUT, "/api/system/roles/{id}/users", "system:role:edit", "replace_role_users"),
        rule_spec!(DELETE, "/api/system/roles/{id}/users/batch", "system:role:remove", "delete_role_users"),
        rule_spec!(DELETE, "/api/system/roles/{id}/users/{user_id}", "system:role:remove", "delete_role_user"),
        rule_spec!(GET, "/api/system/roles/{id}/dept-tree-select", "system:role:query", "role_dept_tree_select"),
    ])
}

fn menu_routes() -> Vec<RoutePermissionRule> {
    from_specs(&[
        rule_spec!(GET, "/api/system/menus", "system:menu:list", "list_menus"),
        rule_spec!(POST, "/api/system/menus", "system:menu:add", "create_menu"),
        rule_spec!(GET, "/api/system/menus/tree", "system:menu:list", "list_menu_tree"),
        rule_spec!(GET, "/api/system/menus/tree-select", "system:menu:list", "menu_tree_select"),
        rule_spec!(GET, "/api/system/menus/role-tree-select/{id}", "system:role:query", "role_menu_tree_select"),
        rule_spec!(GET, "/api/system/menus/{id}", "system:menu:query", "get_menu"),
        rule_spec!(PUT, "/api/system/menus/{id}", "system:menu:edit", "replace_menu"),
        rule_spec!(DELETE, "/api/system/menus/{id}", "system:menu:remove", "delete_menu"),
        rule_spec!(PUT, "/api/system/menus/{id}/sort", "system:menu:edit", "update_menu_sort"),
        rule_spec!(PUT, "/api/system/menus/sort", "system:menu:edit", "update_menu_sorts"),
    ])
}

fn dept_routes() -> Vec<RoutePermissionRule> {
    from_specs(&[
        rule_spec!(GET, "/api/system/depts", "system:dept:list", "list_depts"),
        rule_spec!(POST, "/api/system/depts", "system:dept:add", "create_dept"),
        rule_spec!(GET, "/api/system/depts/tree-select", "system:dept:list", "dept_tree_select"),
        rule_spec!(GET, "/api/system/depts/exclude/{id}", "system:dept:list", "exclude_dept_tree"),
        rule_spec!(GET, "/api/system/depts/{id}", "system:dept:query", "get_dept"),
        rule_spec!(PUT, "/api/system/depts/{id}", "system:dept:edit", "replace_dept"),
        rule_spec!(DELETE, "/api/system/depts/{id}", "system:dept:remove", "delete_dept"),
        rule_spec!(PUT, "/api/system/depts/{id}/sort", "system:dept:edit", "update_dept_sort"),
        rule_spec!(PUT, "/api/system/depts/sort", "system:dept:edit", "update_dept_sorts"),
    ])
}

fn post_routes() -> Vec<RoutePermissionRule> {
    from_specs(&[
        rule_spec!(GET, "/api/system/posts", "system:post:list", "list_posts"),
        rule_spec!(POST, "/api/system/posts", "system:post:add", "create_post"),
        rule_spec!(POST, "/api/system/posts/export", "system:post:export", "export_posts"),
        rule_spec!(GET, "/api/system/posts/options", "system:post:list", "post_options"),
        rule_spec!(GET, "/api/system/posts/{id}", "system:post:query", "get_post"),
        rule_spec!(PUT, "/api/system/posts/{id}", "system:post:edit", "replace_post"),
        rule_spec!(DELETE, "/api/system/posts/{id}", "system:post:remove", "delete_post"),
        rule_spec!(DELETE, "/api/system/posts/batch", "system:post:remove", "delete_posts"),
    ])
}

fn dict_routes() -> Vec<RoutePermissionRule> {
    from_specs(&[
        rule_spec!(GET, "/api/system/dict-types", "system:dict:list", "list_dict_types"),
        rule_spec!(POST, "/api/system/dict-types", "system:dict:add", "create_dict_type"),
        rule_spec!(POST, "/api/system/dict-types/export", "system:dict:export", "export_dict_types"),
        rule_spec!(GET, "/api/system/dict-types/options", "system:dict:list", "dict_type_options"),
        rule_spec!(DELETE, "/api/system/dict-types/cache", "system:dict:remove", "refresh_dict_cache"),
        rule_spec!(GET, "/api/system/dict-types/{id}", "system:dict:query", "get_dict_type"),
        rule_spec!(PUT, "/api/system/dict-types/{id}", "system:dict:edit", "replace_dict_type"),
        rule_spec!(DELETE, "/api/system/dict-types/{id}", "system:dict:remove", "delete_dict_type"),
        rule_spec!(DELETE, "/api/system/dict-types/batch", "system:dict:remove", "delete_dict_types"),
        rule_spec!(GET, "/api/system/dict-data", "system:dict:list", "list_dict_data"),
        rule_spec!(POST, "/api/system/dict-data", "system:dict:add", "create_dict_data"),
        rule_spec!(POST, "/api/system/dict-data/export", "system:dict:export", "export_dict_data"),
        rule_spec!(GET, "/api/system/dict-data/type/{dict_type}", "system:dict:list", "dict_data_by_type"),
        rule_spec!(GET, "/api/system/dict-data/{id}", "system:dict:query", "get_dict_data"),
        rule_spec!(PUT, "/api/system/dict-data/{id}", "system:dict:edit", "replace_dict_data"),
        rule_spec!(DELETE, "/api/system/dict-data/{id}", "system:dict:remove", "delete_dict_data"),
        rule_spec!(DELETE, "/api/system/dict-data/batch", "system:dict:remove", "delete_dict_data_batch"),
    ])
}

fn config_routes() -> Vec<RoutePermissionRule> {
    from_specs(&[
        rule_spec!(GET, "/api/system/configs", "system:config:list", "list_configs"),
        rule_spec!(POST, "/api/system/configs", "system:config:add", "create_config"),
        rule_spec!(POST, "/api/system/configs/export", "system:config:export", "export_configs"),
        rule_spec!(DELETE, "/api/system/configs/cache", "system:config:remove", "refresh_config_cache"),
        rule_spec!(GET, "/api/system/configs/key/{key}", "system:config:query", "config_by_key"),
        rule_spec!(GET, "/api/system/configs/{id}", "system:config:query", "get_config"),
        rule_spec!(PUT, "/api/system/configs/{id}", "system:config:edit", "replace_config"),
        rule_spec!(DELETE, "/api/system/configs/{id}", "system:config:remove", "delete_config"),
        rule_spec!(DELETE, "/api/system/configs/batch", "system:config:remove", "delete_configs"),
    ])
}

fn from_specs(specs: &[RouteRuleSpec]) -> Vec<RoutePermissionRule> {
    specs.iter().map(|spec| route_rule(*spec)).collect()
}

fn route_rule(spec: RouteRuleSpec) -> RoutePermissionRule {
    RoutePermissionRule {
        methods: spec.methods.iter().map(|method| (*method).into()).collect(),
        path_pattern: spec.path_pattern.into(),
        permission: spec.permission.into(),
        handler: spec.handler,
    }
}
