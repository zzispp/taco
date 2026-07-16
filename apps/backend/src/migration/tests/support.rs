use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use configuration::{DatabaseSettings, Settings};
use sqlx::{AssertSqlSafe, PgPool, postgres::PgPoolOptions, query, query_scalar};
use url::Url;

use super::down;

const TEST_CONFIG_ENV: &str = "TACO_TEST_CONFIG";
const CONFIG_ARG: &str = "--config";
const ADMIN_DATABASE_NAME: &str = "postgres";
const TEST_BINARY_NAME: &str = "backend-migration-tests";

static NEXT_TEST_DB_ID: AtomicU64 = AtomicU64::new(0);

pub(super) struct TestDatabase {
    admin_pool: PgPool,
    pool: PgPool,
    database_url: Url,
    database_settings: DatabaseSettings,
    name: String,
}

impl TestDatabase {
    pub(super) async fn create() -> Self {
        let (mut database_settings, configured_url) = configured_database();
        let admin_url = database_url_for(&configured_url, ADMIN_DATABASE_NAME);
        let admin_pool = PgPoolOptions::new().max_connections(1).connect(admin_url.as_str()).await.unwrap();
        let name = test_database_name();
        query(AssertSqlSafe(format!(r#"CREATE DATABASE "{name}""#))).execute(&admin_pool).await.unwrap();
        let database_url = database_url_for(&configured_url, &name);
        let pool = PgPoolOptions::new().max_connections(5).connect(database_url.as_str()).await.unwrap();
        database_settings.name = name.clone();

        Self {
            admin_pool,
            pool,
            database_url,
            database_settings,
            name,
        }
    }

    pub(super) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(super) fn database_url(&self) -> String {
        self.database_url.to_string()
    }

    pub(super) fn database_settings(&self) -> DatabaseSettings {
        self.database_settings.clone()
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

fn configured_database() -> (DatabaseSettings, Url) {
    let config_path = test_config_path();
    let settings = Settings::load_from_args([OsString::from(TEST_BINARY_NAME), OsString::from(CONFIG_ARG), config_path])
        .unwrap_or_else(|error| panic!("failed to load test configuration from {TEST_CONFIG_ENV}: {error}"));
    let database_url = settings
        .database_url()
        .unwrap_or_else(|error| panic!("failed to read database connection from {TEST_CONFIG_ENV}: {error}"));
    let parsed = Url::parse(&database_url).unwrap_or_else(|error| panic!("database connection in {TEST_CONFIG_ENV} is not a valid URL: {error}"));
    assert!(
        matches!(parsed.scheme(), "postgres" | "postgresql"),
        "database connection in {TEST_CONFIG_ENV} must use PostgreSQL"
    );
    (settings.database, parsed)
}

fn test_config_path() -> OsString {
    let configured_path =
        PathBuf::from(std::env::var_os(TEST_CONFIG_ENV).unwrap_or_else(|| panic!("{TEST_CONFIG_ENV} must point to a typed YAML configuration file")));
    if configured_path.is_absolute() {
        return configured_path.into_os_string();
    }

    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join(configured_path).into_os_string()
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

pub(super) async fn rollback_from(pool: &PgPool, target_version: i64) {
    let target_applied: i64 = query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE version = $1")
        .bind(target_version)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(target_applied, 1, "target migration {target_version} is not applied");

    let steps: i64 = query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE version >= $1")
        .bind(target_version)
        .fetch_one(pool)
        .await
        .unwrap();
    down(pool, Some(u32::try_from(steps).unwrap())).await.unwrap();
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
