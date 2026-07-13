use std::{sync::Arc, time::Duration};

use configuration::{SchedulerSettings, Settings};
use scheduler::{
    application::{
        SchedulerRuntimeConfig, SchedulerRuntimeHandle, SchedulerRuntimeParts, SchedulerService, SchedulerServiceParts, SchedulerUseCase,
        start_scheduler_runtime,
        task::{ScheduledTaskMetadata, StaticTaskCatalog, SystemCacheRefreshPort, TaskExecutionContext, TaskExecutionFailure},
        tasks::{HttpRequestTask, RefreshConfigCacheTask, RefreshDictCacheTask},
    },
    infra::{
        MetricsSchedulerTelemetry, PostgresChangeListenerFactory, PostgresExecutionLease, PostgresLeaderLease, ReqwestHttpTaskClient,
        StorageSchedulerRepository,
    },
};
use storage::Database;
use system::application::{SystemError, SystemUseCase};

use super::runtime_config::RuntimeSchedulerConfig;
use crate::BackendResult;

pub(super) struct SchedulerServices {
    pub(super) use_case: Arc<dyn SchedulerUseCase>,
    pub(super) export_config: Arc<RuntimeSchedulerConfig>,
    pub(super) runtime: SchedulerRuntimeHandle,
}

#[derive(Clone)]
struct SchedulerSystemCacheAdapter {
    system: Arc<dyn SystemUseCase>,
}

struct RuntimeAssembly<'a> {
    config: &'a SchedulerSettings,
    repository: Arc<StorageSchedulerRepository>,
    catalog: Arc<StaticTaskCatalog>,
    system: Arc<dyn SystemUseCase>,
    pool: sqlx::PgPool,
    executor_epoch: String,
}

#[async_trait::async_trait]
impl SystemCacheRefreshPort for SchedulerSystemCacheAdapter {
    async fn refresh_config_cache(&self) -> Result<(), TaskExecutionFailure> {
        self.system.refresh_config_cache().await.map_err(|error| cache_refresh_error("config", error))
    }

    async fn refresh_dict_cache(&self) -> Result<(), TaskExecutionFailure> {
        self.system.refresh_dict_cache().await.map_err(|error| cache_refresh_error("dict", error))
    }
}

pub(super) fn build_scheduler_services(settings: &Settings, database: Database, system: Arc<dyn SystemUseCase>) -> BackendResult<SchedulerServices> {
    let config = settings.scheduler_config()?;
    let pool = database.pool().clone();
    let executor_epoch = database.next_id();
    let repository = Arc::new(StorageSchedulerRepository::new(database));
    let catalog = StaticTaskCatalog::try_new([
        HttpRequestTask::descriptor(),
        RefreshConfigCacheTask::descriptor(),
        RefreshDictCacheTask::descriptor(),
    ])?;
    let assembly = RuntimeAssembly {
        config: &config,
        repository: repository.clone(),
        catalog: catalog.clone(),
        system: system.clone(),
        pool,
        executor_epoch,
    };
    let runtime = start_scheduler_runtime(
        runtime_parts(assembly)?,
        SchedulerRuntimeConfig {
            reconcile_interval: Duration::from_millis(config.runtime.reconcile_interval_ms),
        },
    );
    let service = scheduler_service(repository, catalog);
    Ok(SchedulerServices {
        use_case: service,
        export_config: Arc::new(RuntimeSchedulerConfig::new(system)),
        runtime,
    })
}

fn runtime_parts(assembly: RuntimeAssembly<'_>) -> BackendResult<SchedulerRuntimeParts> {
    let context = TaskExecutionContext {
        system_cache: Arc::new(SchedulerSystemCacheAdapter { system: assembly.system }),
        http_client: Arc::new(ReqwestHttpTaskClient::new(scheduler_http_client(assembly.config)?)),
    };
    Ok(SchedulerRuntimeParts {
        store: assembly.repository,
        catalog: assembly.catalog,
        task_context: context,
        leader_lease: Arc::new(PostgresLeaderLease::new(assembly.pool.clone())),
        listener_factory: Arc::new(PostgresChangeListenerFactory::new(assembly.pool.clone())),
        execution_lease: Arc::new(PostgresExecutionLease::new(assembly.pool)),
        telemetry: Arc::new(MetricsSchedulerTelemetry),
        executor_epoch: assembly.executor_epoch,
    })
}

fn scheduler_service(repository: Arc<StorageSchedulerRepository>, catalog: Arc<StaticTaskCatalog>) -> Arc<dyn SchedulerUseCase> {
    Arc::new(SchedulerService::new(SchedulerServiceParts {
        query: repository.clone(),
        commands: repository.clone(),
        catalog,
        clock: repository,
    }))
}

fn scheduler_http_client(config: &SchedulerSettings) -> BackendResult<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_millis(config.http_client.request_timeout_ms))
        .build()?)
}

fn cache_refresh_error(kind: &'static str, error: SystemError) -> TaskExecutionFailure {
    TaskExecutionFailure::new(
        kernel::error::LocalizedError::new("errors.scheduler.task_cache_refresh_failed").with_param("kind", kind),
        format!("scheduler {kind} cache refresh failed: {error}"),
    )
}
