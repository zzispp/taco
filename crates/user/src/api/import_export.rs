use std::collections::HashMap;

use kernel::error::LocalizedError;
use kernel::excel::{read_xlsx, write_xlsx};
use types::{
    http::{Locale, translate_message},
    user::User,
};

use crate::application::{AppError, AppResult, UserImportRow};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum ImportField {
    DeptId,
    Username,
    NickName,
    Email,
    PhoneNumber,
    Sex,
    Status,
}

#[derive(Clone, Copy)]
struct ImportColumnDef {
    field: ImportField,
    label_key: &'static str,
}

type ImportColumns = HashMap<ImportField, usize>;

struct ImportRowReader<'a> {
    row: &'a [String],
    columns: &'a ImportColumns,
    locale: Locale,
}

const EXPORT_SHEET_KEY: &str = "excel.user.export.sheet";
const IMPORT_SHEET_KEY: &str = "excel.user.import.sheet";
const EXPORT_HEADER_KEYS: &[&str] = &[
    "excel.user.headers.user_id",
    "excel.user.headers.dept_id",
    "excel.user.headers.login_name",
    "excel.user.headers.user_name",
    "excel.user.headers.email",
    "excel.user.headers.phone_number",
    "excel.user.headers.sex",
    "excel.user.headers.status",
    "excel.user.headers.create_time",
];
const IMPORT_COLUMNS: &[ImportColumnDef] = &[
    ImportColumnDef {
        field: ImportField::DeptId,
        label_key: "excel.user.headers.dept_id",
    },
    ImportColumnDef {
        field: ImportField::Username,
        label_key: "excel.user.headers.login_name",
    },
    ImportColumnDef {
        field: ImportField::NickName,
        label_key: "excel.user.headers.user_name",
    },
    ImportColumnDef {
        field: ImportField::Email,
        label_key: "excel.user.headers.email",
    },
    ImportColumnDef {
        field: ImportField::PhoneNumber,
        label_key: "excel.user.headers.phone_number",
    },
    ImportColumnDef {
        field: ImportField::Sex,
        label_key: "excel.user.headers.sex",
    },
    ImportColumnDef {
        field: ImportField::Status,
        label_key: "excel.user.headers.status",
    },
];

pub fn export_users_xlsx(users: &[User], locale: Locale) -> AppResult<Vec<u8>> {
    let rows = users.iter().map(user_row).collect::<Vec<_>>();
    write_xlsx(&text(locale, EXPORT_SHEET_KEY), &localized_headers(locale, EXPORT_HEADER_KEYS), &rows).map_err(excel_infrastructure_error)
}

pub fn import_template_xlsx(locale: Locale) -> AppResult<Vec<u8>> {
    write_xlsx(&text(locale, IMPORT_SHEET_KEY), &localized_import_headers(locale), &[]).map_err(excel_infrastructure_error)
}

pub fn parse_import_rows(bytes: &[u8], locale: Locale) -> AppResult<Vec<UserImportRow>> {
    let rows = read_xlsx(bytes).map_err(|_| AppError::InvalidInput(localized("errors.user.import_excel_invalid")))?;
    let (header, data_rows) = rows
        .split_first()
        .ok_or_else(|| AppError::InvalidInput(localized("errors.user.import_excel_empty")))?;
    let columns = import_columns(header, locale)?;
    data_rows
        .iter()
        .filter(|row| !row_is_blank(row))
        .map(|row| import_row(row, &columns, locale))
        .collect()
}

fn user_row(user: &User) -> Vec<String> {
    vec![
        user.id.0.clone(),
        user.dept_id.clone().unwrap_or_default(),
        user.username.clone(),
        user.nick_name.clone(),
        user.email.clone(),
        user.phonenumber.clone().unwrap_or_default(),
        user.sex.clone(),
        user.status.clone(),
        user.create_time.clone(),
    ]
}

fn import_columns(header: &[String], locale: Locale) -> AppResult<ImportColumns> {
    let mut columns = HashMap::new();
    for expected in IMPORT_COLUMNS {
        let label = text(locale, expected.label_key);
        let index = header
            .iter()
            .position(|value| value.trim() == label)
            .ok_or_else(|| AppError::InvalidInput(localized_param("errors.user.import_missing_column", "column", label.clone())))?;
        columns.insert(expected.field, index);
    }
    Ok(columns)
}

