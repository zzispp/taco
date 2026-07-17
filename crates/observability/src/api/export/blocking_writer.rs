use kernel::excel::TemporaryXlsxFile;
use tokio::sync::mpsc;
use types::http::Locale;

use crate::{
    application::{ObservabilityError, ObservabilityResult},
    domain::SystemLogDetail,
};

use super::export_xlsx::SystemLogXlsxWriter;

pub(super) fn write_system_log_xlsx(
    mut receiver: mpsc::Receiver<SystemLogDetail>,
    sheet_name: String,
    headers: Vec<String>,
    locale: Locale,
) -> ObservabilityResult<TemporaryXlsxFile> {
    let mut writer = SystemLogXlsxWriter::new(&sheet_name, headers, locale).map_err(excel_error)?;
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
    use types::http::Locale;

    use crate::domain::{SystemLogDetail, SystemLogLevel, SystemLogSummary};

    use super::write_system_log_xlsx;

    #[test]
    fn blocking_writer_finishes_a_streamed_system_log_workbook() {
        let (sender, receiver) = mpsc::channel(1);
        sender.try_send(detail()).unwrap();
        drop(sender);

        let artifact = write_system_log_xlsx(receiver, "System logs".into(), headers(), Locale::En).unwrap();

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

    fn headers() -> Vec<String> {
        ["id", "time", "level", "target", "message", "fields"].into_iter().map(str::to_owned).collect()
    }
}
