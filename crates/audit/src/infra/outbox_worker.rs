use std::{sync::Arc, time::Duration};

use client_info::IpLocationResolver;
use metrics::counter;
use time::OffsetDateTime;
use tokio::sync::watch;
use uuid::Uuid;

use crate::application::{AuditError, AuditResult};

use super::outbox_repository::{ClaimedAuditEvent, StorageAuditOutboxRepository, audit_location};

const LOCATION_ENRICHMENT_FAILED: &str = "location_enrichment_failed";
const PROJECTION_FAILED: &str = "projection_failed";
const LOCATION_ENRICHMENT_FAILURES_TOTAL: &str = "audit_outbox_location_enrichment_failures_total";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuditOutboxConfig {
    pub worker_count: usize,
    pub claim_batch_size: usize,
    pub poll_interval: Duration,
    pub lease_duration: Duration,
    pub retry_delay: Duration,
    pub cleanup_interval: Duration,
    pub cleanup_batch_size: usize,
    pub processed_retention: Duration,
}

impl AuditOutboxConfig {
    fn validate(self) -> AuditResult<Self> {
        validate_batch_settings(self)?;
        validate_duration_settings(self)?;
        Ok(self)
    }
}

fn validate_batch_settings(config: AuditOutboxConfig) -> AuditResult<()> {
    if config.worker_count == 0 || config.claim_batch_size == 0 || config.cleanup_batch_size == 0 {
        return Err(AuditError::Infrastructure("audit outbox worker and batch settings must be positive".into()));
    }
    if i64::try_from(config.claim_batch_size).is_err() || i64::try_from(config.cleanup_batch_size).is_err() {
        return Err(AuditError::Infrastructure("audit outbox batch settings exceed the supported range".into()));
    }
    Ok(())
}

fn validate_duration_settings(config: AuditOutboxConfig) -> AuditResult<()> {
    let durations = [
        config.poll_interval,
        config.lease_duration,
        config.retry_delay,
        config.cleanup_interval,
        config.processed_retention,
    ];
    if durations.iter().any(Duration::is_zero) {
        return Err(AuditError::Infrastructure("audit outbox duration settings must be positive".into()));
    }
    if durations.into_iter().any(|duration| time::Duration::try_from(duration).is_err()) {
        return Err(AuditError::Infrastructure("audit outbox duration settings exceed the supported range".into()));
    }
    Ok(())
}

pub struct AuditOutboxRuntimeParts {
    pub repository: Arc<StorageAuditOutboxRepository>,
    pub location_resolver: Arc<dyn IpLocationResolver>,
    pub config: AuditOutboxConfig,
}

/// Keeps the outbox worker shutdown channel alive for the runtime lifetime.
#[must_use = "retain this handle in the composition root while audit outbox workers must run"]
#[derive(Clone)]
pub struct AuditOutboxRuntimeHandle {
    shutdown: watch::Sender<bool>,
}

impl AuditOutboxRuntimeHandle {
    pub fn shutdown(&self) {
        self.shutdown.send_replace(true);
    }
}

pub fn start_audit_outbox_runtime(parts: AuditOutboxRuntimeParts) -> AuditResult<AuditOutboxRuntimeHandle> {
    let config = parts.config.validate()?;
    let worker = Arc::new(AuditOutboxWorker {
        repository: parts.repository,
        location_resolver: parts.location_resolver,
        config,
    });
    let (shutdown, receiver) = watch::channel(false);
    start_workers(worker.clone(), receiver.clone());
    tokio::spawn(run_cleanup(worker, receiver));
    Ok(AuditOutboxRuntimeHandle { shutdown })
}

struct AuditOutboxWorker {
    repository: Arc<StorageAuditOutboxRepository>,
    location_resolver: Arc<dyn IpLocationResolver>,
    config: AuditOutboxConfig,
}

impl AuditOutboxWorker {
    async fn run_once(&self, worker_id: &str) -> AuditResult<usize> {
        let now = OffsetDateTime::now_utc();
        let lease_until = add_duration(now, self.config.lease_duration)?;
        let retry_at = add_duration(now, self.config.retry_delay)?;
        let limit = i64::try_from(self.config.claim_batch_size)
            .map_err(|error| AuditError::Infrastructure(format!("audit outbox claim batch conversion failed: {error}")))?;
        let claimed = self
            .repository
            .claim(super::outbox_repository::ClaimOptions {
                lease_token: worker_id,
                limit,
                lease_until,
                retry_at,
            })
            .await?;
        for event in &claimed {
            self.project(event, retry_at).await?;
        }
        Ok(claimed.len())
    }

