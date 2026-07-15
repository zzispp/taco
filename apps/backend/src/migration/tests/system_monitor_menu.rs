use sqlx::{PgPool, query, query_as, query_scalar};

use super::{TestDatabase, down, up};

const MIGRATIONS_BEFORE_SYSTEM_MONITOR: u32 = 14;
const SYSTEM_MONITOR_MENU_ID: &str = "3";
const MOVED_MENUS: &[(&str, &str, i64)] = &[("107", "3", 1), ("108", "3", 2), ("109", "3", 3)];
const RESTORED_MENUS: &[(&str, &str, i64)] = &[("107", "1", 8), ("108", "1", 9), ("109", "1", 10)];
const SYSTEM_MANAGEMENT_MENUS: &[(&str, i64)] = &[("100", 1), ("101", 2), ("102", 3), ("103", 4), ("104", 5), ("105", 6), ("106", 7), ("110", 11)];
const TEST_ROLE_BINDINGS: &[(&str, &str)] = &[("monitor-online", "107"), ("monitor-job", "108"), ("monitor-job-log", "109")];
const TEST_ROLE_IDS: &[&str] = &["monitor-online", "monitor-job", "monitor-job-log"];

#[tokio::test]
async fn system_monitor_migration_groups_menus_and_preserves_role_access() {
    let database = TestDatabase::create().await;
    up(database.pool(), Some(MIGRATIONS_BEFORE_SYSTEM_MONITOR)).await.unwrap();
    insert_test_role_bindings(database.pool()).await;

    up(database.pool(), Some(1)).await.unwrap();

    assert_system_monitor_menu(database.pool()).await;
    assert_menu_relations(database.pool(), MOVED_MENUS).await;
    assert_system_management_menus_unchanged(database.pool()).await;
    assert_parent_bindings(database.pool(), 3).await;

    database.drop().await;
}

#[tokio::test]
async fn system_monitor_down_restores_original_menu_relations() {
    let database = TestDatabase::create().await;
    up(database.pool(), Some(MIGRATIONS_BEFORE_SYSTEM_MONITOR)).await.unwrap();
    insert_test_role_bindings(database.pool()).await;
    up(database.pool(), Some(1)).await.unwrap();
    assert_system_monitor_menu(database.pool()).await;
    assert_menu_relations(database.pool(), MOVED_MENUS).await;
    assert_parent_bindings(database.pool(), 3).await;

    down(database.pool(), Some(1)).await.unwrap();

    assert_menu_relations(database.pool(), RESTORED_MENUS).await;
    assert_eq!(menu_count(database.pool(), SYSTEM_MONITOR_MENU_ID).await, 0);
    assert_parent_bindings(database.pool(), 0).await;
    assert_child_bindings_preserved(database.pool()).await;

    database.drop().await;
}

async fn insert_test_role_bindings(pool: &PgPool) {
    for (index, (role_id, menu_id)) in TEST_ROLE_BINDINGS.iter().enumerate() {
        query("INSERT INTO sys_role (role_id,role_name,role_key,role_sort,status,create_time) VALUES ($1,$1,$1,$2,'0',CURRENT_TIMESTAMP)")
            .bind(role_id)
            .bind(index as i64 + 10)
            .execute(pool)
            .await
            .unwrap();
        query("INSERT INTO sys_role_menu (role_id,menu_id) VALUES ($1,$2)")
            .bind(role_id)
            .bind(menu_id)
            .execute(pool)
            .await
            .unwrap();
    }
}

async fn assert_system_monitor_menu(pool: &PgPool) {
    let menu: (String, String, i64, String, String, String) =
        query_as("SELECT menu_name,parent_id,order_num,path,menu_type,icon FROM sys_menu WHERE menu_id=$1")
            .bind(SYSTEM_MONITOR_MENU_ID)
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(
        menu,
        ("系统监控".into(), "0".into(), 2, "/dashboard/monitor".into(), "M".into(), "icon.monitor".into())
    );
}

async fn assert_menu_relations(pool: &PgPool, expected: &[(&str, &str, i64)]) {
    for (menu_id, parent_id, order_num) in expected {
        let actual: (String, i64) = query_as("SELECT parent_id,order_num FROM sys_menu WHERE menu_id=$1")
            .bind(menu_id)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(actual, ((*parent_id).into(), *order_num), "unexpected relation for menu {menu_id}");
    }
}

async fn assert_system_management_menus_unchanged(pool: &PgPool) {
    for (menu_id, order_num) in SYSTEM_MANAGEMENT_MENUS {
        let actual: (String, i64) = query_as("SELECT parent_id,order_num FROM sys_menu WHERE menu_id=$1")
            .bind(menu_id)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(actual, ("1".into(), *order_num), "system management menu {menu_id} moved unexpectedly");
    }
}

async fn assert_parent_bindings(pool: &PgPool, expected: i64) {
    let count: i64 = query_scalar("SELECT COUNT(*) FROM sys_role_menu WHERE menu_id=$1 AND role_id = ANY($2)")
        .bind(SYSTEM_MONITOR_MENU_ID)
        .bind(TEST_ROLE_IDS)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, expected);
}

async fn assert_child_bindings_preserved(pool: &PgPool) {
    for (role_id, menu_id) in TEST_ROLE_BINDINGS {
        let count: i64 = query_scalar("SELECT COUNT(*) FROM sys_role_menu WHERE role_id=$1 AND menu_id=$2")
            .bind(role_id)
            .bind(menu_id)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "child binding should survive for role {role_id}");
    }
}

async fn menu_count(pool: &PgPool, menu_id: &str) -> i64 {
    query_scalar("SELECT COUNT(*) FROM sys_menu WHERE menu_id=$1")
        .bind(menu_id)
        .fetch_one(pool)
        .await
        .unwrap()
}
