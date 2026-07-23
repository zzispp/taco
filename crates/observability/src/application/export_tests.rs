use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use kernel::excel::{StreamingXlsxWriter, TemporaryXlsxFile};

use crate::{
    application::{ObservabilityError, ObservabilityResult, SystemLogExportSession, SystemLogExportSlice, SystemLogSnapshot},
    domain::{SystemLogDetail, SystemLogFilter, SystemLogLevel, SystemLogSummary},
};

use super::export::{ExportCursor, run_open_export, run_started_export};
use crate::application::SystemLogExportWriter;

#[tokio::test]
async fn successful_export_finishes_the_snapshot_after_the_writer() {
    let events = events();
    let session = FakeSession::new([Ok(page(false))], events.clone());
    let writer = FakeWriter::succeeds(events.clone());

    let artifact = run_open_export(Box::new(session), SystemLogFilter::default(), ExportCursor::new(2).unwrap(), Box::new(writer))
        .await
        .unwrap();

    assert!(artifact.content_length() > 0);
    assert_eq!(recorded(&events), ["page", "append", "writer_finish", "session_finish"]);
}

#[tokio::test]
async fn page_failure_aborts_before_finishing_the_writer() {
    let events = events();
    let session = FakeSession::new([Err(failure("page failure"))], events.clone());
    let writer = FakeWriter::succeeds(events.clone());

    let error = run_open_export(Box::new(session), SystemLogFilter::default(), ExportCursor::new(2).unwrap(), Box::new(writer)).await;
    let error = error_of(error);

    assert!(error.to_string().contains("page failure"));
    assert_eq!(recorded(&events), ["page", "session_abort", "writer_finish"]);
}

#[tokio::test]
async fn writer_append_failure_aborts_and_reports_the_worker_failure() {
    let events = events();
    let session = FakeSession::new([Ok(page(false))], events.clone());
    let writer = FakeWriter::fails_append_and_finish(events.clone());

    let error = run_open_export(Box::new(session), SystemLogFilter::default(), ExportCursor::new(2).unwrap(), Box::new(writer)).await;
    let error = error_of(error);
    let message = error.to_string();

    assert!(message.contains("append failure"));
    assert!(message.contains("writer failure"));
    assert_eq!(recorded(&events), ["page", "append", "session_abort", "writer_finish"]);
}

#[tokio::test]
async fn writer_finish_failure_aborts_instead_of_committing() {
    let events = events();
    let session = FakeSession::new([Ok(page(false))], events.clone());
    let writer = FakeWriter::fails_finish(events.clone());

    let error = run_open_export(Box::new(session), SystemLogFilter::default(), ExportCursor::new(2).unwrap(), Box::new(writer)).await;
    let error = error_of(error);

    assert!(error.to_string().contains("writer failure"));
    assert_eq!(recorded(&events), ["page", "append", "writer_finish", "session_abort"]);
}

#[tokio::test]
async fn writer_creation_failure_after_begin_aborts_the_snapshot() {
    let events = events();
    let session = FakeSession::new([], events.clone());

    let error = run_started_export(
        Box::new(session),
        SystemLogFilter::default(),
        ExportCursor::new(2).unwrap(),
        Err(failure("writer creation failure")),
    )
    .await;
    let error = error_of(error);

    assert!(error.to_string().contains("writer creation failure"));
    assert_eq!(recorded(&events), ["session_abort"]);
}

#[tokio::test]
async fn session_finish_failure_is_returned_after_the_writer_succeeds() {
    let events = events();
    let session = FakeSession::new([Ok(page(false))], events.clone()).with_finish_failure();
    let writer = FakeWriter::succeeds(events.clone());

    let error = run_open_export(Box::new(session), SystemLogFilter::default(), ExportCursor::new(2).unwrap(), Box::new(writer)).await;
    let error = error_of(error);

    assert!(error.to_string().contains("finish failure"));
    assert_eq!(recorded(&events), ["page", "append", "writer_finish", "session_finish"]);
}

#[tokio::test]
async fn export_reports_page_writer_and_rollback_failures_together() {
    let events = events();
    let session = FakeSession::new([Err(failure("page failure"))], events.clone()).with_abort_failure();
    let writer = FakeWriter::fails_finish(events);

    let error = run_open_export(Box::new(session), SystemLogFilter::default(), ExportCursor::new(2).unwrap(), Box::new(writer)).await;
    let error = error_of(error);
    let message = error.to_string();

    assert!(message.contains("page failure"));
    assert!(message.contains("writer failure"));
    assert!(message.contains("rollback failure"));
}

