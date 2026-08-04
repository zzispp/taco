use std::borrow::Cow;

use sqlx::{
    AssertSqlSafe, PgPool, Row, SqlSafeStr,
    migrate::{Migration, MigrationType, Migrator},
    query, query_scalar,
};

#[path = "file_management_online_indexes/fixture.rs"]
mod fixture;

use super::{TestDatabase, down, migrate_through, up};

const ONLINE_INDEX_MIGRATION_VERSIONS: &[i64] = &[20260804000002, 20260804000003, 20260804000004, 20260804000005, 20260804000006, 20260804000007];
const FILE_MANAGEMENT_BEFORE_ONLINE_INDEXES_VERSION: i64 = 20260804000001;
const FAILED_TEST_MIGRATION_VERSION: i64 = 99999999999998;
const ONLINE_INDEXES: &[(&str, &str, &str)] = &[
    ("idx_file_entry_parent_id", "file_entry", "parent_id"),
    ("idx_file_entry_object_id", "file_entry", "object_id"),
    ("idx_file_upload_session_parent_id", "file_upload_session", "parent_id"),
    ("idx_file_upload_session_result_entry_id", "file_upload_session", "result_entry_id"),
    ("idx_sys_user_avatar_file_id", "sys_user", "avatar_file_id"),
    ("idx_file_entry_tag_tag_id", "file_entry_tag", "tag_id"),
];

#[tokio::test]
async fn online_file_indexes_run_without_a_transaction_and_record_success() {
    let database = TestDatabase::create().await;
    for version in ONLINE_INDEX_MIGRATION_VERSIONS {
        let migration = super::super::migrator()
            .iter()
            .find(|migration| migration.version == *version)
            .expect("online index migration is embedded");
        assert!(migration.no_tx, "migration {version} must run without a transaction");
    }

    let latest_version = *ONLINE_INDEX_MIGRATION_VERSIONS.last().expect("online index migrations are non-empty");
    migrate_through(database.pool(), latest_version).await;

    for (index, table, column) in ONLINE_INDEXES {
        assert_index_valid(database.pool(), index, table, column).await;
    }
    for version in ONLINE_INDEX_MIGRATION_VERSIONS {
        assert_migration_recorded(database.pool(), *version).await;
    }

    down(database.pool(), Some(ONLINE_INDEX_MIGRATION_VERSIONS.len() as u32)).await.unwrap();
    for (index, table, _) in ONLINE_INDEXES {
        assert_index_absent(database.pool(), index, table).await;
    }
    database.drop().await;
}

#[tokio::test]
async fn online_file_indexes_upgrade_existing_data_without_changing_reference_integrity() {
    let database = TestDatabase::create().await;
    let pool = database.pool();
    migrate_through(pool, FILE_MANAGEMENT_BEFORE_ONLINE_INDEXES_VERSION).await;
    fixture::seed_existing_file_management_data(pool).await;
    fixture::assert_existing_file_management_data(pool).await;

    up(pool, Some(ONLINE_INDEX_MIGRATION_VERSIONS.len() as u32)).await.unwrap();

    for (index, table, column) in ONLINE_INDEXES {
        assert_index_valid(pool, index, table, column).await;
    }
    fixture::assert_existing_file_management_data(pool).await;
    fixture::assert_reference_integrity(pool).await;
    database.drop().await;
}

#[tokio::test]
async fn failed_non_transaction_migration_exposes_error_without_a_success_record() {
    let database = TestDatabase::create().await;
    let migration = Migration::new(
        FAILED_TEST_MIGRATION_VERSION,
        Cow::Borrowed("forced online index failure"),
        MigrationType::Simple,
        AssertSqlSafe("CREATE INDEX CONCURRENTLY idx_forced_failure ON public.table_that_does_not_exist (id);").into_sql_str(),
        true,
    );
    let migrator = Migrator::with_migrations(vec![migration]);

    let error = super::super::run_migrator(database.pool(), &migrator).await.unwrap_err();

    assert!(error.to_string().contains("table_that_does_not_exist"), "unexpected error: {error}");
    assert_migration_record_absent(database.pool(), FAILED_TEST_MIGRATION_VERSION).await;
    database.drop().await;
}

async fn assert_index_valid(pool: &PgPool, index: &str, table: &str, column: &str) {
    let row: (bool, bool, String) = query_as_index(pool, index, table).await;
    assert!(row.0 && row.1, "index {index} is not ready and valid");
    assert!(row.2.contains(table), "index {index} is attached to an unexpected table: {}", row.2);
    assert!(row.2.contains(&format!("({column})")), "index {index} does not lead with {column}: {}", row.2);
}

async fn assert_index_absent(pool: &PgPool, index: &str, table: &str) {
    let exists: bool = query_scalar("SELECT EXISTS(SELECT 1 FROM pg_indexes WHERE schemaname='public' AND tablename=$1 AND indexname=$2)")
        .bind(table)
        .bind(index)
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(!exists, "index {index} survived rollback");
}

async fn assert_migration_recorded(pool: &PgPool, version: i64) {
    let success: bool = query_scalar("SELECT success FROM _sqlx_migrations WHERE version=$1")
        .bind(version)
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(success, "migration {version} was not recorded as successful");
}

async fn assert_migration_record_absent(pool: &PgPool, version: i64) {
    let count: i64 = query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE version=$1")
        .bind(version)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, 0, "failed migration {version} was recorded");
}

async fn query_as_index(pool: &PgPool, index: &str, table: &str) -> (bool, bool, String) {
    query(
        "SELECT i.indisready, i.indisvalid, pg_get_indexdef(i.indexrelid) \
         FROM pg_index i JOIN pg_class c ON c.oid=i.indexrelid \
         JOIN pg_class t ON t.oid=i.indrelid \
         WHERE c.relnamespace='public'::regnamespace AND c.relname=$1 AND t.relname=$2",
    )
    .bind(index)
    .bind(table)
    .fetch_one(pool)
    .await
    .map(|row| {
        (
            row.get::<bool, _>("indisready"),
            row.get::<bool, _>("indisvalid"),
            row.get::<String, _>("pg_get_indexdef"),
        )
    })
    .unwrap()
}
