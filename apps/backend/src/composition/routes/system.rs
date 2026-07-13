use ::rbac::application::RoutePermissionRule;

use super::{DELETE, GET, POST, PUT, from_specs};

pub(super) fn routes() -> Vec<RoutePermissionRule> {
    let mut rules = dashboard_routes();
    rules.extend(dept_routes());
    rules.extend(post_routes());
    rules.extend(dict_routes());
    rules.extend(config_routes());
    rules.extend(notice_routes());
    rules
}

fn dashboard_routes() -> Vec<RoutePermissionRule> {
    from_specs(&[rule_spec!(GET, "/api/system/dashboard", "system:dashboard:view", "get_server_dashboard")])
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

fn notice_routes() -> Vec<RoutePermissionRule> {
    from_specs(&[
        rule_spec!(GET, "/api/system/notices", "system:notice:list", "list_notices"),
        rule_spec!(POST, "/api/system/notices", "system:notice:add", "create_notice"),
        rule_spec!(PUT, "/api/system/notices/{id}", "system:notice:edit", "replace_notice"),
        rule_spec!(DELETE, "/api/system/notices/{id}", "system:notice:remove", "delete_notice"),
        rule_spec!(DELETE, "/api/system/notices/batch", "system:notice:remove", "delete_notices"),
        rule_spec!(GET, "/api/system/notices/{id}/readers", "system:notice:list", "list_notice_readers"),
    ])
}
