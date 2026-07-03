use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::{PgPool, postgres::PgPoolOptions, query, query_scalar};

use super::{down, ensure_runtime_schema_ready, fresh, refresh, reset, status, up};

const TEST_DB_ADMIN_URL: &str = "postgres://postgres:123456@localhost:5433/postgres";
const TEST_DB_URL_PREFIX: &str = "postgres://postgres:123456@localhost:5433";
const MIGRATION_TOTAL: usize = 2;
const USERS_TABLE_REGCLASS: &str = "public.users";

static NEXT_TEST_DB_ID: AtomicU64 = AtomicU64::new(0);

#[tokio::test]
async fn migrations_support_full_up_down_cycle() {
    let database = TestDatabase::create().await;
    let pool = database.pool();

    assert_status_counts(pool, 0, MIGRATION_TOTAL).await;

    up(pool, Some(1)).await.unwrap();
    assert_status_counts(pool, 1, MIGRATION_TOTAL - 1).await;
    assert!(users_table_exists(pool).await);

    up(pool, None).await.unwrap();
    assert_status_counts(pool, MIGRATION_TOTAL, 0).await;
    assert!(users_table_exists(pool).await);

    down(pool, Some(1)).await.unwrap();
    assert_status_counts(pool, 1, MIGRATION_TOTAL - 1).await;
    assert!(users_table_exists(pool).await);

    refresh(pool).await.unwrap();
    assert_status_counts(pool, MIGRATION_TOTAL, 0).await;
    assert!(users_table_exists(pool).await);

    reset(pool).await.unwrap();
    assert_status_counts(pool, 0, MIGRATION_TOTAL).await;
    assert!(!users_table_exists(pool).await);

    fresh(pool).await.unwrap();
    assert_status_counts(pool, MIGRATION_TOTAL, 0).await;
    assert!(users_table_exists(pool).await);

    database.drop().await;
}

#[tokio::test]
async fn runtime_schema_readiness_fails_when_migrations_are_pending() {
    let database = TestDatabase::create().await;

    let error = ensure_runtime_schema_ready(database.pool()).await.unwrap_err();

    assert!(error.to_string().contains("pending migrations"), "unexpected error: {error}");

    database.drop().await;
}

#[tokio::test]
async fn runtime_schema_readiness_fails_when_a_managed_table_is_missing() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    query("DROP TABLE roles").execute(database.pool()).await.unwrap();

    let error = ensure_runtime_schema_ready(database.pool()).await.unwrap_err();

    assert!(error.to_string().contains("missing managed tables [roles]"), "unexpected error: {error}");

    database.drop().await;
}

#[tokio::test]
async fn runtime_schema_readiness_passes_after_all_migrations() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();

    ensure_runtime_schema_ready(database.pool()).await.unwrap();

    database.drop().await;
}

struct TestDatabase {
    admin_pool: PgPool,
    pool: PgPool,
    name: String,
}

impl TestDatabase {
    async fn create() -> Self {
        let admin_pool = PgPoolOptions::new().max_connections(1).connect(TEST_DB_ADMIN_URL).await.unwrap();
        let name = test_database_name();

        query(&format!(r#"CREATE DATABASE "{name}""#)).execute(&admin_pool).await.unwrap();

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&format!("{TEST_DB_URL_PREFIX}/{name}"))
            .await
            .unwrap();

        Self { admin_pool, pool, name }
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }

    async fn drop(self) {
        self.pool.close().await;
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

async fn assert_status_counts(pool: &PgPool, applied: usize, pending: usize) {
    let rows = status(pool).await.unwrap();
    assert_eq!(rows.iter().filter(|row| row.kind == "applied").count(), applied);
    assert_eq!(rows.iter().filter(|row| row.kind == "pending").count(), pending);
}

async fn users_table_exists(pool: &PgPool) -> bool {
    query_scalar::<_, bool>("SELECT to_regclass($1) IS NOT NULL")
        .bind(USERS_TABLE_REGCLASS)
        .fetch_one(pool)
        .await
        .unwrap()
}

fn test_database_name() -> String {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
    let sequence = NEXT_TEST_DB_ID.fetch_add(1, Ordering::Relaxed);
    format!("hook_migration_test_{}_{}_{}", std::process::id(), timestamp, sequence)
}
