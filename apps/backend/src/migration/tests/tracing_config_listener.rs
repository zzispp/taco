use sqlx::{PgPool, query};

use super::{TestDatabase, up};

const DEBUG_TRACING_CONFIG: &str = r#"{"log_level":"debug","http":{"access_enabled":true,"capture_request_body":false,"capture_response_body":false,"capture_query_parameters":true,"capture_request_headers":false,"max_body_capture_bytes":16384},"slow_operation_ms":{"postgres":500,"redis":100,"outbound_http":1000}}"#;
const INFO_TRACING_CONFIG: &str = r#"{"log_level":"info","http":{"access_enabled":true,"capture_request_body":false,"capture_response_body":false,"capture_query_parameters":true,"capture_request_headers":false,"max_body_capture_bytes":16384},"slow_operation_ms":{"postgres":500,"redis":100,"outbound_http":1000}}"#;

#[tokio::test]
async fn tracing_listener_initial_snapshot_observes_a_commit_after_subscription() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let listener = crate::composition::tracing_config_listener::test_listener(database.pool()).await.unwrap();

    update_tracing_config(database.pool(), DEBUG_TRACING_CONFIG).await;
    let config = crate::composition::tracing_config_listener::test_read_runtime_config(database.pool())
        .await
        .unwrap();

    assert_eq!(config.log_level, taco_tracing::TracingLevel::Debug);
    drop(listener);
    database.drop().await;
}

#[tokio::test]
async fn tracing_listener_reconnect_reconciles_a_notification_gap() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let listener = crate::composition::tracing_config_listener::test_listener(database.pool()).await.unwrap();
    let state = taco_tracing::RuntimeTracingState::new(taco_tracing::parse_runtime_tracing_config(INFO_TRACING_CONFIG).unwrap());
    let (runtime, health) = crate::composition::tracing_config_listener::start_tracing_config_listener(listener, database.pool().clone(), state.clone());
    let control = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&database.database_url())
        .await
        .unwrap();

    query("SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = current_database() AND pid <> pg_backend_pid()")
        .execute(&control)
        .await
        .unwrap();
    update_tracing_config(&control, DEBUG_TRACING_CONFIG).await;
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while state.current().log_level != taco_tracing::TracingLevel::Debug {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("listener did not reconcile after reconnect");
    assert_eq!(state.current().log_level, taco_tracing::TracingLevel::Debug);
    assert!(health.snapshot().failures >= 1);

    drop(runtime);
    control.close().await;
    database.drop().await;
}

async fn update_tracing_config(pool: &PgPool, config: &str) {
    query("UPDATE sys_config SET config_value = $1 WHERE config_key = 'sys.observability.tracingConfig'")
        .bind(config)
        .execute(pool)
        .await
        .unwrap();
}
