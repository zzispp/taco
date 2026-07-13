use sqlx::{PgPool, query, query_as, query_scalar};

use super::{TestDatabase, down, up};

const MIGRATIONS_BEFORE_OVERVIEW: u32 = 16;
const OVERVIEW_MENU_ID: &str = "4";
const DASHBOARD_MENU_ID: &str = "2";
const TEST_ROLE_ID: &str = "overview-dashboard";

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn overview_migration_groups_dashboard_and_preserves_role_access() {
    let database = TestDatabase::create().await;
    up(database.pool(), Some(MIGRATIONS_BEFORE_OVERVIEW)).await.unwrap();
    insert_test_dashboard_binding(database.pool()).await;

    up(database.pool(), Some(1)).await.unwrap();

    assert_overview_menu(database.pool()).await;
    assert_dashboard_relation(database.pool(), "4", 1).await;
    assert_parent_binding(database.pool(), 1).await;
    assert_other_directories_unchanged(database.pool()).await;

    database.drop().await;
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn overview_down_restores_root_dashboard_and_role_bindings() {
    let database = TestDatabase::create().await;
    up(database.pool(), Some(MIGRATIONS_BEFORE_OVERVIEW)).await.unwrap();
    insert_test_dashboard_binding(database.pool()).await;
    up(database.pool(), Some(1)).await.unwrap();

    down(database.pool(), Some(1)).await.unwrap();

    assert_dashboard_relation(database.pool(), "0", 0).await;
    assert_eq!(menu_count(database.pool(), OVERVIEW_MENU_ID).await, 0);
    assert_parent_binding(database.pool(), 0).await;
    assert_dashboard_binding_preserved(database.pool()).await;

    database.drop().await;
}

async fn insert_test_dashboard_binding(pool: &PgPool) {
    query("INSERT INTO sys_role (role_id,role_name,role_key,role_sort,status,create_time) VALUES ($1,$1,$1,10,'0',CURRENT_TIMESTAMP)")
        .bind(TEST_ROLE_ID)
        .execute(pool)
        .await
        .unwrap();
    query("INSERT INTO sys_role_menu (role_id,menu_id) VALUES ($1,$2)")
        .bind(TEST_ROLE_ID)
        .bind(DASHBOARD_MENU_ID)
        .execute(pool)
        .await
        .unwrap();
}

async fn assert_overview_menu(pool: &PgPool) {
    let menu: (String, String, i64, String, String, String) =
        query_as("SELECT menu_name,parent_id,order_num,path,menu_type,icon FROM sys_menu WHERE menu_id=$1")
            .bind(OVERVIEW_MENU_ID)
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(
        menu,
        ("概览".into(), "0".into(), 0, "/dashboard/overview".into(), "M".into(), "icon.dashboard".into())
    );
}

async fn assert_dashboard_relation(pool: &PgPool, parent_id: &str, order_num: i64) {
    let relation: (String, i64) = query_as("SELECT parent_id,order_num FROM sys_menu WHERE menu_id=$1")
        .bind(DASHBOARD_MENU_ID)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(relation, (parent_id.into(), order_num));
}

async fn assert_parent_binding(pool: &PgPool, expected: i64) {
    let count: i64 = query_scalar("SELECT COUNT(*) FROM sys_role_menu WHERE role_id=$1 AND menu_id=$2")
        .bind(TEST_ROLE_ID)
        .bind(OVERVIEW_MENU_ID)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, expected);
}

async fn assert_other_directories_unchanged(pool: &PgPool) {
    let directories: Vec<(String, i64)> = query_as("SELECT menu_id,order_num FROM sys_menu WHERE menu_id IN ('1','3') AND parent_id='0' ORDER BY menu_id")
        .fetch_all(pool)
        .await
        .unwrap();
    assert_eq!(directories, vec![("1".into(), 1), ("3".into(), 2)]);
}

async fn assert_dashboard_binding_preserved(pool: &PgPool) {
    let count: i64 = query_scalar("SELECT COUNT(*) FROM sys_role_menu WHERE role_id=$1 AND menu_id=$2")
        .bind(TEST_ROLE_ID)
        .bind(DASHBOARD_MENU_ID)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

async fn menu_count(pool: &PgPool, menu_id: &str) -> i64 {
    query_scalar("SELECT COUNT(*) FROM sys_menu WHERE menu_id=$1")
        .bind(menu_id)
        .fetch_one(pool)
        .await
        .unwrap()
}
