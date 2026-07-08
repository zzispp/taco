use kernel::excel::write_xlsx;
use kernel::pagination::PageRequest;
use types::{
    http::{Locale, translate_message},
    system::{ConfigItem, DictData, DictType, Post},
};

use crate::application::{ConfigListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter, SystemError, SystemResult};

use super::handlers::SystemExportQuery;

const POST_SHEET_KEY: &str = "excel.system.post.sheet";
const DICT_TYPE_SHEET_KEY: &str = "excel.system.dict_type.sheet";
const DICT_DATA_SHEET_KEY: &str = "excel.system.dict_data.sheet";
const CONFIG_SHEET_KEY: &str = "excel.system.config.sheet";
const POST_HEADER_KEYS: &[&str] = &[
    "excel.system.post.headers.post_id",
    "excel.system.post.headers.post_code",
    "excel.system.post.headers.post_name",
    "excel.system.post.headers.post_sort",
    "excel.common.headers.status",
    "excel.common.headers.remark",
    "excel.common.headers.create_time",
];
const DICT_TYPE_HEADER_KEYS: &[&str] = &[
    "excel.system.dict_type.headers.dict_id",
    "excel.system.dict_type.headers.dict_name",
    "excel.system.dict_type.headers.dict_type",
    "excel.common.headers.status",
    "excel.common.headers.remark",
    "excel.common.headers.create_time",
];
const DICT_DATA_HEADER_KEYS: &[&str] = &[
    "excel.system.dict_data.headers.dict_code",
    "excel.system.dict_data.headers.dict_sort",
    "excel.system.dict_data.headers.dict_label",
    "excel.system.dict_data.headers.dict_value",
    "excel.system.dict_data.headers.dict_type",
    "excel.system.dict_data.headers.css_class",
    "excel.system.dict_data.headers.list_class",
    "excel.system.dict_data.headers.is_default",
    "excel.common.headers.status",
    "excel.common.headers.remark",
    "excel.common.headers.create_time",
];
const CONFIG_HEADER_KEYS: &[&str] = &[
    "excel.system.config.headers.config_id",
    "excel.system.config.headers.config_name",
    "excel.system.config.headers.config_key",
    "excel.system.config.headers.config_value",
    "excel.system.config.headers.config_type",
    "excel.system.config.headers.public_read",
    "excel.common.headers.remark",
    "excel.common.headers.create_time",
];

struct ExportSheet<'a> {
    sheet_key: &'a str,
    header_keys: &'a [&'a str],
    rows: &'a [Vec<String>],
}

pub fn export_posts_xlsx(items: &[Post], locale: Locale) -> SystemResult<Vec<u8>> {
    let rows = items.iter().map(post_row).collect::<Vec<_>>();
    write_export(export_sheet(POST_SHEET_KEY, POST_HEADER_KEYS, &rows), locale)
}

pub fn export_dict_types_xlsx(items: &[DictType], locale: Locale) -> SystemResult<Vec<u8>> {
    let rows = items.iter().map(dict_type_row).collect::<Vec<_>>();
    write_export(export_sheet(DICT_TYPE_SHEET_KEY, DICT_TYPE_HEADER_KEYS, &rows), locale)
}

pub fn export_dict_data_xlsx(items: &[DictData], locale: Locale) -> SystemResult<Vec<u8>> {
    let rows = items.iter().map(dict_data_row).collect::<Vec<_>>();
    write_export(export_sheet(DICT_DATA_SHEET_KEY, DICT_DATA_HEADER_KEYS, &rows), locale)
}

pub fn export_configs_xlsx(items: &[ConfigItem], locale: Locale) -> SystemResult<Vec<u8>> {
    let rows = items.iter().map(config_row).collect::<Vec<_>>();
    write_export(export_sheet(CONFIG_SHEET_KEY, CONFIG_HEADER_KEYS, &rows), locale)
}

pub fn post_export_page(query: &SystemExportQuery, page: u64, page_size: u64) -> PostListFilter {
    PostListFilter {
        page: PageRequest { page, page_size },
        post_code: query.post_code.clone(),
        post_name: query.post_name.clone(),
        status: query.status.clone(),
        remark: query.remark.clone(),
        begin_time: query.begin_time.clone(),
        end_time: query.end_time.clone(),
    }
}

