use sqlx::{Connection, Executor, PgConnection, query};
use storage::{Database, connect_database};
use types::pagination::PageSliceRequest;

use super::{UserRecordInput, UserStore};

const TEST_DB_URL: &str = "postgres://postgres:123456@localhost:5433/postgres";
const TEST_DB_LOCK_ID: i64 = 5_406_001;

#[tokio::test]
async fn user_soft_delete_is_filtered_from_list_and_lookup() {
    let _guard = test_lock().await;
    let database = reset_test_db().await;
    let users = UserStore::new(database);

    let created = users
        .create(UserRecordInput {
            username: "alice".into(),
            password_hash: "hashed".into(),
            email: "alice@example.com".into(),
            role: "user".into(),
            is_active: true,
        })
        .await
        .unwrap();

    users.delete(created.id.clone()).await.unwrap();

    assert!(users.find_by_id(created.id.clone()).await.unwrap().is_none());
    assert!(users.find_by_email("alice@example.com").await.unwrap().is_none());

    let page = users
        .list_slice(PageSliceRequest {
            offset: 0,
            limit: 20,
            page: 1,
            page_size: 20,
        })
        .await
        .unwrap();

    assert_eq!(page.total, 0);
    assert!(page.items.is_empty());
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
