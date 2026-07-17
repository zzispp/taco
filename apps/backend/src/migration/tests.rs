use sqlx::{PgPool, query, query_scalar};

mod audit_logs;
mod audit_logs_rollback;
mod audit_repository;
mod bootstrap_admin;
mod data_integrity;
mod export_snapshots;
mod log_menu_hierarchy;
mod notice_repository;
mod notice_rollback;
mod overview_menu;
mod performance_indexes;
mod postgres_observer;
mod scheduler_assertions;
mod scheduler_conflicts;
mod scheduler_execution_detail;
mod scheduler_execution_persistence;
mod scheduler_log_query;
mod scheduler_runtime;
mod scheduler_schema;
mod scheduler_supervisor;
mod seed_assertions;
mod support;
mod system_log_repository;
mod system_log_search_plan;
mod system_logs;
mod system_monitor_menu;
mod tracing_config_listener;
mod user_password_contract;
mod user_sessions;
use seed_assertions::assert_seed_data_exists;
use support::{TestDatabase, managed_table_exists, rollback_from};

use super::{down, ensure_runtime_schema_ready, fresh, prepare_runtime_schema, refresh, reset, status, up};

const MIGRATION_TOTAL: usize = 27;
const USERS_TABLE_REGCLASS: &str = "public.sys_user";

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

#[tokio::test]
async fn fresh_drops_all_managed_tables_when_migration_state_is_missing() {
    let database = TestDatabase::create().await;
    let pool = database.pool();
    up(pool, None).await.unwrap();
    query("DROP TABLE _sqlx_migrations").execute(pool).await.unwrap();

    fresh(pool).await.unwrap();

    assert_status_counts(pool, MIGRATION_TOTAL, 0).await;
    assert!(managed_table_exists(pool, "audit_outbox").await);
    assert!(managed_table_exists(pool, "sys_job").await);
    assert!(managed_table_exists(pool, "sys_job_execution").await);

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
async fn runtime_schema_preparation_auto_migrates_fresh_database() {
    let database = TestDatabase::create().await;

    prepare_runtime_schema(database.pool(), true).await.unwrap();

    assert_status_counts(database.pool(), MIGRATION_TOTAL, 0).await;
    assert!(users_table_exists(database.pool()).await);
    assert_seed_data_exists(database.pool()).await;

    database.drop().await;
}

#[tokio::test]
async fn runtime_schema_preparation_is_idempotent_after_migration() {
    let database = TestDatabase::create().await;

    prepare_runtime_schema(database.pool(), true).await.unwrap();
    prepare_runtime_schema(database.pool(), true).await.unwrap();

    assert_status_counts(database.pool(), MIGRATION_TOTAL, 0).await;
    assert_eq!(applied_migration_count(database.pool()).await, MIGRATION_TOTAL as i64);

    database.drop().await;
}

#[tokio::test]
async fn runtime_schema_preparation_does_not_auto_migrate_when_disabled() {
    let database = TestDatabase::create().await;

    let error = prepare_runtime_schema(database.pool(), false).await.unwrap_err();

    assert!(error.to_string().contains("pending migrations"), "unexpected error: {error}");
    assert_status_counts(database.pool(), 0, MIGRATION_TOTAL).await;

    database.drop().await;
}

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

#[tokio::test]
async fn runtime_schema_preparation_fails_when_applied_migration_file_is_missing() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    insert_missing_local_migration(database.pool()).await;

    let error = prepare_runtime_schema(database.pool(), true).await.unwrap_err();

    assert!(error.to_string().contains("applied migration 99999999999999"), "unexpected error: {error}");

    database.drop().await;
}

#[tokio::test]
async fn runtime_schema_readiness_fails_when_a_managed_table_is_missing() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    query("DROP TABLE sys_role CASCADE").execute(database.pool()).await.unwrap();

    let error = ensure_runtime_schema_ready(database.pool()).await.unwrap_err();

    assert!(error.to_string().contains("missing managed tables [sys_role]"), "unexpected error: {error}");

    database.drop().await;
}

#[tokio::test]
async fn runtime_schema_readiness_passes_after_all_migrations() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();

    ensure_runtime_schema_ready(database.pool()).await.unwrap();

    database.drop().await;
}

async fn assert_status_counts(pool: &PgPool, applied: usize, pending: usize) {
    let rows = status(pool).await.unwrap();
    assert_eq!(rows.iter().filter(|row| row.kind == "applied").count(), applied);
    assert_eq!(rows.iter().filter(|row| row.kind == "pending").count(), pending);
}

async fn users_table_exists(pool: &PgPool) -> bool {
    managed_table_exists(pool, USERS_TABLE_REGCLASS).await
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
