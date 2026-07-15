use super::*;

const DASHBOARD_MENU_ID: &str = "2";
const DASHBOARD_PATH: &str = "/dashboard";
const OVERVIEW_MENU_ID: &str = "4";
const OVERVIEW_PATH: &str = "/dashboard/overview";
const ADMIN_MENU_ID: &str = "1";
const ADMIN_PATH: &str = "/dashboard/admin";
const USERS_MENU_ID: &str = "100";
const USERS_PATH: &str = "/dashboard/admin/users";
const REPORT_MENU_ID: &str = "200";
const REPORT_RELATIVE_PATH: &str = "reports";
const REPORT_ABSOLUTE_PATH: &str = "/reports";
const MONITOR_MENU_ID: &str = "3";
const LOG_DIRECTORY_ID: &str = "111";
const OPERATION_LOG_MENU_ID: &str = "112";
const LOGIN_LOG_MENU_ID: &str = "113";

#[test]
fn rbac_unique_constraints_map_to_owned_conflicts() {
    for (constraint, key) in [
        ("idx_sys_role_name", "errors.rbac.role_name_exists"),
        ("idx_sys_role_key", "errors.rbac.role_key_exists"),
        ("idx_sys_menu_parent_name", "errors.rbac.menu_name_exists"),
        ("idx_sys_menu_parent_path", "errors.rbac.menu_path_exists"),
        ("idx_sys_menu_route_name", "errors.rbac.route_name_exists"),
    ] {
        let error = storage_error(unique_violation(constraint));
        let RbacError::Conflict(message) = error else {
            panic!("known RBAC unique constraint must map to conflict");
        };
        assert_eq!(message.key(), key);
    }
}

#[test]
fn unknown_rbac_unique_constraint_remains_infrastructure_error() {
    assert!(matches!(storage_error(unique_violation("unknown_unique_index")), RbacError::Infrastructure(message) if message == "duplicate key"));
    assert!(matches!(storage_error(StorageError::Database("connection lost".into())), RbacError::Infrastructure(message) if message == "connection lost"));
}

#[test]
fn role_create_time_is_fixed_millisecond_utc() {
    let create_time = time::OffsetDateTime::parse("2026-07-15T12:34:56.123456789+08:00", &time::format_description::well_known::Rfc3339).unwrap();
    let mapped = role(RoleRecord {
        role_id: "role-1".into(),
        role_name: "Operator".into(),
        role_key: "operator".into(),
        role_sort: 1,
        data_scope: "1".into(),
        menu_check_strictly: true,
        dept_check_strictly: true,
        status: "0".into(),
        system: false,
        remark: None,
        create_time,
    })
    .unwrap();

    assert_eq!(mapped.create_time, "2026-07-15T04:34:56.123Z");
}

fn unique_violation(constraint: &str) -> StorageError {
    StorageError::UniqueViolation {
        constraint: Some(constraint.into()),
        message: "duplicate key".into(),
    }
}

#[test]
fn dashboard_menu_maps_from_overview_directory_with_exact_match() {
    let sections = nav_sections(vec![
        row(TestRoleMenuRecord {
            menu_id: OVERVIEW_MENU_ID,
            menu_name: "概览",
            parent_id: MENU_ROOT_PARENT_ID,
            path: OVERVIEW_PATH,
            menu_type: MENU_TYPE_DIRECTORY,
        }),
        row(TestRoleMenuRecord {
            menu_id: DASHBOARD_MENU_ID,
            menu_name: "仪表盘",
            parent_id: OVERVIEW_MENU_ID,
            path: DASHBOARD_PATH,
            menu_type: MENU_TYPE_MENU,
        }),
    ]);

    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].code, OVERVIEW_MENU_ID);
    assert_eq!(sections[0].subheader, "概览");
    assert_eq!(sections[0].items[0].path, DASHBOARD_PATH);
    assert!(!sections[0].items[0].deep_match);
}

#[test]
fn root_leaf_menu_maps_to_its_own_data_driven_section() {
    let sections = nav_sections(vec![row(TestRoleMenuRecord {
        menu_id: REPORT_MENU_ID,
        menu_name: "报表",
        parent_id: MENU_ROOT_PARENT_ID,
        path: REPORT_RELATIVE_PATH,
        menu_type: MENU_TYPE_MENU,
    })]);

    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].code, REPORT_MENU_ID);
    assert_eq!(sections[0].subheader, "报表");
    assert_eq!(sections[0].items[0].path, REPORT_ABSOLUTE_PATH);
}