    async fn project(&self, claimed: &ClaimedAuditEvent, retry_at: OffsetDateTime) -> AuditResult<()> {
        let location = resolve_location(self.location_resolver.as_ref(), claimed).await;
        match self.repository.complete(claimed, location).await {
            Ok(_) => Ok(()),
            Err(error) => {
                trace_retry(claimed, PROJECTION_FAILED, &error);
                self.repository.retry(claimed, retry_at, PROJECTION_FAILED).await
            }
        }
    }

    async fn cleanup_once(&self) -> AuditResult<u64> {
        let retention = time::Duration::try_from(self.config.processed_retention)
            .map_err(|error| AuditError::Infrastructure(format!("audit outbox retention conversion failed: {error}")))?;
        let limit = i64::try_from(self.config.cleanup_batch_size)
            .map_err(|error| AuditError::Infrastructure(format!("audit outbox cleanup batch conversion failed: {error}")))?;
        let older_than = OffsetDateTime::now_utc()
            .checked_sub(retention)
            .ok_or_else(|| AuditError::Infrastructure("audit outbox retention exceeds timestamp range".into()))?;
        self.repository.cleanup(older_than, limit).await
    }
}

async fn resolve_location(location_resolver: &dyn IpLocationResolver, claimed: &ClaimedAuditEvent) -> crate::domain::AuditLocation {
    match location_resolver.resolve_ip_location(claimed.ip_address()).await {
        Ok(location) => audit_location(location),
        Err(error) => {
            trace_location_resolution_failure(claimed, &error);
            counter!(LOCATION_ENRICHMENT_FAILURES_TOTAL, "stream" => claimed.stream().code()).increment(1);
            crate::domain::AuditLocation::Unknown
        }
    }
}

fn start_workers(worker: Arc<AuditOutboxWorker>, receiver: watch::Receiver<bool>) {
    for _ in 0..worker.config.worker_count {
        let worker = worker.clone();
        let receiver = receiver.clone();
        tokio::spawn(run_worker(worker, receiver));
    }
}

async fn run_worker(worker: Arc<AuditOutboxWorker>, mut shutdown: watch::Receiver<bool>) {
    let worker_id = worker_lease_token();
    loop {
        if shutdown_requested(&shutdown) {
            return;
        }
        let delay = match worker.run_once(&worker_id).await {
            Ok(0) => worker.config.poll_interval,
            Ok(_) => continue,
            Err(error) => {
                hook_tracing::error_with_fields!("audit outbox worker failed", &error, worker_id = worker_id, reason = "worker_cycle_failed");
                worker.config.retry_delay
            }
        };
        if wait_for(delay, &mut shutdown).await {
            return;
        }
    }
}

/// Produces the canonical UUID token stored in `audit_outbox.lease_token`.
fn worker_lease_token() -> String {
    Uuid::now_v7().to_string()
}

async fn run_cleanup(worker: Arc<AuditOutboxWorker>, mut shutdown: watch::Receiver<bool>) {
    loop {
        if wait_for(worker.config.cleanup_interval, &mut shutdown).await {
            return;
        }
        if let Err(error) = worker.cleanup_once().await {
            hook_tracing::error_with_fields!("audit outbox cleanup failed", &error, reason = "cleanup_failed");
        }
    }
}

async fn wait_for(delay: Duration, shutdown: &mut watch::Receiver<bool>) -> bool {
    tokio::select! {
        () = tokio::time::sleep(delay) => false,
        result = shutdown.changed() => result.is_err() || *shutdown.borrow(),
    }
}

fn shutdown_requested(shutdown: &watch::Receiver<bool>) -> bool {
    *shutdown.borrow() || shutdown.has_changed().is_err()
}

fn add_duration(now: OffsetDateTime, value: Duration) -> AuditResult<OffsetDateTime> {
    let duration = time::Duration::try_from(value).map_err(|error| AuditError::Infrastructure(format!("audit outbox duration conversion failed: {error}")))?;
    now.checked_add(duration)
        .ok_or_else(|| AuditError::Infrastructure("audit outbox duration exceeds timestamp range".into()))
}

fn trace_retry<E: std::error::Error + ?Sized>(claimed: &ClaimedAuditEvent, reason: &'static str, error: &E) {
    hook_tracing::error_with_fields!(
        "audit outbox projection deferred",
        error,
        outbox_id = claimed.id,
        stream = claimed.stream().code(),
        reason = reason
    );
}

fn trace_location_resolution_failure<E: std::error::Error + ?Sized>(claimed: &ClaimedAuditEvent, error: &E) {
    hook_tracing::error_with_fields!(
        "audit outbox location resolution failed; projecting unknown location",
        error,
        outbox_id = claimed.id,
        stream = claimed.stream().code(),
        reason = LOCATION_ENRICHMENT_FAILED
    );
}

#[cfg(test)]
mod tests;
