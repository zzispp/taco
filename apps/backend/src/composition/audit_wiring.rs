use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use audit::{
    application::{AuditError, AuditRepository, AuditService, AuditUseCase, parse_export_batch_config},
    infra::{
        AuditOutboxConfig, AuditOutboxRuntimeHandle, AuditOutboxRuntimeParts, StorageAuditOutboxRepository, StorageAuditRepository, start_audit_outbox_runtime,
    },
};
use client_info::IpLocationResolver;
use configuration::Settings;
use constants::system_config::EXPORT_BATCH_CONFIG_KEY;
use kernel::runtime_config::{ExportBatchConfig, ExportConfigProvider};
use storage::Database;
use system::application::SystemUseCase;

use crate::BackendResult;

const SECONDS_PER_DAY: u64 = 86_400;

pub(super) struct AuditServices {
    pub use_case: Arc<dyn AuditUseCase>,
    pub outbox: Arc<StorageAuditOutboxRepository>,
    pub runtime: AuditOutboxRuntimeHandle,
    pub export_config: Arc<dyn ExportConfigProvider<Error = AuditError>>,
}

pub(super) struct AuditServiceParts {
    pub database: Database,
    pub system: Arc<dyn SystemUseCase>,
    pub location_resolver: Arc<dyn IpLocationResolver>,
    pub outbox: AuditOutboxConfig,
}

pub(super) fn build_audit_services(parts: AuditServiceParts) -> Result<AuditServices, AuditError> {
    let repository: Arc<dyn AuditRepository> = Arc::new(StorageAuditRepository::new(parts.database.clone()));
    let outbox = Arc::new(StorageAuditOutboxRepository::new(parts.database));
    let runtime = start_audit_outbox_runtime(AuditOutboxRuntimeParts {
        repository: outbox.clone(),
        location_resolver: parts.location_resolver,
        config: parts.outbox,
    })?;
    Ok(AuditServices {
        use_case: Arc::new(AuditService::new(repository)),
        outbox,
        runtime,
        export_config: Arc::new(RuntimeAuditConfig::new(parts.system)),
    })
}

pub(super) fn audit_outbox_config(settings: &Settings) -> BackendResult<AuditOutboxConfig> {
    let config = settings.audit_config()?.outbox;
    let retention_seconds = config
        .processed_retention_days
        .checked_mul(SECONDS_PER_DAY)
        .ok_or_else(|| std::io::Error::other("audit outbox retention days exceed duration range"))?;
    Ok(AuditOutboxConfig {
        worker_count: config.worker_count,
        claim_batch_size: config.claim_batch_size,
        poll_interval: Duration::from_millis(config.poll_interval_ms),
        lease_duration: Duration::from_millis(config.lease_duration_ms),
        retry_delay: Duration::from_millis(config.retry_delay_ms),
        cleanup_interval: Duration::from_millis(config.cleanup_interval_ms),
        cleanup_batch_size: config.cleanup_batch_size,
        processed_retention: Duration::from_secs(retention_seconds),
    })
}

#[derive(Clone)]
struct RuntimeAuditConfig {
    system: Arc<dyn SystemUseCase>,
}

impl RuntimeAuditConfig {
    fn new(system: Arc<dyn SystemUseCase>) -> Self {
        Self { system }
    }
}

#[async_trait]
impl ExportConfigProvider for RuntimeAuditConfig {
    type Error = AuditError;

    async fn export_batch_config(&self) -> Result<ExportBatchConfig, Self::Error> {
        let value = self.system.config_by_key(EXPORT_BATCH_CONFIG_KEY).await.map_err(system_error)?;
        parse_export_batch_config(&value)
    }
}

fn system_error(error: system::application::SystemError) -> AuditError {
    AuditError::Infrastructure(error.to_string())
}
