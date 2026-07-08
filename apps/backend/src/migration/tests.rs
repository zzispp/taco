use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use sqlx::{AssertSqlSafe, PgPool, postgres::PgPoolOptions, query, query_scalar};

mod seed_assertions;

use seed_assertions::assert_seed_data_exists;

use super::{down, ensure_runtime_schema_ready, fresh, prepare_runtime_schema, refresh, reset, status, up};

const TEST_DB_ADMIN_URL: &str = "postgres://postgres:123456@localhost:5433/postgres";
const TEST_DB_URL_PREFIX: &str = "postgres://postgres:123456@localhost:5433";
const MIGRATION_TOTAL: usize = 11;
const USERS_TABLE_REGCLASS: &str = "public.sys_user";

static NEXT_TEST_DB_ID: AtomicU64 = AtomicU64::new(0);

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
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
    assert_status_counts(pool, MIGRATION_TOTAL - 1, 1).await;
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
    assert_seed_data_exists(pool).await;

    database.drop().await;
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn runtime_schema_readiness_fails_when_migrations_are_pending() {
    let database = TestDatabase::create().await;

    let error = ensure_runtime_schema_ready(database.pool()).await.unwrap_err();

    assert!(error.to_string().contains("pending migrations"), "unexpected error: {error}");

    database.drop().await;
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn runtime_schema_preparation_auto_migrates_fresh_database() {
    let database = TestDatabase::create().await;

    prepare_runtime_schema(database.pool(), true).await.unwrap();

    assert_status_counts(database.pool(), MIGRATION_TOTAL, 0).await;
    assert!(users_table_exists(database.pool()).await);
    assert_seed_data_exists(database.pool()).await;

    database.drop().await;
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn runtime_schema_preparation_is_idempotent_after_migration() {
    let database = TestDatabase::create().await;

    prepare_runtime_schema(database.pool(), true).await.unwrap();
    prepare_runtime_schema(database.pool(), true).await.unwrap();

    assert_status_counts(database.pool(), MIGRATION_TOTAL, 0).await;
    assert_eq!(applied_migration_count(database.pool()).await, MIGRATION_TOTAL as i64);

    database.drop().await;
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn runtime_schema_preparation_does_not_auto_migrate_when_disabled() {
    let database = TestDatabase::create().await;

    let error = prepare_runtime_schema(database.pool(), false).await.unwrap_err();

    assert!(error.to_string().contains("pending migrations"), "unexpected error: {error}");
    assert_status_counts(database.pool(), 0, MIGRATION_TOTAL).await;

    database.drop().await;
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn runtime_schema_preparation_fails_when_migration_is_dirty() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    insert_dirty_migration(database.pool()).await;

    let auto_error = prepare_runtime_schema(database.pool(), true).await.unwrap_err();
    let manual_error = prepare_runtime_schema(database.pool(), false).await.unwrap_err();

    assert!(auto_error.to_string().contains("dirty at migration 999"), "unexpected error: {auto_error}");
    assert!(manual_error.to_string().contains("dirty at migration 999"), "unexpected error: {manual_error}");

    database.drop().await;
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn runtime_schema_preparation_fails_on_checksum_mismatch() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    query("UPDATE _sqlx_migrations SET checksum = decode('00', 'hex') WHERE version = 20260508000001")
        .execute(database.pool())
        .await
        .unwrap();

    let error = prepare_runtime_schema(database.pool(), true).await.unwrap_err();

    assert!(
        error.to_string().contains("checksum mismatch at version 20260508000001"),
        "unexpected error: {error}"
    );

    database.drop().await;
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn runtime_schema_preparation_fails_when_applied_migration_file_is_missing() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    insert_missing_local_migration(database.pool()).await;

    let error = prepare_runtime_schema(database.pool(), true).await.unwrap_err();

    assert!(error.to_string().contains("applied migration 99999999999999"), "unexpected error: {error}");

    database.drop().await;
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn runtime_schema_readiness_fails_when_a_managed_table_is_missing() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    query("DROP TABLE sys_role CASCADE").execute(database.pool()).await.unwrap();

    let error = ensure_runtime_schema_ready(database.pool()).await.unwrap_err();

    assert!(error.to_string().contains("missing managed tables [sys_role]"), "unexpected error: {error}");

    database.drop().await;
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
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

        query(AssertSqlSafe(format!(r#"CREATE DATABASE "{name}""#))).execute(&admin_pool).await.unwrap();

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
        query(AssertSqlSafe(format!(r#"DROP DATABASE IF EXISTS "{}""#, self.name)))
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

async fn applied_migration_count(pool: &PgPool) -> i64 {
    query_scalar::<_, i64>("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = TRUE")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn insert_dirty_migration(pool: &PgPool) {
    query("INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (999, 'dirty_test', FALSE, decode('00', 'hex'), 0)")
        .execute(pool)
        .await
        .unwrap();
}

async fn insert_missing_local_migration(pool: &PgPool) {
    query("INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (99999999999999, 'missing_local_test', TRUE, decode('00', 'hex'), 0)")
        .execute(pool)
        .await
        .unwrap();
}

fn test_database_name() -> String {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
    let sequence = NEXT_TEST_DB_ID.fetch_add(1, Ordering::Relaxed);
    format!("hook_migration_test_{}_{}_{}", std::process::id(), timestamp, sequence)
}
