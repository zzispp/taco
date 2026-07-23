use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::{
    application::{
        ObservabilityError, ObservabilityResult, SystemLogCursorQuery, SystemLogCursorSlice, SystemLogExportLayout, SystemLogExportRequest,
        SystemLogExportService, SystemLogExportSession, SystemLogExportSlice, SystemLogExportUseCase, SystemLogExportWriter, SystemLogExportWriterFactory,
        SystemLogExportWriterRequest, SystemLogRepository,
    },
    domain::{NewSystemLog, SystemLogDetail, SystemLogFilter},
};
use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use kernel::runtime_config::ExportBatchConfig;

use super::delete_all_matching;

#[tokio::test]
async fn manual_cleanup_uses_the_configured_batch_size_until_the_filter_is_empty() {
    let repository = CleanupRepository::new([Ok(2), Ok(1), Ok(0)]);

    let report = delete_all_matching(&repository, SystemLogFilter::default(), 2400).await.unwrap();

    assert_eq!(report.deleted, 3);
    assert_eq!(report.batches, 2);
    assert_eq!(repository.limits(), vec![2400, 2400, 2400]);
}

#[tokio::test]
async fn manual_cleanup_failure_keeps_committed_batch_totals() {
    let repository = CleanupRepository::new([Ok(2), Err(ObservabilityError::Infrastructure("database unavailable".into()))]);

    let error = delete_all_matching(&repository, SystemLogFilter::default(), 1000).await.unwrap_err();

    assert!(matches!(error, ObservabilityError::PartialCleanup { deleted: 2, batches: 1, .. }));
}

#[tokio::test]
async fn export_service_aborts_the_snapshot_when_the_injected_writer_factory_fails() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let service = SystemLogExportService::new(
        Arc::new(ExportRepository { events: events.clone() }),
        Arc::new(FailingWriterFactory { events: events.clone() }),
    );

    let result = service
        .export_xlsx(SystemLogExportRequest {
            filter: SystemLogFilter::default(),
            batch: ExportBatchConfig { page_size: 2 },
            layout: SystemLogExportLayout::new(
                "logs".into(),
                ["id", "time", "level", "target", "message", "fields"].map(str::to_owned),
                ["id", "kind", "part", "content"].map(str::to_owned),
            ),
        })
        .await;
    let Err(error) = result else {
        panic!("export unexpectedly succeeded");
    };

    assert!(error.to_string().contains("writer creation failure"));
    assert_eq!(*events.lock().unwrap(), ["begin_export", "factory_start", "session_abort"]);
}

struct CleanupRepository {
    batches: Mutex<VecDeque<ObservabilityResult<u64>>>,
    limits: Mutex<Vec<u64>>,
}

impl CleanupRepository {
    fn new(batches: impl IntoIterator<Item = ObservabilityResult<u64>>) -> Self {
        Self {
            batches: Mutex::new(batches.into_iter().collect()),
            limits: Mutex::new(Vec::new()),
        }
    }

    fn limits(&self) -> Vec<u64> {
        self.limits.lock().unwrap().clone()
    }
}

#[async_trait]
impl SystemLogRepository for CleanupRepository {
    async fn insert_batch(&self, _: &[NewSystemLog]) -> ObservabilityResult<()> {
        unreachable!()
    }

    async fn page(&self, _: SystemLogFilter, _: SystemLogCursorQuery) -> ObservabilityResult<SystemLogCursorSlice> {
        unreachable!()
    }

    async fn find(&self, _: &str) -> ObservabilityResult<Option<SystemLogDetail>> {
        unreachable!()
    }

    async fn delete_ids_with_audit(&self, _: &[String], _: &AuditOutboxRecord) -> ObservabilityResult<()> {
        unreachable!()
    }

    async fn count(&self, _: SystemLogFilter) -> ObservabilityResult<u64> {
        unreachable!()
    }

    async fn delete_filtered_batch(&self, _: SystemLogFilter, limit: u64) -> ObservabilityResult<u64> {
        self.limits.lock().unwrap().push(limit);
        self.batches.lock().unwrap().pop_front().unwrap()
    }

    async fn begin_export(&self) -> ObservabilityResult<Box<dyn SystemLogExportSession>> {
        unreachable!()
    }
}

struct ExportRepository {
    events: Arc<Mutex<Vec<&'static str>>>,
}

#[async_trait]
impl SystemLogRepository for ExportRepository {
    async fn insert_batch(&self, _: &[NewSystemLog]) -> ObservabilityResult<()> {
        unreachable!()
    }

    async fn page(&self, _: SystemLogFilter, _: SystemLogCursorQuery) -> ObservabilityResult<SystemLogCursorSlice> {
        unreachable!()
    }

    async fn find(&self, _: &str) -> ObservabilityResult<Option<SystemLogDetail>> {
        unreachable!()
    }

    async fn delete_ids_with_audit(&self, _: &[String], _: &AuditOutboxRecord) -> ObservabilityResult<()> {
        unreachable!()
    }

    async fn count(&self, _: SystemLogFilter) -> ObservabilityResult<u64> {
        unreachable!()
    }

    async fn delete_filtered_batch(&self, _: SystemLogFilter, _: u64) -> ObservabilityResult<u64> {
        unreachable!()
    }

    async fn begin_export(&self) -> ObservabilityResult<Box<dyn SystemLogExportSession>> {
        self.events.lock().unwrap().push("begin_export");
        Ok(Box::new(AbortRecordingSession { events: self.events.clone() }))
    }
}

struct FailingWriterFactory {
    events: Arc<Mutex<Vec<&'static str>>>,
}

impl SystemLogExportWriterFactory for FailingWriterFactory {
    fn start(&self, request: SystemLogExportWriterRequest) -> ObservabilityResult<Box<dyn SystemLogExportWriter>> {
        assert_eq!(request.capacity, 2);
        self.events.lock().unwrap().push("factory_start");
        Err(ObservabilityError::Infrastructure("writer creation failure".into()))
    }
}

struct AbortRecordingSession {
    events: Arc<Mutex<Vec<&'static str>>>,
}

#[async_trait]
impl SystemLogExportSession for AbortRecordingSession {
    async fn page(&mut self, _: SystemLogFilter, _: SystemLogCursorQuery) -> ObservabilityResult<SystemLogExportSlice> {
        unreachable!()
    }

    async fn finish(self: Box<Self>) -> ObservabilityResult<()> {
        unreachable!()
    }

    async fn abort(self: Box<Self>) -> ObservabilityResult<()> {
        self.events.lock().unwrap().push("session_abort");
        Ok(())
    }
}
