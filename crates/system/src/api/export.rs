use kernel::excel::{StreamingXlsxWriter, TemporaryXlsxFile};
use types::{
    http::{Locale, translate_message},
    system::{ConfigItem, DictData, DictType, Post},
};

use crate::application::{SystemError, SystemExportBatch, SystemExportSink, SystemResult};

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

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExportKind {
    Posts,
    DictTypes,
    DictData,
    Configs,
}

pub struct SystemXlsxExport {
    kind: ExportKind,
    writer: StreamingXlsxWriter,
}

struct ExportLayout {
    kind: ExportKind,
    sheet_key: &'static str,
    header_keys: &'static [&'static str],
}

impl SystemXlsxExport {
    pub fn posts(locale: Locale) -> SystemResult<Self> {
        Self::new(
            ExportLayout {
                kind: ExportKind::Posts,
                sheet_key: POST_SHEET_KEY,
                header_keys: POST_HEADER_KEYS,
            },
            locale,
        )
    }

    pub fn dict_types(locale: Locale) -> SystemResult<Self> {
        Self::new(
            ExportLayout {
                kind: ExportKind::DictTypes,
                sheet_key: DICT_TYPE_SHEET_KEY,
                header_keys: DICT_TYPE_HEADER_KEYS,
            },
            locale,
        )
    }

    pub fn dict_data(locale: Locale) -> SystemResult<Self> {
        Self::new(
            ExportLayout {
                kind: ExportKind::DictData,
                sheet_key: DICT_DATA_SHEET_KEY,
                header_keys: DICT_DATA_HEADER_KEYS,
            },
            locale,
        )
    }

    pub fn configs(locale: Locale) -> SystemResult<Self> {
        Self::new(
            ExportLayout {
                kind: ExportKind::Configs,
                sheet_key: CONFIG_SHEET_KEY,
                header_keys: CONFIG_HEADER_KEYS,
            },
            locale,
        )
    }

    fn new(layout: ExportLayout, locale: Locale) -> SystemResult<Self> {
        let writer = StreamingXlsxWriter::new(&text(locale, layout.sheet_key), &localized_headers(locale, layout.header_keys)).map_err(excel_error)?;
        Ok(Self { kind: layout.kind, writer })
    }

    pub fn finish(self) -> SystemResult<TemporaryXlsxFile> {
        self.writer.finish().map_err(excel_error)
    }
}

impl SystemExportSink for SystemXlsxExport {
    fn append(&mut self, batch: SystemExportBatch) -> SystemResult<()> {
        let rows: Vec<Vec<String>> = match (self.kind, batch) {
            (ExportKind::Posts, SystemExportBatch::Posts(items)) => items.iter().map(post_row).collect(),
            (ExportKind::DictTypes, SystemExportBatch::DictTypes(items)) => items.iter().map(dict_type_row).collect(),
            (ExportKind::DictData, SystemExportBatch::DictData(items)) => items.iter().map(dict_data_row).collect(),
            (ExportKind::Configs, SystemExportBatch::Configs(items)) => items.iter().map(config_row).collect(),
            _ => return Err(SystemError::Infrastructure("system export batch type mismatch".into())),
        };
        self.writer.append_rows(&rows).map_err(excel_error)
    }
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
    use super::SystemXlsxExport;
    use types::http::Locale;

    #[test]
    fn export_posts_headers_use_requested_locale() {
        let artifact = SystemXlsxExport::posts(Locale::En).unwrap().finish().unwrap();
        let bytes = std::fs::read(artifact.path()).unwrap();
        let rows = kernel::excel::read_xlsx(&bytes).unwrap();

        assert_eq!(rows[0][0], "Post ID");
        assert_eq!(rows[0][1], "Post code");
    }

    #[test]
    fn export_configs_headers_use_requested_locale() {
        let artifact = SystemXlsxExport::configs(Locale::ZhTw).unwrap().finish().unwrap();
        let bytes = std::fs::read(artifact.path()).unwrap();
        let rows = kernel::excel::read_xlsx(&bytes).unwrap();

        assert_eq!(rows[0][0], "參數主鍵");
        assert_eq!(rows[0][5], "公開讀取");
    }
}
