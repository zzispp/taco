use std::collections::HashMap;

use kernel::error::LocalizedError;
use kernel::excel::{read_xlsx, write_xlsx};
use kernel::pagination::PageRequest;
use types::user::User;

use crate::{
    api::dto::UserExportQuery,
    application::{AppError, AppResult, UserImportRow, UserListFilter},
};

const EXPORT_HEADERS: &[&str] = &[
    "用户序号",
    "部门编号",
    "登录名称",
    "用户名称",
    "用户邮箱",
    "手机号码",
    "用户性别",
    "账号状态",
    "创建时间",
];
const IMPORT_HEADERS: &[&str] = &["部门编号", "登录名称", "用户名称", "用户邮箱", "手机号码", "用户性别", "账号状态"];
const EXPORT_SHEET: &str = "用户数据";
const IMPORT_SHEET: &str = "用户导入模板";

pub fn export_users_xlsx(users: &[User]) -> AppResult<Vec<u8>> {
    let rows = users.iter().map(user_row).collect::<Vec<_>>();
    write_xlsx(EXPORT_SHEET, EXPORT_HEADERS, &rows).map_err(excel_infrastructure_error)
}

pub fn import_template_xlsx() -> AppResult<Vec<u8>> {
    write_xlsx(IMPORT_SHEET, IMPORT_HEADERS, &[]).map_err(excel_infrastructure_error)
}

pub fn parse_import_rows(bytes: &[u8]) -> AppResult<Vec<UserImportRow>> {
    let rows = read_xlsx(bytes).map_err(|_| AppError::InvalidInput(localized("errors.user.import_excel_invalid")))?;
    let (header, data_rows) = rows
        .split_first()
        .ok_or_else(|| AppError::InvalidInput(localized("errors.user.import_excel_empty")))?;
    let columns = import_columns(header)?;
    data_rows.iter().filter(|row| !row_is_blank(row)).map(|row| import_row(row, &columns)).collect()
}

pub fn export_query_page(query: &UserExportQuery, page: u64, page_size: u64) -> UserListFilter {
    UserListFilter {
        page: PageRequest { page, page_size },
        username: query.username.clone(),
        phonenumber: query.phonenumber.clone(),
        status: query.status.clone(),
        dept_id: query.dept_id.clone(),
        begin_time: query.begin_time.clone(),
        end_time: query.end_time.clone(),
    }
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

fn import_columns(header: &[String]) -> AppResult<HashMap<&'static str, usize>> {
    let mut columns = HashMap::new();
    for expected in IMPORT_HEADERS {
        let index = header
            .iter()
            .position(|value| value.trim() == *expected)
            .ok_or_else(|| AppError::InvalidInput(localized_param("errors.user.import_missing_column", "column", *expected)))?;
        columns.insert(*expected, index);
    }
    Ok(columns)
}

fn import_row(row: &[String], columns: &HashMap<&'static str, usize>) -> AppResult<UserImportRow> {
    Ok(UserImportRow {
        dept_id: optional_cell(row, columns, "部门编号"),
        username: required_cell(row, columns, "登录名称")?,
        nick_name: required_cell(row, columns, "用户名称")?,
        email: required_cell(row, columns, "用户邮箱")?,
        phonenumber: optional_cell(row, columns, "手机号码"),
        sex: default_cell(row, columns, "用户性别", "2"),
        status: default_cell(row, columns, "账号状态", "0"),
    })
}

fn required_cell(row: &[String], columns: &HashMap<&'static str, usize>, name: &str) -> AppResult<String> {
    let value = cell(row, columns, name).trim().to_owned();
    if value.is_empty() {
        return Err(AppError::InvalidInput(localized_param("errors.user.import_column_blank", "column", name)));
    }
    Ok(value)
}

fn optional_cell(row: &[String], columns: &HashMap<&'static str, usize>, name: &str) -> Option<String> {
    let value = cell(row, columns, name).trim().to_owned();
    (!value.is_empty()).then_some(value)
}

fn default_cell(row: &[String], columns: &HashMap<&'static str, usize>, name: &str, default: &str) -> String {
    optional_cell(row, columns, name).unwrap_or_else(|| default.into())
}

fn cell<'a>(row: &'a [String], columns: &HashMap<&'static str, usize>, name: &str) -> &'a str {
    columns.get(name).and_then(|index| row.get(*index)).map(String::as_str).unwrap_or_default()
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
    use super::{IMPORT_HEADERS, import_template_xlsx, parse_import_rows};
    use kernel::excel::write_xlsx;

    #[test]
    fn import_template_has_no_role_or_post_columns() {
        let rows = kernel::excel::read_xlsx(&import_template_xlsx().unwrap()).unwrap();

        assert_eq!(rows[0], IMPORT_HEADERS);
    }

    #[test]
    fn parses_import_rows_without_roles_or_posts() {
        let bytes = write_xlsx(
            "用户数据",
            IMPORT_HEADERS,
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

        let rows = parse_import_rows(&bytes).unwrap();

        assert_eq!(rows[0].username, "alice");
        assert_eq!(rows[0].dept_id.as_deref(), Some("103"));
    }
}
