use async_trait::async_trait;
use kernel::excel::TemporaryXlsxFile;
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
    application::{ObservabilityError, ObservabilityResult, SystemLogExportWriter, SystemLogExportWriterFactory, SystemLogExportWriterRequest},
    domain::SystemLogDetail,
};

use super::export_xlsx::SystemLogXlsxWriter;

pub struct SystemLogXlsxWriterFactory;

impl SystemLogExportWriterFactory for SystemLogXlsxWriterFactory {
    fn start(&self, request: SystemLogExportWriterRequest) -> ObservabilityResult<Box<dyn SystemLogExportWriter>> {
        let (sheet_name, headers, continuation_headers) = request.layout.into_parts();
        BlockingSystemLogExportWriter::start(request.capacity, sheet_name, headers, continuation_headers)
            .map(|writer| Box::new(writer) as Box<dyn SystemLogExportWriter>)
    }
}

struct BlockingSystemLogExportWriter {
    sender: mpsc::Sender<SystemLogDetail>,
    writer: JoinHandle<ObservabilityResult<TemporaryXlsxFile>>,
}

impl BlockingSystemLogExportWriter {
    fn start(capacity: usize, sheet_name: String, headers: [String; 6], continuation_headers: [String; 4]) -> ObservabilityResult<Self> {
        let xlsx = SystemLogXlsxWriter::new(&sheet_name, headers, continuation_headers).map_err(excel_error)?;
        let (sender, receiver) = mpsc::channel(capacity);
        let writer = tokio::task::spawn_blocking(move || write_system_log_xlsx(receiver, xlsx));
        Ok(Self { sender, writer })
    }
}

#[async_trait]
impl SystemLogExportWriter for BlockingSystemLogExportWriter {
    async fn append(&mut self, item: SystemLogDetail) -> ObservabilityResult<()> {
        self.sender
            .send(item)
            .await
            .map_err(|_| ObservabilityError::Infrastructure("system log XLSX writer stopped before export completed".into()))
    }

    async fn finish(self: Box<Self>) -> ObservabilityResult<TemporaryXlsxFile> {
        let Self { sender, writer } = *self;
        drop(sender);
        writer
            .await
            .map_err(|error| ObservabilityError::Infrastructure(format!("system log XLSX writer task failed: {error}")))?
    }
}

fn write_system_log_xlsx(mut receiver: mpsc::Receiver<SystemLogDetail>, mut writer: SystemLogXlsxWriter) -> ObservabilityResult<TemporaryXlsxFile> {
    while let Some(item) = receiver.blocking_recv() {
        writer.append(item).map_err(excel_error)?;
    }
    writer.finish().map_err(excel_error)
}

fn excel_error(error: impl std::fmt::Display) -> ObservabilityError {
    ObservabilityError::Infrastructure(format!("system log spreadsheet generation failed: {error}"))
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tokio::sync::mpsc;

    use crate::domain::{SystemLogDetail, SystemLogLevel, SystemLogSummary};

    use super::{SystemLogXlsxWriter, write_system_log_xlsx};

    #[test]
    fn blocking_writer_finishes_a_streamed_system_log_workbook() {
        let (sender, receiver) = mpsc::channel(1);
        sender.try_send(detail()).unwrap();
        drop(sender);

        let writer = SystemLogXlsxWriter::new("System logs", headers(), continuation_headers()).unwrap();
        let artifact = write_system_log_xlsx(receiver, writer).unwrap();

        assert!(artifact.content_length() > 0);
    }

    fn detail() -> SystemLogDetail {
        SystemLogDetail {
            summary: SystemLogSummary {
                id: "log-1".into(),
                occurred_at: time::OffsetDateTime::UNIX_EPOCH,
                level: SystemLogLevel::Info,
                target: "test::export".into(),
                message: "message".into(),
            },
            fields: json!({"request_id": "request-1"}),
        }
    }

    fn headers() -> [String; 6] {
        ["id", "time", "level", "target", "message", "fields"].map(str::to_owned)
    }

    fn continuation_headers() -> [String; 4] {
        ["id", "kind", "part", "content"].map(str::to_owned)
    }
}
