use sqlx::{PgPool, query_scalar};

const EXPECTED_DASHBOARD_MENU_COUNT: i64 = 1;
const EXPECTED_ONLINE_MENU_COUNT: i64 = 1;
const EXPECTED_ONLINE_QUERY_PERMISSION_COUNT: i64 = 1;
const EXPECTED_ONLINE_FORCE_LOGOUT_PERMISSION_COUNT: i64 = 1;
const EXPECTED_OVERVIEW_MENU_COUNT: i64 = 1;
const EXPECTED_OVERVIEW_ROLE_BINDING_COUNT: i64 = 1;
const EXPECTED_SYSTEM_MONITOR_MENU_COUNT: i64 = 1;
const EXPECTED_SYSTEM_MONITOR_ROLE_BINDING_COUNT: i64 = 1;
const EXPECTED_MENU_ICONS: &[(&str, &str)] = &[
    ("3", "icon.monitor"),
    ("4", "icon.dashboard"),
    ("103", "icon.dept"),
    ("104", "icon.post"),
    ("105", "icon.dict"),
    ("106", "icon.config"),
    ("107", "icon.online"),
    ("108", "icon.job"),
    ("109", "icon.job-log"),
    ("110", "icon.notice"),
    ("111", "icon.logs"),
    ("114", "icon.system-log"),
];

pub(super) async fn assert_navigation_seed(pool: &PgPool) {
    assert_overview_menu_exists(pool).await;
    assert_dashboard_menu_exists(pool).await;
    assert_system_monitor_menu_exists(pool).await;
    assert_online_menu_exists(pool).await;
    assert_online_query_permission_exists(pool).await;
    assert_online_force_logout_permission_exists(pool).await;
    assert_menu_icons(pool).await;
}

async fn assert_overview_menu_exists(pool: &PgPool) {
    let count: i64 = query_scalar(
        "SELECT COUNT(*) FROM sys_menu WHERE menu_id='4' AND menu_name='概览' AND parent_id='0' AND order_num=0 AND path='/dashboard/overview' AND menu_type='M' AND icon='icon.dashboard' AND visible='0' AND status='0'",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(count, EXPECTED_OVERVIEW_MENU_COUNT);

    let binding_count: i64 = query_scalar("SELECT COUNT(*) FROM sys_role_menu WHERE role_id='2' AND menu_id='4'")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(binding_count, EXPECTED_OVERVIEW_ROLE_BINDING_COUNT);
}

async fn assert_dashboard_menu_exists(pool: &PgPool) {
    let count: i64 = query_scalar(
        "SELECT COUNT(*) FROM sys_menu WHERE menu_id='2' AND path = '/dashboard' AND perms = 'system:dashboard:view' AND parent_id='4' AND order_num=1 AND menu_type='C' AND visible = '0' AND status = '0'",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(count, EXPECTED_DASHBOARD_MENU_COUNT);
}

async fn assert_system_monitor_menu_exists(pool: &PgPool) {
    let menu_count: i64 = query_scalar(
        "SELECT COUNT(*) FROM sys_menu WHERE menu_id='3' AND menu_name='系统监控' AND parent_id='0' AND order_num=2 AND path='/dashboard/monitor' AND menu_type='M' AND icon='icon.monitor' AND visible='0' AND status='0'",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(menu_count, EXPECTED_SYSTEM_MONITOR_MENU_COUNT);

    let binding_count: i64 = query_scalar("SELECT COUNT(*) FROM sys_role_menu WHERE role_id='2' AND menu_id='3'")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(binding_count, EXPECTED_SYSTEM_MONITOR_ROLE_BINDING_COUNT);
}

async fn assert_online_menu_exists(pool: &PgPool) {
    let count: i64 = query_scalar(
        "SELECT COUNT(*) FROM sys_menu WHERE path = '/dashboard/admin/online' AND perms = 'system:online:list' AND parent_id = '3' AND order_num = 1 AND visible = '0' AND status = '0'",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(count, EXPECTED_ONLINE_MENU_COUNT);
}

async fn assert_online_force_logout_permission_exists(pool: &PgPool) {
    let count: i64 =
        query_scalar("SELECT COUNT(*) FROM sys_menu WHERE parent_id = '107' AND perms = 'system:online:forceLogout' AND menu_type = 'F' AND status = '0'")
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(count, EXPECTED_ONLINE_FORCE_LOGOUT_PERMISSION_COUNT);
}

async fn assert_online_query_permission_exists(pool: &PgPool) {
    let count: i64 =
        query_scalar("SELECT COUNT(*) FROM sys_menu WHERE parent_id = '107' AND perms = 'system:online:query' AND menu_type = 'F' AND status = '0'")
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(count, EXPECTED_ONLINE_QUERY_PERMISSION_COUNT);
}

async fn assert_menu_icons(pool: &PgPool) {
    for (menu_id, icon) in EXPECTED_MENU_ICONS {
        let count: i64 = query_scalar("SELECT COUNT(*) FROM sys_menu WHERE menu_id = $1 AND icon = $2")
            .bind(*menu_id)
            .bind(*icon)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "menu {menu_id} should use icon {icon}");
    }
}
