use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use audit_contract::{
    AUDIT_OUTBOX_PAYLOAD_VERSION, ActorSnapshot, AuditOutboxEvent, AuditOutboxRecord, AuditStatus, AuditStream, BusinessType, LoginEventType,
    OperationAuditEvent, OperatorType, SecurityAuditEvent,
};
use sqlx::{PgPool, migrate::Migrator, postgres::PgPoolOptions, query_as, query_scalar};
use storage::outbox::{append_audit_record, clear_audit_stream, lock_audit_stream_shared};
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use time::OffsetDateTime;
use tokio::time::{sleep, timeout};
use uuid::Uuid;

const POSTGRES_PORT: u16 = 5_432;
const MAX_CONNECTIONS: u32 = 8;
const POSTGRES_IMAGE: &str = "postgres";
const POSTGRES_TAG: &str = "17-alpine";
const DATABASE_NAME: &str = "storage";
const DATABASE_USER: &str = "storage";
const READY_LOG: &str = "database system is ready to accept connections";
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
const CONNECTION_RETRY_INTERVAL: Duration = Duration::from_millis(100);
const LOCK_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(10);

#[tokio::test]
async fn append_is_visible_only_when_the_callers_transaction_commits() {
    let database = TestDatabase::create().await;
    let committed = operation_record("committed");
    let rolled_back = operation_record("rolled-back");

    let mut transaction = database.pool().begin().await.unwrap();
    append_audit_record(&mut transaction, &committed).await.unwrap();
    assert_eq!(outbox_count(database.pool(), &committed.id).await, 0);
    transaction.commit().await.unwrap();
    assert_eq!(outbox_count(database.pool(), &committed.id).await, 1);
    assert_persisted_record(database.pool(), &committed).await;

    let mut transaction = database.pool().begin().await.unwrap();
    append_audit_record(&mut transaction, &rolled_back).await.unwrap();
    transaction.rollback().await.unwrap();
    assert_eq!(outbox_count(database.pool(), &rolled_back.id).await, 0);

    database.close().await;
}

#[tokio::test]
async fn clear_waits_for_shared_stream_users_and_remains_transactional() {
    let database = TestDatabase::create().await;
    append_committed(database.pool(), operation_record("existing")).await;
    let mut shared_lock = database.pool().begin().await.unwrap();
    lock_audit_stream_shared(&mut shared_lock, AuditStream::Operation).await.unwrap();

    let clear_task = tokio::spawn({
        let pool = database.pool().clone();
        async move {
            let mut transaction = pool.begin().await.unwrap();
            clear_audit_stream(&mut transaction, AuditStream::Operation).await.unwrap();
            transaction.rollback().await.unwrap();
        }
    });

    wait_for_blocked_advisory_lock(database.pool()).await;
    assert_eq!(outbox_count(database.pool(), "existing").await, 1);
    shared_lock.commit().await.unwrap();
    clear_task.await.unwrap();
    assert_eq!(outbox_count(database.pool(), "existing").await, 1);

    database.close().await;
}

#[tokio::test]
async fn append_waits_for_clear_and_survives_when_it_runs_after_clear() {
    let database = TestDatabase::create().await;
    append_committed(database.pool(), operation_record("old")).await;
    append_committed(database.pool(), security_record("other-stream")).await;
    let mut clear = database.pool().begin().await.unwrap();
    clear_audit_stream(&mut clear, AuditStream::Operation).await.unwrap();

    let append_task = tokio::spawn({
        let pool = database.pool().clone();
        async move { append_committed(&pool, operation_record("new")).await }
    });

    wait_for_blocked_advisory_lock(database.pool()).await;
    clear.commit().await.unwrap();
    append_task.await.unwrap();
    assert_eq!(outbox_count(database.pool(), "old").await, 0);
    assert_eq!(outbox_count(database.pool(), "new").await, 1);
    assert_eq!(outbox_count(database.pool(), "other-stream").await, 1);

    database.close().await;
}

async fn append_committed(pool: &PgPool, record: AuditOutboxRecord) {
    let mut transaction = pool.begin().await.unwrap();
    append_audit_record(&mut transaction, &record).await.unwrap();
    transaction.commit().await.unwrap();
}

