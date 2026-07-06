use std::io::Cursor;

use calamine::{Data, Reader, open_workbook_auto_from_rs};
use rust_xlsxwriter::{Workbook, XlsxError};

pub type ExcelResult<T> = Result<T, String>;

pub fn write_xlsx(sheet_name: &str, headers: &[&str], rows: &[Vec<String>]) -> ExcelResult<Vec<u8>> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name(sheet_name).map_err(excel_write_error)?;
    write_headers(worksheet, headers)?;
    write_rows(worksheet, rows)?;
    workbook.save_to_buffer().map_err(excel_write_error)
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

fn write_headers(worksheet: &mut rust_xlsxwriter::Worksheet, headers: &[&str]) -> ExcelResult<()> {
    for (column, header) in headers.iter().enumerate() {
        worksheet.write_string(0, column_index(column)?, *header).map_err(excel_write_error)?;
    }
    Ok(())
}

fn write_rows(worksheet: &mut rust_xlsxwriter::Worksheet, rows: &[Vec<String>]) -> ExcelResult<()> {
    for (row_index, row) in rows.iter().enumerate() {
        write_row(worksheet, row_index + 1, row)?;
    }
    Ok(())
}

fn write_row(worksheet: &mut rust_xlsxwriter::Worksheet, row_index: usize, row: &[String]) -> ExcelResult<()> {
    for (column, value) in row.iter().enumerate() {
        worksheet
            .write_string(row_number(row_index)?, column_index(column)?, value)
            .map_err(excel_write_error)?;
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

fn row_number(value: usize) -> ExcelResult<u32> {
    u32::try_from(value).map_err(|_| "excel row index is too large".to_owned())
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
    use super::{read_xlsx, write_xlsx};

    #[test]
    fn writes_and_reads_xlsx_rows() {
        let bytes = write_xlsx("用户数据", &["登录名称", "用户名称"], &[vec!["alice".into(), "Alice".into()]]).unwrap();
        let rows = read_xlsx(&bytes).unwrap();

        assert_eq!(rows[0], vec!["登录名称", "用户名称"]);
        assert_eq!(rows[1], vec!["alice", "Alice"]);
    }
}
