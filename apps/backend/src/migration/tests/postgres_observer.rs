use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use async_trait::async_trait;
use sqlx::query_scalar;
use storage::{Database, PostgresOperationObserver};
use taco_tracing::{
    HttpLogCaptureConfig, InfrastructureDependency, InfrastructureObserver, RuntimeTracingConfig, RuntimeTracingState, SlowOperationThresholds, SystemLogEvent,
    SystemLogLayer, SystemLogSink, TracingLevel, start_system_log_runtime_with_state,
};
use tracing_subscriber::{Registry, layer::SubscriberExt};

use super::TestDatabase;

#[tokio::test]
async fn shared_postgres_boundary_observes_failure_slow_threshold_and_runtime_reload() {
    let test_database = TestDatabase::create().await;
    let database = Database::new(test_database.pool().clone());
    let state = RuntimeTracingState::new(config(1_000));
    let sink = Arc::new(CollectingSink::default());
    let runtime = start_system_log_runtime_with_state(sink.clone(), state.clone());
    database.set_postgres_observer(Arc::new(TracingPostgresObserver {
        observer: InfrastructureObserver::new(state.clone()),
    }));
    let subscriber = Registry::default().with(SystemLogLayer::new(runtime.emitter()));
    taco_tracing::__tracing::subscriber::set_global_default(subscriber).expect("PostgreSQL boundary test must install the process subscriber first");

    run_queries(database.clone(), state.clone()).await;
    wait_for_events(&sink).await;

    let events = sink.events();
    let postgres_events = events
        .iter()
        .filter(|event| event.fields["dependency"] == "postgres" && event.fields["operation"] == "postgres_fetch_optional")
        .collect::<Vec<_>>();
    assert_eq!(postgres_events.len(), 2);
    assert!(postgres_events.iter().any(|event| event.message == "slow infrastructure operation"));
    assert!(postgres_events.iter().any(|event| event.message == "infrastructure operation failed"));

    runtime.shutdown().await;
    test_database.drop().await;
}

async fn run_queries(database: Database, state: RuntimeTracingState) {
    let first: i32 = query_scalar("SELECT 1")
        .fetch_one(database.pool())
        .await
        .expect("initial PostgreSQL query must succeed");
    assert_eq!(first, 1);
    state.reload(config(0));
    let second: i32 = query_scalar("SELECT 2")
        .fetch_one(database.pool())
        .await
        .expect("reloaded PostgreSQL query must succeed");
    assert_eq!(second, 2);
    let failed = query_scalar::<_, i64>("SELECT missing_column").fetch_one(database.pool()).await;
    assert!(failed.is_err());
}

fn config(postgres_threshold: u64) -> RuntimeTracingConfig {
    RuntimeTracingConfig {
        log_level: TracingLevel::Trace,
        http: HttpLogCaptureConfig {
            access_enabled: true,
            capture_request_body: false,
            capture_response_body: false,
            capture_query_parameters: false,
            capture_request_headers: false,
            max_body_capture_bytes: 0,
        },
        slow_operation_ms: SlowOperationThresholds {
            postgres: postgres_threshold,
            redis: 1_000,
            outbound_http: 1_000,
        },
    }
}

#[derive(Clone)]
struct TracingPostgresObserver {
    observer: InfrastructureObserver,
}

impl PostgresOperationObserver for TracingPostgresObserver {
    fn record(&self, operation: &'static str, elapsed: Duration, succeeded: bool) {
        self.observer.record(InfrastructureDependency::Postgres, operation, elapsed, succeeded);
    }
}

#[derive(Default)]
struct CollectingSink(Mutex<Vec<SystemLogEvent>>);

impl CollectingSink {
    fn events(&self) -> Vec<SystemLogEvent> {
        self.0.lock().unwrap().clone()
    }
}

#[async_trait]
impl SystemLogSink for CollectingSink {
    async fn insert_batch(&self, events: Vec<SystemLogEvent>) -> Result<(), String> {
        self.0.lock().unwrap().extend(events);
        Ok(())
    }
}

async fn wait_for_events(sink: &CollectingSink) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while postgres_event_count(&sink.events()) < 2 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("PostgreSQL observer did not emit both expected events");
}

fn postgres_event_count(events: &[SystemLogEvent]) -> usize {
    events
        .iter()
        .filter(|event| event.fields["dependency"] == "postgres" && event.fields["operation"] == "postgres_fetch_optional")
        .count()
}