fn import_row(row: &[String], columns: &ImportColumns, locale: Locale) -> AppResult<UserImportRow> {
    let reader = ImportRowReader { row, columns, locale };
    Ok(UserImportRow {
        dept_id: reader.optional_cell(ImportField::DeptId),
        username: reader.required_cell(ImportField::Username)?,
        nick_name: reader.required_cell(ImportField::NickName)?,
        email: reader.required_cell(ImportField::Email)?,
        phonenumber: reader.optional_cell(ImportField::PhoneNumber),
        sex: reader.optional_cell(ImportField::Sex).unwrap_or_else(|| "2".into()),
        status: reader.optional_cell(ImportField::Status).unwrap_or_else(|| "0".into()),
    })
}

impl ImportRowReader<'_> {
    fn required_cell(&self, field: ImportField) -> AppResult<String> {
        let value = self.cell(field).trim().to_owned();
        if value.is_empty() {
            return Err(AppError::InvalidInput(localized_param(
                "errors.user.import_column_blank",
                "column",
                field_label(field, self.locale),
            )));
        }
        Ok(value)
    }

    fn optional_cell(&self, field: ImportField) -> Option<String> {
        let value = self.cell(field).trim().to_owned();
        (!value.is_empty()).then_some(value)
    }

    fn cell(&self, field: ImportField) -> &str {
        self.columns
            .get(&field)
            .and_then(|index| self.row.get(*index))
            .map(String::as_str)
            .unwrap_or_default()
    }
}

fn field_label(field: ImportField, locale: Locale) -> String {
    IMPORT_COLUMNS
        .iter()
        .find(|column| column.field == field)
        .map(|column| text(locale, column.label_key))
        .unwrap_or_default()
}

fn localized_import_headers(locale: Locale) -> Vec<String> {
    IMPORT_COLUMNS.iter().map(|column| text(locale, column.label_key)).collect()
}

fn localized_headers(locale: Locale, keys: &[&str]) -> Vec<String> {
    keys.iter().map(|key| text(locale, key)).collect()
}

fn text(locale: Locale, key: &str) -> String {
    translate_message(locale, key)
}

fn row_is_blank(row: &[String]) -> bool {
    row.iter().all(|value| value.trim().is_empty())
}

fn excel_infrastructure_error(error: String) -> AppError {
    AppError::Infrastructure(error)
}

fn localized(key: &'static str) -> LocalizedError {
    LocalizedError::new(key)
}

fn localized_param(key: &'static str, param: &'static str, value: impl Into<String>) -> LocalizedError {
    LocalizedError::new(key).with_param(param, value)
}

#[cfg(test)]
mod tests {
    use super::{import_template_xlsx, localized_import_headers, parse_import_rows};
    use kernel::excel::write_xlsx;
    use types::http::Locale;

    #[cfg_attr(miri, ignore = "Miri isolation blocks rust_xlsxwriter SystemTime usage")]
    #[test]
    fn import_template_has_no_role_or_post_columns() {
        let rows = kernel::excel::read_xlsx(&import_template_xlsx(Locale::ZhCn).unwrap()).unwrap();

        assert_eq!(rows[0], localized_import_headers(Locale::ZhCn));
    }

    #[cfg_attr(miri, ignore = "Miri isolation blocks rust_xlsxwriter SystemTime usage")]
    #[test]
    fn import_template_uses_requested_locale() {
        let rows = kernel::excel::read_xlsx(&import_template_xlsx(Locale::En).unwrap()).unwrap();

        assert_eq!(rows[0], localized_import_headers(Locale::En));
    }

    #[cfg_attr(miri, ignore = "Miri isolation blocks rust_xlsxwriter SystemTime usage")]
    #[test]
    fn parses_import_rows_without_roles_or_posts() {
        let bytes = write_xlsx(
            "用户数据",
            &localized_import_headers(Locale::ZhCn),
            &[vec![
                "103".into(),
                "alice".into(),
                "Alice".into(),
                "alice@example.com".into(),
                "".into(),
                "2".into(),
                "0".into(),
            ]],
        )
        .unwrap();

        let rows = parse_import_rows(&bytes, Locale::ZhCn).unwrap();

        assert_eq!(rows[0].username, "alice");
        assert_eq!(rows[0].dept_id.as_deref(), Some("103"));
    }
}
