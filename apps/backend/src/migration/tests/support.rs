use std::{
    ffi::OsString,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use configuration::Settings;
use sqlx::{AssertSqlSafe, PgPool, postgres::PgPoolOptions, query, query_scalar};
use storage::Database;
use url::Url;
use user::{
    application::{BootstrapAdministratorOutcome, BootstrapAdministratorRecord, BootstrapAdministratorRepository},
    infra::StorageUserRepository,
};

use super::{down, status, up};

const ADMIN_DATABASE_NAME: &str = "postgres";
const MUTATION_TEST_ADMINISTRATOR_USERNAME: &str = "migration-mutation-admin";
const MUTATION_TEST_ADMINISTRATOR_EMAIL: &str = "migration-mutation-admin@test.invalid";
const MUTATION_TEST_ADMINISTRATOR_PASSWORD_HASH: &str = "migration-mutation-admin-password-hash";

static NEXT_TEST_DB_ID: AtomicU64 = AtomicU64::new(0);

pub(super) struct TestDatabase {
    admin_pool: PgPool,
    pool: PgPool,
    database_url: Url,
    name: String,
}

impl TestDatabase {
    pub(super) async fn create() -> Self {
        let configured_url = configured_database();
        let admin_url = database_url_for(&configured_url, ADMIN_DATABASE_NAME);
        let admin_pool = PgPoolOptions::new().max_connections(1).connect(admin_url.as_str()).await.unwrap();
        let name = test_database_name();
        query(AssertSqlSafe(format!(r#"CREATE DATABASE "{name}""#))).execute(&admin_pool).await.unwrap();
        let database_url = database_url_for(&configured_url, &name);
        let pool = PgPoolOptions::new().max_connections(5).connect(database_url.as_str()).await.unwrap();
        Self {
            admin_pool,
            pool,
            database_url,
            name,
        }
    }

    pub(super) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(super) fn database_url(&self) -> String {
        self.database_url.to_string()
    }

    pub(super) async fn drop(self) {
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

fn configured_database() -> Url {
    let database_url = load_local_settings()
        .database_url()
        .unwrap_or_else(|error| panic!("test configuration database connection is invalid: {error}"));
    let parsed = Url::parse(&database_url).unwrap_or_else(|error| panic!("test configuration database connection is not a valid URL: {error}"));
    assert!(
        matches!(parsed.scheme(), "postgres" | "postgresql"),
        "test configuration database connection must use PostgreSQL"
    );
    parsed
}

fn load_local_settings() -> Settings {
    let path = local_configuration_path();
    Settings::load_from_args(vec![OsString::from("taco"), OsString::from("--config"), path.clone().into_os_string()])
        .unwrap_or_else(|error| panic!("failed to load local test configuration {}: {error}", path.display()))
}

fn local_configuration_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../config/config.yaml")
}

fn database_url_for(configured_url: &Url, database_name: &str) -> Url {
    let mut database_url = configured_url.clone();
    database_url.set_path(database_name);
    database_url.set_fragment(None);
    database_url
}

pub(super) async fn managed_table_exists(pool: &PgPool, table: &str) -> bool {
    query_scalar::<_, bool>("SELECT to_regclass($1) IS NOT NULL")
        .bind(table)
        .fetch_one(pool)
        .await
        .unwrap()
}

pub(super) async fn bootstrap_system_administrator(pool: &PgPool) {
    let repository = StorageUserRepository::new(Database::new(pool.clone()));
    let outcome = repository
        .create_system_administrator_if_absent(BootstrapAdministratorRecord {
            username: MUTATION_TEST_ADMINISTRATOR_USERNAME.into(),
            nick_name: MUTATION_TEST_ADMINISTRATOR_USERNAME.into(),
            email: MUTATION_TEST_ADMINISTRATOR_EMAIL.into(),
            password_hash: MUTATION_TEST_ADMINISTRATOR_PASSWORD_HASH.into(),
        })
        .await
        .unwrap();

    assert_eq!(outcome, BootstrapAdministratorOutcome::Created);
    assert!(repository.has_enabled_system_administrator().await.unwrap());
}

pub(super) async fn migrate_through(pool: &PgPool, target_version: i64) {
    let rows = status(pool).await.unwrap();
    assert!(rows.iter().all(|row| row.kind == "pending"), "migrate_through requires a clean migration state");

    let target_index = rows
        .iter()
        .position(|row| row.version == target_version)
        .unwrap_or_else(|| panic!("target migration {target_version} is not present in the local migration source"));
    let steps = u32::try_from(target_index + 1).unwrap();
    up(pool, Some(steps)).await.unwrap();

    assert_migrated_through(pool, target_version).await;
}

pub(super) async fn rollback_from(pool: &PgPool, target_version: i64) {
    let latest_applied: Option<i64> = query_scalar("SELECT MAX(version) FROM _sqlx_migrations WHERE success = TRUE")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(
        latest_applied,
        Some(target_version),
        "rollback_from requires the target migration to be the latest applied migration"
    );

    down(pool, Some(1)).await.unwrap();

    let target_applied: i64 = query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE version = $1 AND success = TRUE")
        .bind(target_version)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(target_applied, 0, "target migration {target_version} remained applied after rollback");
}

async fn assert_migrated_through(pool: &PgPool, target_version: i64) {
    for row in status(pool).await.unwrap() {
        let expected = if row.version <= target_version { "applied" } else { "pending" };
        assert_eq!(row.kind, expected, "unexpected migration state for version {}", row.version);
    }
}

fn test_database_name() -> String {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros();
    let sequence = NEXT_TEST_DB_ID.fetch_add(1, Ordering::Relaxed);
    format!("taco_migration_test_{}_{}_{}", std::process::id(), timestamp, sequence)
}

#[test]
fn database_url_replaces_only_database_path() {
    let configured_url = Url::parse("postgres://db.example.invalid:5432/original?sslmode=require").unwrap();

    let database_url = database_url_for(&configured_url, "dynamic_test_database");

    assert_eq!(
        database_url.as_str(),
        "postgres://db.example.invalid:5432/dynamic_test_database?sslmode=require"
    );
}
