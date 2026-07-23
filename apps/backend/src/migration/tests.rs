use sqlx::{PgPool, query};

mod audit_logs;
mod audit_logs_rollback;
mod audit_repository;
mod bootstrap_administrator;
mod data_integrity;
mod export_snapshots;
mod file_management;
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
mod system_log_audited_delete;
mod system_log_repository;
mod system_log_search_plan;
mod system_logs;
mod system_monitor_menu;
mod tracing_config_listener;
mod user_password_contract;
mod user_sessions;
use seed_assertions::assert_seed_data_exists;
use support::{TestDatabase, bootstrap_system_administrator, managed_table_exists, migrate_through, rollback_from};

use super::{down, ensure_runtime_schema_ready, fresh, status, up};

const MIGRATION_TOTAL: usize = 32;
const FORWARD_ONLY_MIGRATION_VERSION: i64 = 20260717000007;
const USERS_TABLE_REGCLASS: &str = "public.sys_user";

#[tokio::test]
async fn migrations_support_forward_application_and_fresh_rebuild() {
    let database = TestDatabase::create().await;
    let pool = database.pool();

    assert_status_counts(pool, 0, MIGRATION_TOTAL).await;

    up(pool, Some(1)).await.unwrap();
    assert_status_counts(pool, 1, MIGRATION_TOTAL - 1).await;
    assert!(users_table_exists(pool).await);

    up(pool, None).await.unwrap();
    assert_status_counts(pool, MIGRATION_TOTAL, 0).await;
    assert!(users_table_exists(pool).await);

    fresh(pool).await.unwrap();
    assert_status_counts(pool, MIGRATION_TOTAL, 0).await;
    assert!(users_table_exists(pool).await);
    assert_seed_data_exists(pool).await;

    database.drop().await;
}

#[tokio::test]
async fn rollback_rejects_forward_only_migrations_instead_of_skipping_them() {
    let database = TestDatabase::create().await;
    migrate_through(database.pool(), FORWARD_ONLY_MIGRATION_VERSION).await;

    let error = down(database.pool(), Some(1)).await.unwrap_err();

    assert!(
        error.to_string().contains("cannot roll back through forward-only migration 20260717000007"),
        "unexpected error: {error}"
    );
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
    assert!(managed_table_exists(pool, "file_entry").await);
    assert!(managed_table_exists(pool, "file_upload_session").await);
    assert!(managed_table_exists(pool, "file_provider_cleanup").await);

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
async fn runtime_schema_readiness_is_idempotent_after_explicit_migration() {
    let database = TestDatabase::create().await;

    up(database.pool(), None).await.unwrap();
    ensure_runtime_schema_ready(database.pool()).await.unwrap();
    ensure_runtime_schema_ready(database.pool()).await.unwrap();

    assert_status_counts(database.pool(), MIGRATION_TOTAL, 0).await;

    database.drop().await;
}

#[tokio::test]
async fn runtime_schema_readiness_fails_when_migration_is_dirty() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    insert_dirty_migration(database.pool()).await;

    let error = ensure_runtime_schema_ready(database.pool()).await.unwrap_err();

    assert!(error.to_string().contains("dirty at migration 999"), "unexpected error: {error}");

    database.drop().await;
}

#[tokio::test]
async fn runtime_schema_readiness_fails_on_checksum_mismatch() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    query("UPDATE _sqlx_migrations SET checksum = decode('00', 'hex') WHERE version = 20260508000001")
        .execute(database.pool())
        .await
        .unwrap();

    let error = ensure_runtime_schema_ready(database.pool()).await.unwrap_err();

    assert!(
        error.to_string().contains("checksum mismatch at version 20260508000001"),
        "unexpected error: {error}"
    );

    database.drop().await;
}

#[tokio::test]
async fn runtime_schema_readiness_fails_when_applied_migration_file_is_missing() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    insert_missing_local_migration(database.pool()).await;

    let error = ensure_runtime_schema_ready(database.pool()).await.unwrap_err();

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
