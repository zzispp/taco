use std::{
    fs::File,
    io::{Cursor, Seek, SeekFrom, Write},
    path::Path,
};

use calamine::{Data, Reader, open_workbook_auto_from_rs};
use rust_xlsxwriter::{Workbook, XlsxError};
use tempfile::{NamedTempFile, TempPath};

pub type ExcelResult<T> = Result<T, String>;

const WORKSHEET_INDEX: usize = 0;
const HEADER_ROW: u32 = 0;
const FIRST_DATA_ROW: u32 = 1;

pub fn write_xlsx<S: AsRef<str>>(sheet_name: &str, headers: &[S], rows: &[Vec<String>]) -> ExcelResult<Vec<u8>> {
    let mut writer = StreamingXlsxWriter::new(sheet_name, headers)?;
    writer.append_rows(rows)?;
    writer.finish_to_buffer()
}

/// A completed XLSX artifact backed by a securely created temporary file.
pub struct TemporaryXlsxFile {
    temporary_file: NamedTempFile,
    content_length: u64,
}

impl TemporaryXlsxFile {
    /// Returns the number of bytes in the completed XLSX artifact.
    #[must_use]
    pub const fn content_length(&self) -> u64 {
        self.content_length
    }

    /// Returns the temporary path for diagnostics and tests.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.temporary_file.path()
    }

    /// Transfers the file and its cleanup lifetime to a streaming response.
    #[must_use]
    pub fn into_stream_parts(self) -> (File, u64, TemporaryXlsxCleanupGuard) {
        let Self {
            temporary_file,
            content_length,
        } = self;
        let (file, temporary_path) = temporary_file.into_parts();
        let cleanup = TemporaryXlsxCleanupGuard {
            _temporary_path: temporary_path,
        };
        (file, content_length, cleanup)
    }
}

/// Owns the temporary path until an XLSX response finishes or is dropped.
pub struct TemporaryXlsxCleanupGuard {
    _temporary_path: TempPath,
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

    /// Finalizes the workbook into a temporary artifact suitable for streaming.
    pub fn finish(self) -> ExcelResult<TemporaryXlsxFile> {
        let temporary_file = NamedTempFile::new().map_err(excel_temporary_file_error)?;
        save_workbook_to_temporary_file(self.workbook, temporary_file)
    }

    fn finish_to_buffer(mut self) -> ExcelResult<Vec<u8>> {
        self.workbook.save_to_buffer().map_err(excel_write_error)
    }
}

fn save_workbook_to_temporary_file(mut workbook: Workbook, mut temporary_file: NamedTempFile) -> ExcelResult<TemporaryXlsxFile> {
    workbook.save_to_writer(temporary_file.as_file_mut()).map_err(excel_write_error)?;
    temporary_file.as_file_mut().flush().map_err(excel_temporary_file_error)?;
    let content_length = temporary_file.as_file().metadata().map_err(excel_temporary_file_error)?.len();
    rewind(temporary_file.as_file_mut())?;
    Ok(TemporaryXlsxFile {
        temporary_file,
        content_length,
    })
}

fn rewind(file: &mut File) -> ExcelResult<()> {
    file.seek(SeekFrom::Start(0)).map_err(excel_temporary_file_error)?;
    Ok(())
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

fn excel_temporary_file_error(error: std::io::Error) -> String {
    format!("excel temporary file error: {error}")
}

fn excel_read_error(error: impl std::fmt::Display) -> String {
    format!("excel read error: {error}")
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{Read, Seek},
    };

    use tempfile::NamedTempFile;

    use super::{StreamingXlsxWriter, read_xlsx, save_workbook_to_temporary_file, write_xlsx};

    #[test]
    fn writes_and_reads_xlsx_rows() {
        let bytes = write_xlsx("用户数据", &["登录名称", "用户名称"], &[vec!["alice".into(), "Alice".into()]]).unwrap();
        let rows = read_xlsx(&bytes).unwrap();

        assert_eq!(rows[0], vec!["登录名称", "用户名称"]);
        assert_eq!(rows[1], vec!["alice", "Alice"]);
    }

    #[test]
    fn appends_rows_in_independent_batches() {
        let mut writer = StreamingXlsxWriter::new("users", &["name"]).unwrap();
        writer.append_rows(&[vec!["alice".into()]]).unwrap();
        writer.append_rows(&[vec!["bob".into()]]).unwrap();

        let artifact = writer.finish().unwrap();
        let bytes = std::fs::read(artifact.path()).unwrap();
        let rows = read_xlsx(&bytes).unwrap();

        assert_eq!(rows, vec![vec!["name"], vec!["alice"], vec!["bob"]]);
        assert_eq!(artifact.content_length(), bytes.len() as u64);
    }

    #[test]
    fn dropping_artifact_removes_its_temporary_path() {
        let artifact = StreamingXlsxWriter::new("users", &["name"]).unwrap().finish().unwrap();
        let path = artifact.path().to_path_buf();

        assert!(path.exists());
        drop(artifact);
        assert!(!path.exists());
    }

    #[test]
    fn stream_parts_start_at_the_beginning_and_guard_the_path() {
        let mut writer = StreamingXlsxWriter::new("users", &["name"]).unwrap();
        writer.append_rows(&[vec!["alice".into()]]).unwrap();
        let artifact = writer.finish().unwrap();
        let path = artifact.path().to_path_buf();
        let (mut file, content_length, cleanup) = artifact.into_stream_parts();
        let mut bytes = Vec::new();

        assert_eq!(file.stream_position().unwrap(), 0);
        file.read_to_end(&mut bytes).unwrap();
        assert_eq!(content_length, bytes.len() as u64);
        assert_eq!(read_xlsx(&bytes).unwrap(), vec![vec!["name"], vec!["alice"]]);
        drop(file);
        assert!(path.exists());
        drop(cleanup);
        assert!(!path.exists());
    }

    #[test]
    fn failed_save_removes_its_temporary_path() {
        let writer = StreamingXlsxWriter::new("users", &["name"]).unwrap();
        let temporary_file = NamedTempFile::new().unwrap();
        let path = temporary_file.path().to_path_buf();
        let (writable_file, temporary_path) = temporary_file.into_parts();
        drop(writable_file);
        let read_only_file = File::open(&path).unwrap();
        let read_only_temporary_file = NamedTempFile::from_parts(read_only_file, temporary_path);

        let result = save_workbook_to_temporary_file(writer.workbook, read_only_temporary_file);

        assert!(result.is_err());
        assert!(!path.exists());
    }
}
