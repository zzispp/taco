use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use constants::system_config::{SYSTEM_CONFIG_CHANGED_CHANNEL, TRACING_CONFIG_KEY};
use metrics::{counter, gauge};
use sqlx::{PgPool, postgres::PgListener, query_scalar};
use taco_tracing::{InfrastructureDependency, InfrastructureObserver, RuntimeTracingConfig, RuntimeTracingState, parse_runtime_tracing_config};

use crate::BackendResult;

const REQUIRED_TRACING_CONFIG_ERROR: &str = "required tracing runtime config is missing";
const LISTENER_RETRY_DELAY: Duration = Duration::from_secs(1);
const LISTENER_HEALTH_METRIC: &str = "tracing_config_listener_healthy";
const LISTENER_FAILURE_METRIC: &str = "tracing_config_listener_failures_total";

#[derive(Clone)]
pub(crate) struct TracingConfigListenerHealth(Arc<TracingConfigListenerHealthState>);

struct TracingConfigListenerHealthState {
    healthy: AtomicBool,
    failures: AtomicU64,
    last_failure: Mutex<Option<String>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TracingConfigListenerHealthSnapshot {
    pub(crate) healthy: bool,
    pub(crate) failures: u64,
    pub(crate) last_failure: Option<String>,
}

impl TracingConfigListenerHealth {
    pub(crate) fn new() -> Self {
        Self(Arc::new(TracingConfigListenerHealthState {
            healthy: AtomicBool::new(true),
            failures: AtomicU64::new(0),
            last_failure: Mutex::new(None),
        }))
    }

    pub(crate) fn snapshot(&self) -> TracingConfigListenerHealthSnapshot {
        TracingConfigListenerHealthSnapshot {
            healthy: self.0.healthy.load(Ordering::Relaxed),
            failures: self.0.failures.load(Ordering::Relaxed),
            last_failure: self.0.last_failure.lock().unwrap().clone(),
        }
    }

    fn recovered(&self) {
        self.0.healthy.store(true, Ordering::Relaxed);
        gauge!(LISTENER_HEALTH_METRIC).set(1.0);
    }

