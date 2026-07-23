use kernel::{excel::TemporaryXlsxFile, pagination::CursorDirection, runtime_config::ExportBatchConfig};

use crate::domain::{SystemLogDetail, SystemLogFilter};

use super::{
    ObservabilityError, ObservabilityResult, SystemLogBoundary, SystemLogCursorQuery, SystemLogExportSession, SystemLogExportWriter,
    SystemLogExportWriterFactory, SystemLogExportWriterRequest, SystemLogRepository, SystemLogSnapshot,
};

pub struct SystemLogExportRequest {
    pub filter: SystemLogFilter,
    pub batch: ExportBatchConfig,
    pub layout: SystemLogExportLayout,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SystemLogExportLayout {
    sheet_name: String,
    headers: [String; 6],
    continuation_headers: [String; 4],
}

impl SystemLogExportLayout {
    pub fn new(sheet_name: String, headers: [String; 6], continuation_headers: [String; 4]) -> Self {
        Self {
            sheet_name,
            headers,
            continuation_headers,
        }
    }

    pub(crate) fn into_parts(self) -> (String, [String; 6], [String; 4]) {
        (self.sheet_name, self.headers, self.continuation_headers)
    }
}

pub(super) async fn system_logs(
    repository: &dyn SystemLogRepository,
    writer_factory: &dyn SystemLogExportWriterFactory,
    request: SystemLogExportRequest,
) -> ObservabilityResult<TemporaryXlsxFile> {
    let SystemLogExportRequest { filter, batch, layout } = request;
    let cursor = ExportCursor::new(batch.page_size)?;
    let capacity = usize::try_from(batch.page_size)
        .map_err(|error| ObservabilityError::Infrastructure(format!("system log export writer capacity conversion failed: {error}")))?;
    let export = repository.begin_export().await?;
    let writer = writer_factory.start(SystemLogExportWriterRequest { capacity, layout });
    run_started_export(export, filter, cursor, writer).await
}

pub(super) async fn run_started_export(
    export: Box<dyn SystemLogExportSession>,
    filter: SystemLogFilter,
    cursor: ExportCursor,
    writer: ObservabilityResult<Box<dyn SystemLogExportWriter>>,
) -> ObservabilityResult<TemporaryXlsxFile> {
    match writer {
        Ok(writer) => run_open_export(export, filter, cursor, writer).await,
        Err(writer_error) => abort_after_writer_failure(export, writer_error).await,
    }
}

pub(super) async fn run_open_export(
    mut export: Box<dyn SystemLogExportSession>,
    filter: SystemLogFilter,
    mut cursor: ExportCursor,
    mut writer: Box<dyn SystemLogExportWriter>,
) -> ObservabilityResult<TemporaryXlsxFile> {
    match queue_export_pages(export.as_mut(), filter, &mut cursor, writer.as_mut()).await {
        Ok(()) => finish_successful_export(export, writer).await,
        Err(page_error) => abort_failed_export(export, writer, page_error).await,
    }
}

async fn finish_successful_export(export: Box<dyn SystemLogExportSession>, writer: Box<dyn SystemLogExportWriter>) -> ObservabilityResult<TemporaryXlsxFile> {
    match writer.finish().await {
        Ok(artifact) => {
            export.finish().await?;
            Ok(artifact)
        }
        Err(writer_error) => abort_after_writer_failure(export, writer_error).await,
    }
}

async fn abort_failed_export(
    export: Box<dyn SystemLogExportSession>,
    writer: Box<dyn SystemLogExportWriter>,
    page_error: ObservabilityError,
) -> ObservabilityResult<TemporaryXlsxFile> {
    let abort_result = export.abort().await;
    let writer_result = writer.finish().await;
    let primary = combine_pipeline_failures(page_error, writer_result.err());
    Err(combine_abort_failure(primary, abort_result.err()))
}

async fn abort_after_writer_failure(export: Box<dyn SystemLogExportSession>, writer_error: ObservabilityError) -> ObservabilityResult<TemporaryXlsxFile> {
    let abort_error = export.abort().await.err();
    Err(combine_abort_failure(writer_error, abort_error))
}

fn combine_pipeline_failures(primary: ObservabilityError, writer: Option<ObservabilityError>) -> ObservabilityError {
    match writer {
        Some(writer) => ObservabilityError::Infrastructure(format!("system log export failed: {primary}; XLSX writer failed: {writer}")),
        None => primary,
    }
}

fn combine_abort_failure(primary: ObservabilityError, abort: Option<ObservabilityError>) -> ObservabilityError {
    match abort {
        Some(abort) => ObservabilityError::Infrastructure(format!("system log export failed: {primary}; snapshot rollback failed: {abort}")),
        None => primary,
    }
}

async fn queue_export_pages(
    export: &mut dyn SystemLogExportSession,
    filter: SystemLogFilter,
    cursor: &mut ExportCursor,
    writer: &mut dyn SystemLogExportWriter,
) -> ObservabilityResult<()> {
    loop {
        let page = export.page(filter.clone(), cursor.request()).await?;
        if page.items.is_empty() {
            return empty_page_result(page.has_next);
        }
        let boundary = SystemLogBoundary::from_summary(&page.items.last().ok_or_else(missing_export_row)?.summary);
        let has_more = cursor.advance(boundary, page.snapshot, page.has_next)?;
        queue_page(writer, page.items).await?;
        if !has_more {
            return Ok(());
        }
    }
}

async fn queue_page(writer: &mut dyn SystemLogExportWriter, items: Vec<SystemLogDetail>) -> ObservabilityResult<()> {
    for item in items {
        writer.append(item).await?;
    }
    Ok(())
}

fn empty_page_result(has_next: bool) -> ObservabilityResult<()> {
    if has_next {
        return Err(ObservabilityError::Infrastructure(
            "system log export returned an empty page with more rows indicated".into(),
        ));
    }
    Ok(())
}

#[derive(Debug)]
pub(super) struct ExportCursor {
    limit: u64,
    boundary: Option<SystemLogBoundary>,
    snapshot: Option<SystemLogSnapshot>,
}

impl ExportCursor {
    pub(super) fn new(limit: u64) -> ObservabilityResult<Self> {
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
    use crate::application::{SystemLogBoundary, SystemLogSnapshot};

    use super::ExportCursor;

    #[test]
    fn export_cursor_keeps_the_snapshot_across_batches() {
        let mut cursor = ExportCursor::new(2).unwrap();
        let snapshot = SystemLogSnapshot::new(1);
        let boundary = SystemLogBoundary {
            occurred_at_nanos: "0".into(),
            id: "last".into(),
        };

        assert!(cursor.advance(boundary.clone(), Some(snapshot.clone()), true).unwrap());
        assert_eq!(cursor.request().boundary, Some(boundary));
        assert_eq!(cursor.request().snapshot, Some(snapshot));
    }
}