pub fn dict_type_export_page(query: &SystemExportQuery, page: u64, page_size: u64) -> DictTypeListFilter {
    DictTypeListFilter {
        page: PageRequest { page, page_size },
        dict_name: query.dict_name.clone(),
        dict_type: query.dict_type.clone(),
        status: query.status.clone(),
        begin_time: query.begin_time.clone(),
        end_time: query.end_time.clone(),
    }
}

pub fn dict_data_export_page(query: &SystemExportQuery, page: u64, page_size: u64) -> DictDataListFilter {
    DictDataListFilter {
        page: PageRequest { page, page_size },
        dict_type: query.dict_type.clone(),
        dict_label: query.dict_label.clone(),
        status: query.status.clone(),
    }
}

pub fn config_export_page(query: &SystemExportQuery, page: u64, page_size: u64) -> ConfigListFilter {
    ConfigListFilter {
        page: PageRequest { page, page_size },
        config_name: query.config_name.clone(),
        config_key: query.config_key.clone(),
        config_type: query.config_type.clone(),
        public_read: query.public_read,
        begin_time: query.begin_time.clone(),
        end_time: query.end_time.clone(),
    }
}

fn export_sheet<'a>(sheet_key: &'a str, header_keys: &'a [&'a str], rows: &'a [Vec<String>]) -> ExportSheet<'a> {
    ExportSheet { sheet_key, header_keys, rows }
}

fn write_export(sheet: ExportSheet<'_>, locale: Locale) -> SystemResult<Vec<u8>> {
    write_xlsx(&text(locale, sheet.sheet_key), &localized_headers(locale, sheet.header_keys), sheet.rows).map_err(excel_error)
}

fn localized_headers(locale: Locale, keys: &[&str]) -> Vec<String> {
    keys.iter().map(|key| text(locale, key)).collect()
}

fn text(locale: Locale, key: &str) -> String {
    translate_message(locale, key)
}

fn post_row(item: &Post) -> Vec<String> {
    vec![
        item.post_id.clone(),
        item.post_code.clone(),
        item.post_name.clone(),
        item.post_sort.to_string(),
        item.status.clone(),
        item.remark.clone().unwrap_or_default(),
        item.create_time.clone(),
    ]
}

fn dict_type_row(item: &DictType) -> Vec<String> {
    vec![
        item.dict_id.clone(),
        item.dict_name.clone(),
        item.dict_type.clone(),
        item.status.clone(),
        item.remark.clone().unwrap_or_default(),
        item.create_time.clone(),
    ]
}

fn dict_data_row(item: &DictData) -> Vec<String> {
    vec![
        item.dict_code.clone(),
        item.dict_sort.to_string(),
        item.dict_label.clone(),
        item.dict_value.clone(),
        item.dict_type.clone(),
        item.css_class.clone().unwrap_or_default(),
        item.list_class.clone().unwrap_or_default(),
        item.is_default.clone(),
        item.status.clone(),
        item.remark.clone().unwrap_or_default(),
        item.create_time.clone(),
    ]
}

fn config_row(item: &ConfigItem) -> Vec<String> {
    vec![
        item.config_id.clone(),
        item.config_name.clone(),
        item.config_key.clone(),
        item.config_value.clone(),
        item.config_type.clone(),
        item.public_read.to_string(),
        item.remark.clone().unwrap_or_default(),
        item.create_time.clone(),
    ]
}

fn excel_error(error: String) -> SystemError {
    SystemError::Infrastructure(error)
}

#[cfg(test)]
mod tests {
    use super::{export_configs_xlsx, export_posts_xlsx};
    use types::http::Locale;

    #[cfg_attr(miri, ignore = "Miri isolation blocks rust_xlsxwriter SystemTime usage")]
    #[test]
    fn export_posts_headers_use_requested_locale() {
        let rows = kernel::excel::read_xlsx(&export_posts_xlsx(&[], Locale::En).unwrap()).unwrap();

        assert_eq!(rows[0][0], "Post ID");
        assert_eq!(rows[0][1], "Post code");
    }

    #[cfg_attr(miri, ignore = "Miri isolation blocks rust_xlsxwriter SystemTime usage")]
    #[test]
    fn export_configs_headers_use_requested_locale() {
        let rows = kernel::excel::read_xlsx(&export_configs_xlsx(&[], Locale::ZhTw).unwrap()).unwrap();

        assert_eq!(rows[0][0], "參數主鍵");
        assert_eq!(rows[0][5], "公開讀取");
    }
}
