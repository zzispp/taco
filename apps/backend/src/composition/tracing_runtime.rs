use std::sync::Arc;

use async_trait::async_trait;
use constants::system_config::EXPORT_BATCH_CONFIG_KEY;
use kernel::runtime_config::{ExportBatchConfig, ExportConfigProvider};
use observability::{
    application::{ObservabilityError, SystemLogRepository, SystemLogRetentionUseCase, SystemLogService, SystemLogUseCase},
    infra::{ObservabilitySystemLogSink, StorageSystemLogRepository},
};
use system::application::{SystemError, SystemUseCase};
use taco_tracing::{
    HttpLogCaptureState, InfrastructureObserver, RuntimeTracingState, SystemLogLayer, SystemLogRuntime, init_global_subscriber,
    start_system_log_runtime_with_state,
};

use crate::BackendResult;

pub(crate) use super::tracing_config_listener::{TracingConfigListenerHealth, TracingConfigListenerRuntime};
use super::tracing_config_listener::{establish_tracing_config_subscription, start_tracing_config_listener};

pub(super) struct ObservabilityServices {
    pub(super) logs: Arc<dyn SystemLogUseCase>,
    pub(super) retention: Arc<dyn SystemLogRetentionUseCase>,
    pub(super) system_log_runtime: Arc<SystemLogRuntime>,
    pub(super) config_listener_runtime: Arc<TracingConfigListenerRuntime>,
    pub(super) config_listener_health: TracingConfigListenerHealth,
    pub(super) infrastructure_observer: InfrastructureObserver,
    pub(super) http_log_state: HttpLogCaptureState,
}

pub(super) async fn build_observability_services(database: storage::Database) -> BackendResult<ObservabilityServices> {
    let (listener, config) = establish_tracing_config_subscription(database.raw_pool()).await?;
    let tracing_state = RuntimeTracingState::new(config);
    let infrastructure_observer = InfrastructureObserver::new(tracing_state.clone());
    let repository: Arc<dyn SystemLogRepository> = Arc::new(StorageSystemLogRepository::new(database.clone()));
    let service = Arc::new(SystemLogService::new(repository.clone()));
    let logs: Arc<dyn SystemLogUseCase> = service.clone();
    let retention: Arc<dyn SystemLogRetentionUseCase> = service;
    let runtime = Arc::new(start_system_log_runtime_with_state(
        Arc::new(ObservabilitySystemLogSink::new(repository)),
        tracing_state.clone(),
    ));
    init_global_subscriber(tracing_state.clone(), SystemLogLayer::new(runtime.emitter()))?;
    database.set_postgres_observer(Arc::new(TracingPostgresObserver {
        observer: infrastructure_observer.clone(),
    }));
    let http_log_state = HttpLogCaptureState::from_runtime_state(tracing_state.clone());
    let (config_listener_runtime, config_listener_health) = start_tracing_config_listener(listener, database.raw_pool().clone(), tracing_state);
    Ok(ObservabilityServices {
        logs,
        retention,
        system_log_runtime: runtime,
        config_listener_runtime: Arc::new(config_listener_runtime),
        config_listener_health,
        infrastructure_observer,
        http_log_state,
    })
}

#[derive(Clone)]
struct TracingPostgresObserver {
    observer: InfrastructureObserver,
}

impl storage::PostgresOperationObserver for TracingPostgresObserver {
    fn record(&self, operation: &'static str, elapsed: std::time::Duration, succeeded: bool) {
        self.observer
            .record(taco_tracing::InfrastructureDependency::Postgres, operation, elapsed, succeeded);
    }
}

pub(super) fn observability_export_config(system: Arc<dyn SystemUseCase>) -> Arc<dyn ExportConfigProvider<Error = ObservabilityError>> {
    Arc::new(RuntimeObservabilityConfig::new(system))
}

#[derive(Clone)]
struct RuntimeObservabilityConfig {
    system: Arc<dyn SystemUseCase>,
}

impl RuntimeObservabilityConfig {
    fn new(system: Arc<dyn SystemUseCase>) -> Self {
        Self { system }
    }
}

#[async_trait]
impl ExportConfigProvider for RuntimeObservabilityConfig {
    type Error = ObservabilityError;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error> {
        let value = self.system.config_by_key(EXPORT_BATCH_CONFIG_KEY).await.map_err(observability_config_error)?;
        kernel::runtime_config::parse_export_batch_config(&value).map_err(|error| ObservabilityError::Infrastructure(error.to_string()))
    }
}

fn observability_config_error(error: SystemError) -> ObservabilityError {
    ObservabilityError::Infrastructure(format!("system export configuration read failed: {error}"))
}