async fn outbox_count(pool: &PgPool, id: &str) -> i64 {
    query_scalar("SELECT COUNT(*) FROM audit_outbox WHERE outbox_id=$1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn assert_persisted_record(pool: &PgPool, record: &AuditOutboxRecord) {
    let row: (String, String, i16, serde_json::Value, OffsetDateTime) =
        query_as("SELECT stream,event_type,payload_version,payload,occurred_at FROM audit_outbox WHERE outbox_id=$1")
            .bind(&record.id)
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(row.0, record.stream().code());
    assert_eq!(row.1, record.event_type());
    assert_eq!(row.2, AUDIT_OUTBOX_PAYLOAD_VERSION);
    assert_eq!(row.3, record.payload().unwrap());
    assert_eq!(row.4, record.occurred_at);
}

async fn wait_for_blocked_advisory_lock(pool: &PgPool) {
    timeout(LOCK_WAIT_TIMEOUT, async {
        loop {
            let blocked: bool = query_scalar("SELECT EXISTS (SELECT 1 FROM pg_locks WHERE locktype='advisory' AND NOT granted)")
                .fetch_one(pool)
                .await
                .unwrap();
            if blocked {
                return;
            }
            sleep(LOCK_POLL_INTERVAL).await;
        }
    })
    .await
    .expect("outbox operation must wait for the stream advisory lock");
}

fn operation_record(id: &str) -> AuditOutboxRecord {
    AuditOutboxRecord {
        id: id.into(),
        occurred_at: OffsetDateTime::UNIX_EPOCH,
        event: AuditOutboxEvent::Operation(OperationAuditEvent {
            title_key: "audit.module.storage_test".into(),
            business_type: BusinessType::Insert,
            handler: "storage::outbox_test".into(),
            request_method: "POST".into(),
            operator_type: OperatorType::Manage,
            actor: ActorSnapshot {
                user_id: Some("user-1".into()),
                username: "alice".into(),
                department_id: None,
                department_name: String::new(),
            },
            operation_url: "/test".into(),
            operation_ip: "127.0.0.1".into(),
            status: AuditStatus::Success,
            request_id: "request-1".into(),
            request_params: String::new(),
            response_result: String::new(),
            error_message: String::new(),
            cost_time_ms: 1,
        }),
    }
}

fn security_record(id: &str) -> AuditOutboxRecord {
    AuditOutboxRecord {
        id: id.into(),
        occurred_at: OffsetDateTime::UNIX_EPOCH,
        event: AuditOutboxEvent::Security(SecurityAuditEvent {
            request_id: "request-1".into(),
            route: "/login".into(),
            user_id: Some("user-1".into()),
            username: "alice".into(),
            ip_address: "127.0.0.1".into(),
            browser: "Test".into(),
            os: "Test".into(),
            status: AuditStatus::Success,
            event_type: LoginEventType::LoginSuccess,
            message_key: "audit.login.success".into(),
            message_params: BTreeMap::new(),
        }),
    }
}

struct TestDatabase {
    pool: PgPool,
    container: ContainerAsync<GenericImage>,
}

impl TestDatabase {
    async fn create() -> Self {
        let password = Uuid::now_v7().to_string();
        let container = start_postgres(&password).await;
        let port = container.get_host_port_ipv4(POSTGRES_PORT.tcp()).await.unwrap();
        let pool = connect_when_ready(port, &password).await;
        Migrator::new(migrations_path()).await.unwrap().run(&pool).await.unwrap();
        Self { pool, container }
    }

    fn pool(&self) -> &PgPool {
        &self.pool
    }

    async fn close(self) {
        self.pool.close().await;
        self.container.stop().await.unwrap();
    }
}

async fn start_postgres(password: &str) -> ContainerAsync<GenericImage> {
    GenericImage::new(POSTGRES_IMAGE, POSTGRES_TAG)
        .with_exposed_port(POSTGRES_PORT.tcp())
        .with_wait_for(WaitFor::message_on_stdout(READY_LOG))
        .with_env_var("POSTGRES_DB", DATABASE_NAME)
        .with_env_var("POSTGRES_USER", DATABASE_USER)
        .with_env_var("POSTGRES_PASSWORD", password)
        .start()
        .await
        .unwrap()
}

async fn connect_when_ready(port: u16, password: &str) -> PgPool {
    let url = format!("postgres://{DATABASE_USER}:{password}@127.0.0.1:{port}/{DATABASE_NAME}");
    let started = std::time::Instant::now();
    loop {
        match PgPoolOptions::new().max_connections(MAX_CONNECTIONS).connect(&url).await {
            Ok(pool) => return pool,
            Err(error) if started.elapsed() >= CONNECTION_TIMEOUT => panic!("PostgreSQL connection timed out: {error}"),
            Err(_) => sleep(CONNECTION_RETRY_INTERVAL).await,
        }
    }
}

fn migrations_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../migrations")
}
