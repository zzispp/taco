use std::{sync::Arc, time::Duration};

use configuration::{SchedulerSettings, Settings};
use file::application::FileCleanupUseCase;
use observability::{
    application::{SystemLogRetentionUseCase, SystemLogUseCase},
    domain::{SystemLogFilter, SystemLogLevel},
};
use scheduler::{
    application::{
        SchedulerAuditedUseCase, SchedulerRuntimeConfig, SchedulerRuntimeHandle, SchedulerRuntimeParts, SchedulerService, SchedulerServiceParts,
        SchedulerUseCase, start_scheduler_runtime,
        task::{
            FileCleanupPort, FileTrashCleanupResult, FileUploadSessionCleanupResult, ScheduledTaskMetadata, StaticTaskCatalog, SystemCacheRefreshPort,
            SystemLogCleanupFilter, SystemLogCleanupLevel, SystemLogCleanupPort, SystemLogCleanupResult, TaskExecutionContext, TaskExecutionFailure,
        },
        tasks::{
            CacheRefreshKind, CleanupUploadSessionsTask, FileCleanupKind, FileTrashCleanupReport, FileUploadSessionCleanupReport, HttpRequestTask,
            PurgeTrashTask, RefreshConfigCacheTask, RefreshDictCacheTask, SystemLogCleanupReport, SystemLogCleanupTask, cache_refresh_failure,
            file_cleanup_failure,
        },
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
    pub(super) audited: Arc<dyn SchedulerAuditedUseCase>,
    pub(super) export_config: Arc<RuntimeSchedulerConfig>,
    pub(super) runtime: SchedulerRuntimeHandle,
}

#[derive(Clone)]
struct SchedulerSystemCacheAdapter {
    system: Arc<dyn SystemUseCase>,
}

#[derive(Clone)]
struct SchedulerSystemLogCleanupAdapter {
    logs: Arc<dyn SystemLogUseCase>,
    retention: Arc<dyn SystemLogRetentionUseCase>,
}

#[derive(Clone)]
struct SchedulerFileCleanupAdapter {
    files: Arc<dyn FileCleanupUseCase>,
}

struct RuntimeAssembly<'a> {
    config: &'a SchedulerSettings,
    repository: Arc<StorageSchedulerRepository>,
    catalog: Arc<StaticTaskCatalog>,
    system: Arc<dyn SystemUseCase>,
    logs: Arc<dyn SystemLogUseCase>,
    retention: Arc<dyn SystemLogRetentionUseCase>,
    file_cleanup: Arc<dyn FileCleanupUseCase>,
    observer: taco_tracing::InfrastructureObserver,
    pool: sqlx::PgPool,
    executor_epoch: String,
}

#[async_trait::async_trait]
impl SystemCacheRefreshPort for SchedulerSystemCacheAdapter {
    async fn refresh_config_cache(&self) -> Result<(), TaskExecutionFailure> {
        self.system
            .refresh_config_cache()
            .await
            .map_err(|error| cache_refresh_error(CacheRefreshKind::Config, error))
    }

    async fn refresh_dict_cache(&self) -> Result<(), TaskExecutionFailure> {
        self.system
            .refresh_dict_cache()
            .await
            .map_err(|error| cache_refresh_error(CacheRefreshKind::Dict, error))
    }
}

#[async_trait::async_trait]
impl SystemLogCleanupPort for SchedulerSystemLogCleanupAdapter {
    async fn cleanup_expired(&self, retention_days: u64, batch_size: u64) -> Result<SystemLogCleanupResult, TaskExecutionFailure> {
        let report = self
            .retention
            .cleanup_expired(retention_days, batch_size)
            .await
            .map_err(system_log_cleanup_failure)?;
        Ok(SystemLogCleanupResult {
            deleted: report.deleted,
            batches: report.batches,
        })
    }

    async fn cleanup_filtered(&self, filter: SystemLogCleanupFilter, batch_size: u64) -> Result<SystemLogCleanupResult, TaskExecutionFailure> {
        let report = self
            .logs
            .delete_filtered(system_log_filter(filter), batch_size)
            .await
            .map_err(system_log_cleanup_failure)?;
        Ok(SystemLogCleanupResult {
            deleted: report.deleted,
            batches: report.batches,
        })
    }
}

#[async_trait::async_trait]
impl FileCleanupPort for SchedulerFileCleanupAdapter {
    async fn purge_trash(&self, retention_days: u64, batch_size: u64) -> Result<FileTrashCleanupResult, TaskExecutionFailure> {
        let report = self
            .files
            .purge_trash(retention_days, batch_size)
            .await
            .map_err(|error| file_cleanup_failure(FileCleanupKind::PurgeTrash, format!("file trash cleanup failed: {error}")))?;
        let result = FileTrashCleanupResult {
            purged_entries: report.purged_entries,
            blocked_roots: report.blocked_roots,
            deleted_objects: report.deleted_objects,
            failed_objects: report.failed_objects,
            retried_provider_cleanups: report.retried_provider_cleanups,
            failed_provider_cleanups: report.failed_provider_cleanups,
        };
        if result.failed_objects > 0 || result.failed_provider_cleanups > 0 {
            return Err(
                file_cleanup_failure(FileCleanupKind::PurgeTrash, "file trash cleanup completed with provider failures")
                    .with_detail(FileTrashCleanupReport::from(result)),
            );
        }
        Ok(result)
    }

