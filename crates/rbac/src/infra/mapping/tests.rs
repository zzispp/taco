use super::*;

const DASHBOARD_MENU_ID: &str = "2";
const DASHBOARD_PATH: &str = "/dashboard";
const ADMIN_MENU_ID: &str = "1";
const ADMIN_PATH: &str = "/dashboard/admin";
const USERS_MENU_ID: &str = "100";
const USERS_PATH: &str = "/dashboard/admin/users";
const REPORT_MENU_ID: &str = "200";
const REPORT_RELATIVE_PATH: &str = "reports";
const REPORT_ABSOLUTE_PATH: &str = "/reports";

#[test]
fn dashboard_menu_maps_to_dashboard_root_with_exact_match() {
    let sections = nav_sections(vec![row(TestRoleMenuRecord {
        menu_id: DASHBOARD_MENU_ID,
        menu_name: "仪表盘",
        parent_id: MENU_ROOT_PARENT_ID,
        path: DASHBOARD_PATH,
        menu_type: MENU_TYPE_MENU,
    })]);

    assert_eq!(sections.len(), 1);
    assert_eq!(sections[0].code, NAV_OVERVIEW_SECTION_CODE);
    assert_eq!(sections[0].items[0].path, DASHBOARD_PATH);
    assert!(!sections[0].items[0].deep_match);
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
    let sections = nav_sections(vec![row(TestRoleMenuRecord {
        menu_id: REPORT_MENU_ID,
        menu_name: "报表",
        parent_id: MENU_ROOT_PARENT_ID,
        path: REPORT_RELATIVE_PATH,
        menu_type: MENU_TYPE_MENU,
    })]);

    assert_eq!(sections[0].items[0].path, REPORT_ABSOLUTE_PATH);
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
