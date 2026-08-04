use std::time::Duration;

use configuration::{DatabasePoolSettings, DatabaseScheme, DatabaseSessionSettings, DatabaseSettings, DatabaseSslMode};
use sqlx::{postgres::PgPoolOptions, query_as};
use storage::{StorageError, connect_database};
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use tokio::time::{sleep, timeout};
use uuid::Uuid;

const POSTGRES_PORT: u16 = 5_432;
const POSTGRES_IMAGE: &str = "postgres";
const POSTGRES_TAG: &str = "17-alpine";
const DATABASE_NAME: &str = "storage_config";
const DATABASE_USER: &str = "storage_config";
const READY_LOG: &str = "database system is ready to accept connections";
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
const CONNECTION_RETRY_INTERVAL: Duration = Duration::from_millis(100);

#[tokio::test]
async fn invalid_pool_configuration_fails_before_network_connection() {
    let mut settings = database_settings(1, "unused", 1);
    settings.pool.max_connections = 0;

    let error = match connect_database(&settings).await {
        Ok(_) => panic!("zero max_connections must fail validation"),
        Err(error) => error,
    };

    assert!(matches!(error, StorageError::Database(message) if message.contains("database.pool.max_connections")));
}

#[tokio::test]
async fn connection_initialization_applies_session_settings_and_pool_exhaustion_is_bounded() {
    let database = TestPostgres::start().await;
    let settings = database_settings(database.port, &database.password, 1);
    let configured = connect_database(&settings).await.unwrap();
    let session: (String, String, String, String) = query_as(
        "SELECT current_setting('application_name'),current_setting('statement_timeout'),current_setting('lock_timeout'),current_setting('idle_in_transaction_session_timeout')",
    )
    .fetch_one(configured.raw_pool())
    .await
    .unwrap();

    assert_eq!(session, ("taco-storage-test".into(), "1700ms".into(), "2300ms".into(), "3700ms".into()));
    let held = configured.raw_pool().acquire().await.unwrap();
    let result = configured.raw_pool().acquire().await;
    assert!(matches!(result, Err(sqlx::Error::PoolTimedOut)));
    drop(held);
    configured.raw_pool().close().await;
    database.stop().await;
}

#[tokio::test]
async fn connection_initialization_failure_is_visible_to_the_caller() {
    let database = TestPostgres::start().await;
    let settings = database_settings(database.port, "wrong-password", 1);

    let error = match connect_database(&settings).await {
        Ok(_) => panic!("wrong credentials must fail connection initialization"),
        Err(error) => error,
    };

    assert!(matches!(error, StorageError::Database(_)));
    database.stop().await;
}

fn database_settings(port: u16, password: &str, max_connections: u32) -> DatabaseSettings {
    DatabaseSettings {
        scheme: DatabaseScheme::Postgres,
        ssl_mode: DatabaseSslMode::Disable,
        host: "127.0.0.1".into(),
        port,
        username: DATABASE_USER.into(),
        password: password.into(),
        name: DATABASE_NAME.into(),
        pool: DatabasePoolSettings {
            max_connections,
            acquire_timeout_ms: 120,
            idle_timeout_ms: 60_000,
            max_lifetime_ms: 300_000,
        },
        session: DatabaseSessionSettings {
            application_name: "taco-storage-test".into(),
            statement_timeout_ms: 1_700,
            lock_timeout_ms: 2_300,
            idle_in_transaction_session_timeout_ms: 3_700,
        },
    }
}

struct TestPostgres {
    container: ContainerAsync<GenericImage>,
    port: u16,
    password: String,
}

impl TestPostgres {
    async fn start() -> Self {
        let password = Uuid::now_v7().to_string();
        let container = GenericImage::new(POSTGRES_IMAGE, POSTGRES_TAG)
            .with_exposed_port(POSTGRES_PORT.tcp())
            .with_wait_for(WaitFor::message_on_stdout(READY_LOG))
            .with_env_var("POSTGRES_DB", DATABASE_NAME)
            .with_env_var("POSTGRES_USER", DATABASE_USER)
            .with_env_var("POSTGRES_PASSWORD", &password)
            .start()
            .await
            .unwrap();
        let port = container.get_host_port_ipv4(POSTGRES_PORT.tcp()).await.unwrap();
        wait_for_connection(port, &password).await;
        Self { container, port, password }
    }

    async fn stop(self) {
        self.container.stop().await.unwrap();
    }
}

async fn wait_for_connection(port: u16, password: &str) {
    let url = format!("postgres://{DATABASE_USER}:{password}@127.0.0.1:{port}/{DATABASE_NAME}");
    timeout(CONNECTION_TIMEOUT, async {
        loop {
            match PgPoolOptions::new().max_connections(1).connect(&url).await {
                Ok(pool) => {
                    pool.close().await;
                    return;
                }
                Err(_) => sleep(CONNECTION_RETRY_INTERVAL).await,
            }
        }
    })
    .await
    .expect("PostgreSQL test container must accept connections");
}