    async fn cleanup_upload_sessions(&self, batch_size: u64) -> Result<FileUploadSessionCleanupResult, TaskExecutionFailure> {
        let report = self
            .files
            .cleanup_upload_sessions(batch_size)
            .await
            .map_err(|error| file_cleanup_failure(FileCleanupKind::UploadSessions, format!("file upload-session cleanup failed: {error}")))?;
        let result = FileUploadSessionCleanupResult {
            expired_sessions: report.expired_sessions,
            reconciled_sessions: report.reconciled_sessions,
            retried_provider_cleanups: report.retried_provider_cleanups,
            failed_provider_cleanups: report.failed_provider_cleanups,
        };
        if result.failed_provider_cleanups > 0 {
            return Err(
                file_cleanup_failure(FileCleanupKind::UploadSessions, "file upload-session cleanup completed with provider failures")
                    .with_detail(FileUploadSessionCleanupReport::from(result)),
            );
        }
        Ok(result)
    }
}

pub(super) fn build_scheduler_services(
    settings: &Settings,
    database: Database,
    system: Arc<dyn SystemUseCase>,
    logs: Arc<dyn SystemLogUseCase>,
    retention: Arc<dyn SystemLogRetentionUseCase>,
    file_cleanup: Arc<dyn FileCleanupUseCase>,
    observer: taco_tracing::InfrastructureObserver,
) -> BackendResult<SchedulerServices> {
    let config = settings.scheduler_config()?;
    let pool = database.raw_pool().clone();
    let executor_epoch = database.next_id();
    let repository = Arc::new(StorageSchedulerRepository::new(database));
    let catalog = StaticTaskCatalog::try_new([
        HttpRequestTask::descriptor(),
        RefreshConfigCacheTask::descriptor(),
        RefreshDictCacheTask::descriptor(),
        SystemLogCleanupTask::descriptor(),
        PurgeTrashTask::descriptor(),
        CleanupUploadSessionsTask::descriptor(),
    ])?;
    let assembly = RuntimeAssembly {
        config: &config,
        repository: repository.clone(),
        catalog: catalog.clone(),
        system: system.clone(),
        logs,
        retention,
        file_cleanup,
        observer,
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
        use_case: service.clone(),
        audited: service,
        export_config: Arc::new(RuntimeSchedulerConfig::new(system)),
        runtime,
    })
}

fn runtime_parts(assembly: RuntimeAssembly<'_>) -> BackendResult<SchedulerRuntimeParts> {
    let context = TaskExecutionContext {
        system_cache: Arc::new(SchedulerSystemCacheAdapter { system: assembly.system }),
        http_client: Arc::new(ReqwestHttpTaskClient::new(scheduler_http_client(assembly.config)?, assembly.observer)),
        system_log_cleanup: Arc::new(SchedulerSystemLogCleanupAdapter {
            logs: assembly.logs,
            retention: assembly.retention,
        }),
        file_cleanup: Arc::new(SchedulerFileCleanupAdapter { files: assembly.file_cleanup }),
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

fn scheduler_service(repository: Arc<StorageSchedulerRepository>, catalog: Arc<StaticTaskCatalog>) -> Arc<SchedulerService> {
    Arc::new(SchedulerService::new(SchedulerServiceParts {
        query: repository.clone(),
        commands: repository.clone(),
        audited_commands: repository.clone(),
        catalog,
        clock: repository,
    }))
}

fn scheduler_http_client(config: &SchedulerSettings) -> BackendResult<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_millis(config.http_client.request_timeout_ms))
        .build()?)
}

fn cache_refresh_error(kind: CacheRefreshKind, error: SystemError) -> TaskExecutionFailure {
    cache_refresh_failure(kind, format!("scheduler cache refresh failed: {error}"))
}

fn system_log_cleanup_failure(error: observability::application::ObservabilityError) -> TaskExecutionFailure {
    let diagnostic = format!("system log cleanup failed: {error}");
    match error {
        observability::application::ObservabilityError::PartialCleanup { deleted, batches, .. } => TaskExecutionFailure::new(
            kernel::error::LocalizedError::new("errors.scheduler.task_system_log_cleanup_failed"),
            diagnostic,
        )
        .with_detail(SystemLogCleanupReport::new(deleted, batches)),
        other => TaskExecutionFailure::new(
            kernel::error::LocalizedError::new("errors.scheduler.task_system_log_cleanup_failed"),
            format!("system log cleanup failed: {other}"),
        ),
    }
}

fn system_log_filter(filter: SystemLogCleanupFilter) -> SystemLogFilter {
    SystemLogFilter {
        keyword: filter.keyword,
        levels: filter.levels.into_iter().map(observability_level).collect(),
        target: filter.target,
        begin_time: Some(filter.begin_time),
        end_time: Some(filter.end_time),
    }
}

fn observability_level(level: SystemLogCleanupLevel) -> SystemLogLevel {
    match level {
        SystemLogCleanupLevel::Trace => SystemLogLevel::Trace,
        SystemLogCleanupLevel::Debug => SystemLogLevel::Debug,
        SystemLogCleanupLevel::Info => SystemLogLevel::Info,
        SystemLogCleanupLevel::Warn => SystemLogLevel::Warn,
        SystemLogCleanupLevel::Error => SystemLogLevel::Error,
    }
}
