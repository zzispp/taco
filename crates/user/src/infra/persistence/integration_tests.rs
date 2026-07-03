use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::{Executor, PgPool, postgres::PgPoolOptions, query};
use storage::{Database, connect_database};
use types::pagination::PageSliceRequest;

use super::{UserRecordInput, UserStore};

const TEST_DB_ADMIN_URL: &str = "postgres://postgres:123456@localhost:5433/postgres";
const TEST_DB_URL_PREFIX: &str = "postgres://postgres:123456@localhost:5433";

static NEXT_TEST_DB_ID: AtomicU64 = AtomicU64::new(0);

#[tokio::test]
async fn user_soft_delete_is_filtered_from_list_and_lookup() {
    let database = TestDatabase::create().await;
    let users = UserStore::new(database.database().clone());

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

    database.drop().await;
}

struct TestDatabase {
    admin_pool: PgPool,
    database: Database,
    name: String,
}

impl TestDatabase {
    async fn create() -> Self {
        let admin_pool = PgPoolOptions::new().max_connections(1).connect(TEST_DB_ADMIN_URL).await.unwrap();
        let name = test_database_name();

        query(&format!(r#"CREATE DATABASE "{name}""#)).execute(&admin_pool).await.unwrap();

        let database = connect_database(&format!("{TEST_DB_URL_PREFIX}/{name}")).await.unwrap();
        for sql in migration_sqls() {
            database.pool().execute(sql).await.unwrap();
        }

        Self { admin_pool, database, name }
    }

    fn database(&self) -> &Database {
        &self.database
    }

    async fn drop(self) {
        self.database.pool().close().await;
        query("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = $1 AND pid <> pg_backend_pid()")
            .bind(&self.name)
            .execute(&self.admin_pool)
            .await
            .unwrap();
        query(&format!(r#"DROP DATABASE IF EXISTS "{}""#, self.name))
            .execute(&self.admin_pool)
            .await
            .unwrap();
        self.admin_pool.close().await;
    }
}

fn migration_sqls() -> [&'static str; 2] {
    [
        include_str!("../../../../../migrations/20260508000001_baseline.up.sql"),
        include_str!("../../../../../migrations/20260508000002_add_rbac_timestamps.up.sql"),
    ]
}

fn test_database_name() -> String {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
    let sequence = NEXT_TEST_DB_ID.fetch_add(1, Ordering::Relaxed);
    format!("hook_user_test_{}_{}_{}", std::process::id(), timestamp, sequence)
}
