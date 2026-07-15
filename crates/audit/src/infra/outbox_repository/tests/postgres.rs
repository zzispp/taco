use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions};
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use uuid::Uuid;

const POSTGRES_PORT: u16 = 5_432;
const MAX_CONNECTIONS: u32 = 8;
const POSTGRES_IMAGE: &str = "postgres";
const POSTGRES_TAG: &str = "17-alpine";
const DATABASE_NAME: &str = "audit";
const DATABASE_USER: &str = "audit";
const READY_LOG: &str = "database system is ready to accept connections";
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
const CONNECTION_RETRY_INTERVAL: Duration = Duration::from_millis(100);

pub(super) struct TestDatabase {
    pool: PgPool,
    container: ContainerAsync<GenericImage>,
}

impl TestDatabase {
    pub(super) async fn create() -> Self {
        let password = Uuid::now_v7().to_string();
        let container = GenericImage::new(POSTGRES_IMAGE, POSTGRES_TAG)
            .with_exposed_port(POSTGRES_PORT.tcp())
            .with_wait_for(WaitFor::message_on_stdout(READY_LOG))
            .with_env_var("POSTGRES_DB", DATABASE_NAME)
            .with_env_var("POSTGRES_USER", DATABASE_USER)
            .with_env_var("POSTGRES_PASSWORD", password.as_str())
            .start()
            .await
            .expect("test PostgreSQL container must start");
        let port = container
            .get_host_port_ipv4(POSTGRES_PORT.tcp())
            .await
            .expect("test PostgreSQL port must be exposed");
        let pool = connect_when_ready(port, &password).await.expect("test PostgreSQL connection must succeed");
        Migrator::new(migrations_path())
            .await
            .expect("audit migrations must load")
            .run(&pool)
            .await
            .expect("audit schema migrations must succeed");
        Self { pool, container }
    }

    pub(super) fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(super) async fn close(self) {
        let Self { pool, container } = self;
        pool.close().await;
        container.stop().await.expect("test PostgreSQL container must stop");
    }
}

fn migrations_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../migrations")
}

fn database_url(port: u16, password: &str) -> String {
    format!("postgres://{DATABASE_USER}:{password}@127.0.0.1:{port}/{DATABASE_NAME}")
}

async fn connect_when_ready(port: u16, password: &str) -> Result<PgPool, String> {
    let url = database_url(port, password);
    let started = Instant::now();
    loop {
        match PgPoolOptions::new().max_connections(MAX_CONNECTIONS).connect(&url).await {
            Ok(pool) => return Ok(pool),
            Err(error) if started.elapsed() >= CONNECTION_TIMEOUT => {
                return Err(format!("PostgreSQL did not accept connections within {CONNECTION_TIMEOUT:?}: {error}"));
            }
            Err(_) => tokio::time::sleep(CONNECTION_RETRY_INTERVAL).await,
        }
    }
}