type Events = Arc<Mutex<Vec<&'static str>>>;

struct FakeSession {
    pages: VecDeque<ObservabilityResult<SystemLogExportSlice>>,
    events: Events,
    abort_fails: bool,
    finish_fails: bool,
}

impl FakeSession {
    fn new(pages: impl IntoIterator<Item = ObservabilityResult<SystemLogExportSlice>>, events: Events) -> Self {
        Self {
            pages: pages.into_iter().collect(),
            events,
            abort_fails: false,
            finish_fails: false,
        }
    }

    fn with_abort_failure(mut self) -> Self {
        self.abort_fails = true;
        self
    }

    fn with_finish_failure(mut self) -> Self {
        self.finish_fails = true;
        self
    }
}

#[async_trait]
impl SystemLogExportSession for FakeSession {
    async fn page(&mut self, _: SystemLogFilter, _: super::SystemLogCursorQuery) -> ObservabilityResult<SystemLogExportSlice> {
        record(&self.events, "page");
        self.pages.pop_front().unwrap_or_else(|| panic!("unexpected export page request"))
    }

    async fn finish(self: Box<Self>) -> ObservabilityResult<()> {
        record(&self.events, "session_finish");
        if self.finish_fails { Err(failure("finish failure")) } else { Ok(()) }
    }

    async fn abort(self: Box<Self>) -> ObservabilityResult<()> {
        record(&self.events, "session_abort");
        if self.abort_fails { Err(failure("rollback failure")) } else { Ok(()) }
    }
}

struct FakeWriter {
    events: Events,
    append_fails: bool,
    finish_fails: bool,
}

impl FakeWriter {
    fn succeeds(events: Events) -> Self {
        Self {
            events,
            append_fails: false,
            finish_fails: false,
        }
    }

    fn fails_finish(events: Events) -> Self {
        Self {
            events,
            append_fails: false,
            finish_fails: true,
        }
    }

    fn fails_append_and_finish(events: Events) -> Self {
        Self {
            events,
            append_fails: true,
            finish_fails: true,
        }
    }
}

#[async_trait]
impl SystemLogExportWriter for FakeWriter {
    async fn append(&mut self, _: SystemLogDetail) -> ObservabilityResult<()> {
        record(&self.events, "append");
        if self.append_fails { Err(failure("append failure")) } else { Ok(()) }
    }

    async fn finish(self: Box<Self>) -> ObservabilityResult<TemporaryXlsxFile> {
        record(&self.events, "writer_finish");
        if self.finish_fails { Err(failure("writer failure")) } else { artifact() }
    }
}

fn page(has_next: bool) -> SystemLogExportSlice {
    SystemLogExportSlice {
        items: vec![detail("log-1")],
        snapshot: Some(SystemLogSnapshot::new(1)),
        has_next,
    }
}

fn detail(id: &str) -> SystemLogDetail {
    SystemLogDetail {
        summary: SystemLogSummary {
            id: id.into(),
            occurred_at: time::OffsetDateTime::UNIX_EPOCH,
            level: SystemLogLevel::Info,
            target: "test::export".into(),
            message: "message".into(),
        },
        fields: serde_json::json!({}),
    }
}

fn artifact() -> ObservabilityResult<TemporaryXlsxFile> {
    StreamingXlsxWriter::new("test", &["id"])
        .and_then(StreamingXlsxWriter::finish)
        .map_err(ObservabilityError::Infrastructure)
}

fn failure(message: &str) -> ObservabilityError {
    ObservabilityError::Infrastructure(message.into())
}

fn events() -> Events {
    Arc::new(Mutex::new(Vec::new()))
}

fn record(events: &Events, event: &'static str) {
    events.lock().unwrap().push(event);
}

fn recorded(events: &Events) -> Vec<&'static str> {
    events.lock().unwrap().clone()
}

fn error_of(result: ObservabilityResult<TemporaryXlsxFile>) -> ObservabilityError {
    match result {
        Ok(_) => panic!("export unexpectedly succeeded"),
        Err(error) => error,
    }
}
