use std::io::Cursor;

use calamine::{Data, Reader, open_workbook_auto_from_rs};
use rust_xlsxwriter::{Workbook, XlsxError};

pub type ExcelResult<T> = Result<T, String>;

const WORKSHEET_INDEX: usize = 0;
const HEADER_ROW: u32 = 0;
const FIRST_DATA_ROW: u32 = 1;

pub fn write_xlsx<S: AsRef<str>>(sheet_name: &str, headers: &[S], rows: &[Vec<String>]) -> ExcelResult<Vec<u8>> {
    let mut writer = StreamingXlsxWriter::new(sheet_name, headers)?;
    writer.append_rows(rows)?;
    writer.finish()
}

/// XLSX writer that flushes completed rows and accepts export pages incrementally.
pub struct StreamingXlsxWriter {
    workbook: Workbook,
    next_row: u32,
}

impl StreamingXlsxWriter {
    /// Creates one constant-memory worksheet and writes its header row.
    pub fn new<S: AsRef<str>>(sheet_name: &str, headers: &[S]) -> ExcelResult<Self> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet_with_constant_memory();
        worksheet.set_name(sheet_name).map_err(excel_write_error)?;
        write_headers(worksheet, headers)?;
        Ok(Self {
            workbook,
            next_row: FIRST_DATA_ROW,
        })
    }

    /// Appends rows after all rows supplied by previous calls.
    pub fn append_rows(&mut self, rows: &[Vec<String>]) -> ExcelResult<()> {
        let worksheet = self.workbook.worksheet_from_index(WORKSHEET_INDEX).map_err(excel_write_error)?;
        for row in rows {
            write_row(worksheet, self.next_row, row)?;
            self.next_row = self.next_row.checked_add(1).ok_or_else(|| "excel row index is too large".to_owned())?;
        }
        Ok(())
    }

    /// Finalizes the workbook into the response buffer.
    pub fn finish(mut self) -> ExcelResult<Vec<u8>> {
        self.workbook.save_to_buffer().map_err(excel_write_error)
    }
}

pub fn read_xlsx(bytes: &[u8]) -> ExcelResult<Vec<Vec<String>>> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut workbook = open_workbook_auto_from_rs(cursor).map_err(excel_read_error)?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| "excel workbook has no sheet".to_owned())?
        .map_err(excel_read_error)?;
    Ok(range.rows().map(row_values).collect())
}

fn write_headers<S: AsRef<str>>(worksheet: &mut rust_xlsxwriter::Worksheet, headers: &[S]) -> ExcelResult<()> {
    for (column, header) in headers.iter().enumerate() {
        worksheet
            .write_string(HEADER_ROW, column_index(column)?, header.as_ref())
            .map_err(excel_write_error)?;
    }
    Ok(())
}

fn write_row(worksheet: &mut rust_xlsxwriter::Worksheet, row_index: u32, row: &[String]) -> ExcelResult<()> {
    for (column, value) in row.iter().enumerate() {
        worksheet.write_string(row_index, column_index(column)?, value).map_err(excel_write_error)?;
    }
    Ok(())
}

fn row_values(row: &[Data]) -> Vec<String> {
    row.iter().map(cell_value).collect()
}

fn cell_value(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.trim().to_owned(),
        Data::Float(value) => numeric_value(*value),
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) | Data::DurationIso(value) => value.trim().to_owned(),
        Data::Error(value) => value.to_string(),
    }
}

fn numeric_value(value: f64) -> String {
    if value.fract() == 0.0 { format!("{value:.0}") } else { value.to_string() }
}

fn column_index(value: usize) -> ExcelResult<u16> {
    u16::try_from(value).map_err(|_| "excel column index is too large".to_owned())
}

fn excel_write_error(error: XlsxError) -> String {
    format!("excel write error: {error}")
}

fn excel_read_error(error: impl std::fmt::Display) -> String {
    format!("excel read error: {error}")
}

#[cfg(test)]
mod tests {
    use super::{StreamingXlsxWriter, read_xlsx, write_xlsx};

    #[cfg_attr(miri, ignore = "Miri isolation blocks rust_xlsxwriter SystemTime usage")]
    #[test]
    fn writes_and_reads_xlsx_rows() {
        let bytes = write_xlsx("用户数据", &["登录名称", "用户名称"], &[vec!["alice".into(), "Alice".into()]]).unwrap();
        let rows = read_xlsx(&bytes).unwrap();

        assert_eq!(rows[0], vec!["登录名称", "用户名称"]);
        assert_eq!(rows[1], vec!["alice", "Alice"]);
    }

    #[cfg_attr(miri, ignore = "Miri isolation blocks rust_xlsxwriter SystemTime usage")]
    #[test]
    fn appends_rows_in_independent_batches() {
        let mut writer = StreamingXlsxWriter::new("users", &["name"]).unwrap();
        writer.append_rows(&[vec!["alice".into()]]).unwrap();
        writer.append_rows(&[vec!["bob".into()]]).unwrap();

        let rows = read_xlsx(&writer.finish().unwrap()).unwrap();

        assert_eq!(rows, vec![vec!["name"], vec!["alice"], vec!["bob"]]);
    }
}