    fn failed(&self, operation: &'static str) {
        self.0.healthy.store(false, Ordering::Relaxed);
        self.0.failures.fetch_add(1, Ordering::Relaxed);
        *self.0.last_failure.lock().unwrap() = Some(operation.into());
        gauge!(LISTENER_HEALTH_METRIC).set(0.0);
        counter!(LISTENER_FAILURE_METRIC, "operation" => operation).increment(1);
    }
}

pub(crate) struct TracingConfigListenerRuntime {
    task: tokio::task::JoinHandle<()>,
}

struct ListenerRuntime {
    listener: PgListener,
    pool: PgPool,
    state: RuntimeTracingState,
    health: TracingConfigListenerHealth,
    observer: InfrastructureObserver,
}

struct ListenerRuntimeParts {
    listener: PgListener,
    pool: PgPool,
    state: RuntimeTracingState,
    health: TracingConfigListenerHealth,
}

impl ListenerRuntime {
    fn new(parts: ListenerRuntimeParts) -> Self {
        Self {
            observer: InfrastructureObserver::new(parts.state.clone()),
            listener: parts.listener,
            pool: parts.pool,
            state: parts.state,
            health: parts.health,
        }
    }
}

impl Drop for TracingConfigListenerRuntime {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub(super) async fn establish_tracing_config_subscription(pool: &PgPool) -> BackendResult<(PgListener, RuntimeTracingConfig)> {
    let listener = connect_listener(pool).await?;
    let config = read_runtime_tracing_config(pool).await?;
    Ok((listener, config))
}

pub(crate) fn start_tracing_config_listener(
    listener: PgListener,
    pool: PgPool,
    state: RuntimeTracingState,
) -> (TracingConfigListenerRuntime, TracingConfigListenerHealth) {
    let health = TracingConfigListenerHealth::new();
    let runtime = ListenerRuntime::new(ListenerRuntimeParts {
        listener,
        pool,
        state,
        health: health.clone(),
    });
    let task = tokio::spawn(run_listener(runtime));
    (TracingConfigListenerRuntime { task }, health)
}

async fn connect_listener(pool: &PgPool) -> Result<PgListener, sqlx::Error> {
    let mut listener = PgListener::connect_with(pool).await?;
    listener.listen(SYSTEM_CONFIG_CHANGED_CHANNEL).await?;
    Ok(listener)
}

async fn run_listener(mut runtime: ListenerRuntime) {
    loop {
        match runtime.listener.try_recv().await {
            Ok(Some(_)) => reconcile_until_success(&runtime).await,
            Ok(None) => {
                runtime.health.failed("disconnect");
                runtime
                    .observer
                    .record(InfrastructureDependency::Postgres, "tracing_config_listener_disconnect", Duration::ZERO, false);
                taco_tracing::warn_with_fields!(
                    "observability tracing configuration listener reconnected",
                    channel = SYSTEM_CONFIG_CHANGED_CHANNEL
                );
                reconcile_until_success(&runtime).await;
            }
            Err(error) => {
                runtime.health.failed("listen");
                runtime
                    .observer
                    .record(InfrastructureDependency::Postgres, "tracing_config_listen", Duration::ZERO, false);
                taco_tracing::error_with_fields!(
                    "observability tracing configuration listener failed",
                    &error,
                    channel = SYSTEM_CONFIG_CHANGED_CHANNEL
                );
                reconcile_until_success(&runtime).await;
            }
        }
    }
}

async fn reconcile_until_success(runtime: &ListenerRuntime) {
    loop {
        match read_observed_runtime_tracing_config(&runtime.pool, &runtime.observer).await {
            Ok(config) => {
                runtime.state.reload(config);
                runtime.health.recovered();
                taco_tracing::info_with_fields!("observability tracing runtime configuration reloaded", key = TRACING_CONFIG_KEY);
                return;
            }
            Err(error) => {
                runtime.health.failed("reconcile");
                taco_tracing::error_with_fields!("observability tracing configuration reload failed", error.as_ref(), key = TRACING_CONFIG_KEY);
                tokio::time::sleep(LISTENER_RETRY_DELAY).await;
            }
        }
    }
}

async fn read_runtime_tracing_config(pool: &PgPool) -> BackendResult<RuntimeTracingConfig> {
    let value = query_scalar::<_, String>("SELECT config_value FROM sys_config WHERE config_key = $1")
        .bind(TRACING_CONFIG_KEY)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| std::io::Error::other(REQUIRED_TRACING_CONFIG_ERROR))?;
    Ok(parse_runtime_tracing_config(&value)?)
}

async fn read_observed_runtime_tracing_config(pool: &PgPool, observer: &InfrastructureObserver) -> BackendResult<RuntimeTracingConfig> {
    let value = observer
        .observe(
            InfrastructureDependency::Postgres,
            "tracing_config_read",
            query_scalar::<_, String>("SELECT config_value FROM sys_config WHERE config_key = $1")
                .bind(TRACING_CONFIG_KEY)
                .fetch_optional(pool),
        )
        .await?
        .ok_or_else(|| std::io::Error::other(REQUIRED_TRACING_CONFIG_ERROR))?;
    Ok(parse_runtime_tracing_config(&value)?)
}

#[cfg(test)]
pub(crate) async fn test_listener(pool: &PgPool) -> Result<PgListener, sqlx::Error> {
    connect_listener(pool).await
}

#[cfg(test)]
pub(crate) async fn test_read_runtime_config(pool: &PgPool) -> BackendResult<RuntimeTracingConfig> {
    read_runtime_tracing_config(pool).await
}

#[cfg(test)]
mod tests {
    use super::TracingConfigListenerHealth;

    #[test]
    fn listener_health_tracks_failure_and_recovery() {
        let health = TracingConfigListenerHealth::new();
        health.failed("listen");

        let failed = health.snapshot();
        assert!(!failed.healthy);
        assert_eq!(failed.failures, 1);
        assert_eq!(failed.last_failure.as_deref(), Some("listen"));

        health.recovered();

        assert!(health.snapshot().healthy);
    }
}