#[test]
fn admin_leaf_menu_items_use_exact_match() {
    let sections = nav_sections(vec![
        row(TestRoleMenuRecord {
            menu_id: ADMIN_MENU_ID,
            menu_name: "系统管理",
            parent_id: MENU_ROOT_PARENT_ID,
            path: ADMIN_PATH,
            menu_type: MENU_TYPE_DIRECTORY,
        }),
        row(TestRoleMenuRecord {
            menu_id: USERS_MENU_ID,
            menu_name: "用户管理",
            parent_id: ADMIN_MENU_ID,
            path: USERS_PATH,
            menu_type: MENU_TYPE_MENU,
        }),
    ]);

    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].code, ADMIN_MENU_ID);
    assert_eq!(sections[0].subheader, "系统管理");
    assert_eq!(sections[0].items[0].path, USERS_PATH);
    assert!(!sections[0].items[0].deep_match);
}

#[test]
fn relative_menu_path_is_normalized_without_route_dictionary() {
    let sections = nav_sections(vec![
        row(TestRoleMenuRecord {
            menu_id: OVERVIEW_MENU_ID,
            menu_name: "概览",
            parent_id: MENU_ROOT_PARENT_ID,
            path: OVERVIEW_PATH,
            menu_type: MENU_TYPE_DIRECTORY,
        }),
        row(TestRoleMenuRecord {
            menu_id: REPORT_MENU_ID,
            menu_name: "报表",
            parent_id: OVERVIEW_MENU_ID,
            path: REPORT_RELATIVE_PATH,
            menu_type: MENU_TYPE_MENU,
        }),
    ]);

    assert_eq!(sections[0].items[0].path, REPORT_ABSOLUTE_PATH);
}

#[test]
fn nested_log_directory_preserves_leaf_routes_in_navbar() {
    let sections = nav_sections(vec![
        row(TestRoleMenuRecord {
            menu_id: MONITOR_MENU_ID,
            menu_name: "系统监控",
            parent_id: MENU_ROOT_PARENT_ID,
            path: "/dashboard/monitor",
            menu_type: MENU_TYPE_DIRECTORY,
        }),
        row(TestRoleMenuRecord {
            menu_id: LOG_DIRECTORY_ID,
            menu_name: "日志管理",
            parent_id: MONITOR_MENU_ID,
            path: "/dashboard/monitor/logs",
            menu_type: MENU_TYPE_DIRECTORY,
        }),
        row(TestRoleMenuRecord {
            menu_id: OPERATION_LOG_MENU_ID,
            menu_name: "操作日志",
            parent_id: LOG_DIRECTORY_ID,
            path: "/dashboard/monitor/logs/operation-logs",
            menu_type: MENU_TYPE_MENU,
        }),
        row(TestRoleMenuRecord {
            menu_id: LOGIN_LOG_MENU_ID,
            menu_name: "登录日志",
            parent_id: LOG_DIRECTORY_ID,
            path: "/dashboard/monitor/logs/login-logs",
            menu_type: MENU_TYPE_MENU,
        }),
    ]);

    let logs = &sections[0].items[0];
    assert_eq!(logs.code, LOG_DIRECTORY_ID);
    assert!(logs.deep_match);
    assert_eq!(
        logs.children.iter().map(|item| item.code.as_str()).collect::<Vec<_>>(),
        [OPERATION_LOG_MENU_ID, LOGIN_LOG_MENU_ID]
    );
    assert!(logs.children.iter().all(|item| !item.deep_match));
    assert_eq!(logs.children[0].path, "/dashboard/monitor/logs/operation-logs");
    assert_eq!(logs.children[1].path, "/dashboard/monitor/logs/login-logs");
}

fn row(input: TestRoleMenuRecord<'_>) -> RoleMenuRecord {
    RoleMenuRecord {
        role_key: "common".into(),
        menu_id: input.menu_id.into(),
        menu_name: input.menu_name.into(),
        parent_id: input.parent_id.into(),
        path: input.path.into(),
        menu_type: input.menu_type.into(),
        icon: "icon.dashboard".into(),
        order_num: 0,
    }
}

struct TestRoleMenuRecord<'a> {
    menu_id: &'a str,
    menu_name: &'a str,
    parent_id: &'a str,
    path: &'a str,
    menu_type: &'a str,
}
