use kernel::{excel::TemporaryXlsxFile, pagination::CursorDirection, runtime_config::ExportBatchConfig};
use tokio::{sync::mpsc, task::JoinHandle};
use types::http::{Locale, translate_message};

use crate::{
    application::{ObservabilityError, ObservabilityResult, SystemLogBoundary, SystemLogCursorQuery, SystemLogExportSession, SystemLogSnapshot},
    domain::{SystemLogDetail, SystemLogFilter},
};

use super::SystemLogApiState;

mod blocking_writer;
mod export_xlsx;
use blocking_writer::write_system_log_xlsx;

const HEADER_KEYS: &[&str] = &[
    "excel.observability.system.headers.id",
    "excel.observability.system.headers.time",
    "excel.observability.system.headers.level",
    "excel.observability.system.headers.target",
    "excel.observability.system.headers.message",
    "excel.observability.system.headers.fields",
];

pub struct ExportRequest<'a> {
    pub state: &'a SystemLogApiState,
    pub filter: SystemLogFilter,
    pub batch: ExportBatchConfig,
    pub locale: Locale,
}

pub async fn system_logs(request: ExportRequest<'_>) -> ObservabilityResult<TemporaryXlsxFile> {
    let ExportRequest { state, filter, batch, locale } = request;
    let headers = HEADER_KEYS.iter().map(|key| translate_message(locale, key)).collect::<Vec<_>>();
    let mut export = state.logs.begin_export().await?;
    let mut cursor = ExportCursor::new(batch.page_size)?;
    let (sender, writer) = start_writer(batch.page_size, translate_message(locale, "excel.observability.system.sheet"), headers, locale)?;
    let page_result = queue_export_pages(export.as_mut(), filter, &mut cursor, &sender).await;
    drop(sender);
    let session_result = finish_export_session(export, page_result).await;
    let writer_result = finish_writer(writer).await;
    complete_export(session_result, writer_result)
}

fn start_writer(
    capacity: u64,
    sheet_name: String,
    headers: Vec<String>,
    locale: Locale,
) -> ObservabilityResult<(mpsc::Sender<SystemLogDetail>, JoinHandle<ObservabilityResult<TemporaryXlsxFile>>)> {
    let capacity = usize::try_from(capacity)
        .map_err(|error| ObservabilityError::Infrastructure(format!("system log export writer capacity conversion failed: {error}")))?;
    let (sender, receiver) = mpsc::channel(capacity);
    let writer = tokio::task::spawn_blocking(move || write_system_log_xlsx(receiver, sheet_name, headers, locale));
    Ok((sender, writer))
}

async fn queue_export_pages(
    export: &mut dyn SystemLogExportSession,
    filter: SystemLogFilter,
    cursor: &mut ExportCursor,
    sender: &mpsc::Sender<SystemLogDetail>,
) -> ObservabilityResult<()> {
    loop {
        let page = export.page(filter.clone(), cursor.request()).await?;
        if page.items.is_empty() {
            return Ok(());
        }
        let boundary = SystemLogBoundary::from_summary(&page.items.last().ok_or_else(missing_export_row)?.summary);
        let has_more = cursor.advance(boundary, page.snapshot.clone(), page.has_next)?;
        queue_page(sender, page.items).await?;
        if !has_more {
            return Ok(());
        }
    }
}

async fn queue_page(sender: &mpsc::Sender<SystemLogDetail>, items: Vec<SystemLogDetail>) -> ObservabilityResult<()> {
    for item in items {
        sender
            .send(item)
            .await
            .map_err(|_| ObservabilityError::Infrastructure("system log XLSX writer stopped before export completed".into()))?;
    }
    Ok(())
}

async fn finish_writer(writer: JoinHandle<ObservabilityResult<TemporaryXlsxFile>>) -> ObservabilityResult<TemporaryXlsxFile> {
    writer
        .await
        .map_err(|error| ObservabilityError::Infrastructure(format!("system log XLSX writer task failed: {error}")))?
}

async fn finish_export_session(export: Box<dyn SystemLogExportSession>, page_result: ObservabilityResult<()>) -> ObservabilityResult<()> {
    page_result?;
    export.finish().await
}

fn complete_export(session_result: ObservabilityResult<()>, writer_result: ObservabilityResult<TemporaryXlsxFile>) -> ObservabilityResult<TemporaryXlsxFile> {
    match (session_result, writer_result) {
        (Ok(()), Ok(artifact)) => Ok(artifact),
        (Err(session), Ok(_)) => Err(session),
        (Ok(()), Err(writer)) => Err(writer),
        (Err(session), Err(writer)) => Err(ObservabilityError::Infrastructure(format!(
            "system log export session failed: {session}; XLSX writer failed: {writer}"
        ))),
    }
}

#[derive(Debug)]
struct ExportCursor {
    limit: u64,
    boundary: Option<SystemLogBoundary>,
    snapshot: Option<SystemLogSnapshot>,
}

impl ExportCursor {
    fn new(limit: u64) -> ObservabilityResult<Self> {
        if limit == 0 {
            return Err(ObservabilityError::Infrastructure(
                "system log export page size is zero after validation".into(),
            ));
        }
        Ok(Self {
            limit,
            boundary: None,
            snapshot: None,
        })
    }

    fn request(&self) -> SystemLogCursorQuery {
        SystemLogCursorQuery {
            limit: self.limit,
            direction: CursorDirection::Next,
            boundary: self.boundary.clone(),
            snapshot: self.snapshot.clone(),
        }
    }

    fn advance(&mut self, boundary: SystemLogBoundary, snapshot: Option<SystemLogSnapshot>, has_more: bool) -> ObservabilityResult<bool> {
        self.snapshot = Some(snapshot.ok_or_else(|| ObservabilityError::Infrastructure("system log export page is missing its snapshot".into()))?);
        self.boundary = Some(boundary);
        Ok(has_more)
    }
}

fn missing_export_row() -> ObservabilityError {
    ObservabilityError::Infrastructure("system log export page lost its last row".into())
}

#[cfg(test)]
mod tests {
    use crate::application::{ObservabilityError, SystemLogSnapshot};

    use super::{ExportCursor, HEADER_KEYS, complete_export};

    #[test]
    fn export_cursor_keeps_the_snapshot_across_batches() {
        let mut cursor = ExportCursor::new(2).unwrap();
        let snapshot = SystemLogSnapshot::new(1);
        let boundary = crate::application::SystemLogBoundary {
            occurred_at_nanos: "0".into(),
            id: "last".into(),
        };

        assert!(cursor.advance(boundary.clone(), Some(snapshot.clone()), true).unwrap());
        assert_eq!(cursor.request().boundary, Some(boundary));
        assert_eq!(cursor.request().snapshot, Some(snapshot));
    }

    #[test]
    fn export_headers_keep_all_fixed_columns() {
        assert_eq!(HEADER_KEYS.len(), 6);
    }

    #[test]
    fn export_reports_both_session_and_writer_failures() {
        let result = complete_export(
            Err(ObservabilityError::Infrastructure("session failure".into())),
            Err(ObservabilityError::Infrastructure("writer failure".into())),
        );
        let Err(error) = result else {
            panic!("two failures must fail the export");
        };

        assert!(error.to_string().contains("session failure"));
        assert!(error.to_string().contains("writer failure"));
    }
}
