use sqlx::{Connection, Executor, PgConnection, query};
use storage::{Database, connect_database};

use super::{ApiPermissionRecordInput, RbacStore, RoleApiBindingRecordInput, RoleRecordInput};

const TEST_DB_URL: &str = "postgres://postgres:123456@localhost:5433/postgres";
const TEST_DB_LOCK_ID: i64 = 5_406_001;

#[tokio::test]
async fn replace_role_apis_replaces_bindings_transactionally() {
    let _guard = test_lock().await;
    let database = reset_test_db().await;
    let rbac = RbacStore::new(database);

    let api = rbac
        .create_api(ApiPermissionRecordInput {
            code: "test_api".into(),
            method: "GET".into(),
            path_pattern: "/api/test".into(),
            name: "Test".into(),
            group: "Tests".into(),
            enabled: true,
            system: false,
        })
        .await
        .unwrap();

    rbac.create_role(RoleRecordInput {
        code: "tester".into(),
        name: "Tester".into(),
        description: "tester".into(),
        enabled: true,
        system: false,
        sort_order: 1,
    })
    .await
    .unwrap();

    rbac.replace_role_apis(
        "tester",
        vec![RoleApiBindingRecordInput {
            role_code: "tester".into(),
            api_permission_id: api.id.clone(),
        }],
    )
    .await
    .unwrap();

    assert_eq!(rbac.role_api_ids("tester").await.unwrap(), vec![api.id.clone()]);

    rbac.replace_role_apis("tester", vec![]).await.unwrap();

    assert!(rbac.role_api_ids("tester").await.unwrap().is_empty());
    assert!(!rbac.role_has_api_bindings("tester").await.unwrap());
}

async fn reset_test_db() -> Database {
    let database = connect_database(TEST_DB_URL).await.unwrap();
    for table in managed_tables() {
        database.pool().execute(format!("DROP TABLE IF EXISTS {table} CASCADE").as_str()).await.unwrap();
    }
    database.pool().execute("DROP TABLE IF EXISTS _sqlx_migrations").await.unwrap();
    for sql in migration_sqls() {
        database.pool().execute(sql).await.unwrap();
    }
    database
}

async fn test_lock() -> PgConnection {
    let mut connection = PgConnection::connect(TEST_DB_URL).await.unwrap();
    query("SELECT pg_advisory_lock($1)")
        .bind(TEST_DB_LOCK_ID)
        .execute(&mut connection)
        .await
        .unwrap();
    connection
}

fn managed_tables() -> [&'static str; 7] {
    [
        "role_menu_permissions",
        "role_api_permissions",
        "menu_items",
        "menu_sections",
        "api_permissions",
        "roles",
        "users",
    ]
}

fn migration_sqls() -> [&'static str; 2] {
    [
        include_str!("../../../../../migrations/20260508000001_baseline.up.sql"),
        include_str!("../../../../../migrations/20260508000002_add_rbac_timestamps.up.sql"),
    ]
}
