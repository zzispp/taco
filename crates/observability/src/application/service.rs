use std::sync::Arc;

use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use kernel::{
    excel::TemporaryXlsxFile,
    pagination::{CursorPage, CursorPageRequest},
};

use crate::domain::{SystemLogDetail, SystemLogFilter, SystemLogSummary};

use super::{
    ObservabilityError, ObservabilityResult, SystemLogExportRequest, SystemLogExportUseCase, SystemLogExportWriterFactory, SystemLogRepository,
    SystemLogRetentionReport, SystemLogRetentionStore, SystemLogRetentionUseCase, SystemLogUseCase, localized, system_log_cursor_page, system_log_cursor_query,
};

#[derive(Clone)]
pub struct SystemLogService {
    repository: Arc<dyn SystemLogRepository>,
    retention_store: Arc<dyn SystemLogRetentionStore>,
}

impl SystemLogService {
    pub fn new(repository: Arc<dyn SystemLogRepository>, retention_store: Arc<dyn SystemLogRetentionStore>) -> Self {
        Self { repository, retention_store }
    }
}

#[async_trait]
impl SystemLogUseCase for SystemLogService {
    async fn page(&self, filter: SystemLogFilter, page: CursorPageRequest) -> ObservabilityResult<CursorPage<SystemLogSummary>> {
        validate_filter(&filter)?;
        let query = system_log_cursor_query(&filter, &page)?;
        let slice = self.repository.page(filter.clone(), query.clone()).await?;
        system_log_cursor_page(&filter, &query, slice)
    }

    async fn detail(&self, id: &str) -> ObservabilityResult<SystemLogDetail> {
        self.repository.find(id).await?.ok_or(ObservabilityError::NotFound)
    }

    async fn delete_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> ObservabilityResult<()> {
        self.repository.delete_ids_with_audit(&validate_ids(ids)?, &audit).await
    }

    async fn count(&self, filter: SystemLogFilter) -> ObservabilityResult<u64> {
        validate_time_range(&filter)?;
        self.repository.count(filter).await
    }

    async fn delete_filtered(&self, filter: SystemLogFilter, batch_size: u64) -> ObservabilityResult<SystemLogRetentionReport> {
        validate_required_time_range(&filter)?;
        delete_all_matching(self.repository.as_ref(), filter, batch_size).await
    }
}

#[derive(Clone)]
pub struct SystemLogExportService {
    repository: Arc<dyn SystemLogRepository>,
    writer_factory: Arc<dyn SystemLogExportWriterFactory>,
}

impl SystemLogExportService {
    pub fn new(repository: Arc<dyn SystemLogRepository>, writer_factory: Arc<dyn SystemLogExportWriterFactory>) -> Self {
        Self { repository, writer_factory }
    }
}

#[async_trait]
impl SystemLogExportUseCase for SystemLogExportService {
    async fn export_xlsx(&self, request: SystemLogExportRequest) -> ObservabilityResult<TemporaryXlsxFile> {
        super::export::system_logs(self.repository.as_ref(), self.writer_factory.as_ref(), request).await
    }
}

async fn delete_all_matching(repository: &dyn SystemLogRepository, filter: SystemLogFilter, batch_size: u64) -> ObservabilityResult<SystemLogRetentionReport> {
    if batch_size == 0 {
        return Err(ObservabilityError::InvalidInput(localized("errors.observability.invalid_cleanup_batch_size")));
    }
    let mut report = SystemLogRetentionReport { deleted: 0, batches: 0 };
    loop {
        let deleted = match repository.delete_filtered_batch(filter.clone(), batch_size).await {
            Ok(deleted) => deleted,
            Err(error) => return Err(partial_cleanup_error(report, error)),
        };
        if deleted == 0 {
            return Ok(report);
        }
        report.deleted = report
            .deleted
            .checked_add(deleted)
            .ok_or_else(|| ObservabilityError::Infrastructure("system log manual cleanup deleted count overflow".into()))?;
        report.batches = report
            .batches
            .checked_add(1)
            .ok_or_else(|| ObservabilityError::Infrastructure("system log manual cleanup batch count overflow".into()))?;
    }
}

fn partial_cleanup_error(report: SystemLogRetentionReport, error: ObservabilityError) -> ObservabilityError {
    if report.deleted == 0 {
        return error;
    }
    ObservabilityError::partial_cleanup(report, error.to_string())
}

#[async_trait]
impl SystemLogRetentionUseCase for SystemLogService {
    async fn cleanup_expired(&self, retention_days: u64, boundary_batch_size: u64) -> ObservabilityResult<SystemLogRetentionReport> {
        super::retention::cleanup_expired(self.retention_store.as_ref(), retention_days, boundary_batch_size).await
    }
}

fn validate_filter(filter: &SystemLogFilter) -> ObservabilityResult<()> {
    validate_time_range(filter)
}

fn validate_time_range(filter: &SystemLogFilter) -> ObservabilityResult<()> {
    if filter.begin_time.zip(filter.end_time).is_some_and(|(begin, end)| begin > end) {
        return Err(ObservabilityError::InvalidInput(localized("errors.observability.invalid_date_range")));
    }
    Ok(())
}

fn validate_required_time_range(filter: &SystemLogFilter) -> ObservabilityResult<()> {
    validate_time_range(filter)?;
    if filter.begin_time.is_none() || filter.end_time.is_none() {
        return Err(ObservabilityError::InvalidInput(localized("errors.observability.time_range_required")));
    }
    Ok(())
}

fn validate_ids(ids: Vec<String>) -> ObservabilityResult<Vec<String>> {
    if ids.is_empty() || ids.iter().any(|id| id.trim().is_empty() || id.trim() != id) {
        return Err(ObservabilityError::InvalidInput(localized("errors.observability.invalid_ids")));
    }
    let unique = ids.iter().collect::<std::collections::HashSet<_>>();
    if unique.len() != ids.len() {
        return Err(ObservabilityError::InvalidInput(localized("errors.observability.invalid_ids")));
    }
    Ok(ids)
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
