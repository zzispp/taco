use ::rbac::application::RoutePermissionRule;

use super::{DELETE, GET, POST, PUT, from_specs};

pub(super) fn routes() -> Vec<RoutePermissionRule> {
    let mut rules = user_routes();
    rules.extend(online_routes());
    rules.extend(role_routes());
    rules.extend(menu_routes());
    rules
}

pub(super) fn data_scope_handlers() -> Vec<&'static str> {
    vec![
        "list_users",
        "export_users",
        "get_user",
        "replace_user",
        "delete_user",
        "delete_users",
        "reset_user_password",
        "update_user_status",
        "user_roles",
        "replace_user_roles",
        "list_roles",
        "export_roles",
        "role_users",
        "replace_role_users",
        "delete_role_user",
        "delete_role_users",
        "list_online_sessions",
        "force_logout_online_session",
        "list_depts",
        "dept_tree_select",
        "get_dept",
        "replace_dept",
        "update_dept_sort",
        "update_dept_sorts",
        "delete_dept",
    ]
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

fn online_routes() -> Vec<RoutePermissionRule> {
    from_specs(&[
        rule_spec!(GET, "/api/system/online/list", "system:online:list", "list_online_sessions"),
        rule_spec!(
            DELETE,
            "/api/system/online/{token_id}",
            "system:online:forceLogout",
            "force_logout_online_session"
        ),
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
