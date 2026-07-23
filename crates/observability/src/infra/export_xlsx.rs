use kernel::excel::{ExcelResult, TemporaryXlsxFile, finish_xlsx_workbook};
use rust_xlsxwriter::{Workbook, Worksheet, XlsxError};
use time::{OffsetDateTime, UtcOffset, format_description::BorrowedFormatItem};

use crate::{
    application::{ObservabilityError, ObservabilityResult},
    domain::SystemLogDetail,
};

const MAX_CELL_CHARS: usize = 32_767;
const MAX_WORKSHEET_ROWS: u32 = 1_048_576;
const FIRST_DATA_ROW: u32 = 1;
const UTC_RFC3339_MILLIS: &[BorrowedFormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");

pub(super) struct SystemLogXlsxWriter {
    workbook: Workbook,
    primary: WorksheetSeries,
    continuation: WorksheetSeries,
}

impl SystemLogXlsxWriter {
    pub(super) fn new(sheet_name: &str, headers: [String; 6], continuation_headers: [String; 4]) -> ExcelResult<Self> {
        let mut workbook = Workbook::new();
        let primary = WorksheetSeries::create(&mut workbook, sheet_name, headers.into())?;
        let continuation = WorksheetSeries::create(&mut workbook, &format!("{sheet_name}-continuation"), continuation_headers.into())?;
        Ok(Self {
            workbook,
            primary,
            continuation,
        })
    }

    pub(super) fn append(&mut self, item: SystemLogDetail) -> ObservabilityResult<()> {
        let (row, message, fields) = export_row(item)?;
        self.primary.append(&mut self.workbook, &row).map_err(export_error)?;
        self.append_continuation(message)?;
        self.append_continuation(fields)
    }

    pub(super) fn finish(self) -> ExcelResult<TemporaryXlsxFile> {
        finish_xlsx_workbook(self.workbook)
    }

    fn append_continuation(&mut self, value: ValueChunks) -> ObservabilityResult<()> {
        let log_id = value.log_id.clone();
        let kind = value.kind;
        for (part, content) in value.continuations().into_iter().enumerate() {
            self.continuation
                .append(&mut self.workbook, &[log_id.clone(), kind.into(), (part + 1).to_string(), content])
                .map_err(export_error)?;
        }
        Ok(())
    }
}

fn export_row(item: SystemLogDetail) -> ObservabilityResult<(Vec<String>, ValueChunks, ValueChunks)> {
    let summary = item.summary;
    let fields = serde_json::to_string(&item.fields).map_err(|error| ObservabilityError::Infrastructure(error.to_string()))?;
    let occurred_at = timestamp(summary.occurred_at)?;
    let message = ValueChunks::new(summary.id.clone(), "message", summary.message);
    let fields = ValueChunks::new(summary.id.clone(), "fields", fields);
    let row = vec![
        summary.id,
        occurred_at,
        summary.level.code().into(),
        summary.target,
        message.primary_value(),
        fields.primary_value(),
    ];
    Ok((row, message, fields))
}

fn timestamp(value: OffsetDateTime) -> ObservabilityResult<String> {
    value
        .to_offset(UtcOffset::UTC)
        .format(UTC_RFC3339_MILLIS)
        .map_err(|error| ObservabilityError::Infrastructure(error.to_string()))
}

struct WorksheetSeries {
    base_name: String,
    headers: Vec<String>,
    worksheet_index: usize,
    sheet_number: usize,
    next_row: u32,
}

impl WorksheetSeries {
    fn create(workbook: &mut Workbook, base_name: &str, headers: Vec<String>) -> ExcelResult<Self> {
        let worksheet_index = workbook.worksheets().len();
        let worksheet = workbook.add_worksheet_with_constant_memory();
        worksheet.set_name(sheet_name(base_name, 1)).map_err(excel_write_error)?;
        write_row(worksheet, 0, &headers)?;
        Ok(Self {
            base_name: base_name.into(),
            headers,
            worksheet_index,
            sheet_number: 1,
            next_row: FIRST_DATA_ROW,
        })
    }

    fn append(&mut self, workbook: &mut Workbook, row: &[String]) -> ExcelResult<()> {
        self.rotate(workbook)?;
        let worksheet = workbook.worksheet_from_index(self.worksheet_index).map_err(excel_write_error)?;
        write_row(worksheet, self.next_row, row)?;
        self.next_row += 1;
        Ok(())
    }

    fn rotate(&mut self, workbook: &mut Workbook) -> ExcelResult<()> {
        if self.next_row < MAX_WORKSHEET_ROWS {
            return Ok(());
        }
        self.sheet_number += 1;
        self.worksheet_index = workbook.worksheets().len();
        let worksheet = workbook.add_worksheet_with_constant_memory();
        worksheet.set_name(sheet_name(&self.base_name, self.sheet_number)).map_err(excel_write_error)?;
        write_row(worksheet, 0, &self.headers)?;
        self.next_row = FIRST_DATA_ROW;
        Ok(())
    }
}

struct ValueChunks {
    log_id: String,
    kind: &'static str,
    value: String,
}

impl ValueChunks {
    fn new(log_id: String, kind: &'static str, value: String) -> Self {
        Self { log_id, kind, value }
    }

    fn primary_value(&self) -> String {
        if self.value.chars().count() <= MAX_CELL_CHARS {
            return self.value.clone();
        }
        format!("[continuation:{kind}]", kind = self.kind)
    }

    fn continuations(self) -> Vec<String> {
        if self.value.chars().count() > MAX_CELL_CHARS {
            text_chunks(&self.value)
        } else {
            Vec::new()
        }
    }
}

fn text_chunks(value: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut start = 0;
    let mut chars = 0;
    for (offset, _) in value.char_indices() {
        if chars == MAX_CELL_CHARS {
            chunks.push(value[start..offset].into());
            start = offset;
            chars = 0;
        }
        chars += 1;
    }
    chunks.push(value[start..].into());
    chunks
}

fn sheet_name(base: &str, number: usize) -> String {
    let suffix = if number == 1 { String::new() } else { format!("-{number}") };
    let prefix = base.chars().take(MAX_CELL_CHARS.min(31 - suffix.chars().count())).collect::<String>();
    format!("{prefix}{suffix}")
}

fn write_row(worksheet: &mut Worksheet, row: u32, values: &[String]) -> ExcelResult<()> {
    for (column, value) in values.iter().enumerate() {
        let column = u16::try_from(column).map_err(|_| "excel column index is too large".to_owned())?;
        worksheet.write_string(row, column, value).map_err(excel_write_error)?;
    }
    Ok(())
}

fn export_error(error: String) -> ObservabilityError {
    ObservabilityError::Infrastructure(format!("system log spreadsheet generation failed: {error}"))
}

fn excel_write_error(error: XlsxError) -> String {
    format!("excel write error: {error}")
}

#[cfg(test)]
mod tests {
    use crate::domain::{SystemLogDetail, SystemLogLevel, SystemLogSummary};

    use super::{MAX_CELL_CHARS, SystemLogXlsxWriter, ValueChunks, text_chunks};

    #[test]
    fn long_values_are_reconstructable_from_unicode_safe_chunks() {
        let value = "日".repeat(MAX_CELL_CHARS + 2);
        let chunks = ValueChunks::new("log-1".into(), "fields", value.clone());

        assert_eq!(chunks.primary_value(), "[continuation:fields]");
        assert_eq!(chunks.continuations().concat(), value);
    }

    #[test]
    fn chunks_at_the_excel_boundary_without_splitting_utf8() {
        let value = "日".repeat(MAX_CELL_CHARS + 1);
        let chunks = text_chunks(&value);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].chars().count(), MAX_CELL_CHARS);
        assert_eq!(chunks.concat(), value);
    }

    #[test]
    fn writer_finishes_when_message_and_fields_exceed_excel_cell_limits() {
        let mut writer = writer();
        let long = "日".repeat(MAX_CELL_CHARS + 1);

        writer
            .append(SystemLogDetail {
                summary: SystemLogSummary {
                    id: "long-log".into(),
                    occurred_at: time::OffsetDateTime::UNIX_EPOCH,
                    level: SystemLogLevel::Info,
                    target: "test::export".into(),
                    message: long.clone(),
                },
                fields: serde_json::json!({"payload": long}),
            })
            .unwrap();

        assert!(writer.finish().unwrap().content_length() > 0);
    }

    #[test]
    fn writer_rotates_the_primary_worksheet_before_the_excel_row_limit() {
        let mut writer = writer();
        writer.primary.next_row = super::MAX_WORKSHEET_ROWS;

        writer.append(detail("row-limit")).unwrap();

        assert_eq!(writer.primary.sheet_number, 2);
        assert_eq!(writer.workbook.worksheets().len(), 3);
    }

    fn writer() -> SystemLogXlsxWriter {
        SystemLogXlsxWriter::new(
            "System logs",
            ["id", "time", "level", "target", "message", "fields"].map(str::to_owned),
            ["id", "kind", "part", "content"].map(str::to_owned),
        )
        .unwrap()
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
}
