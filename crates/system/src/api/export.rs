use kernel::excel::write_xlsx;
use kernel::pagination::PageRequest;
use types::system::{ConfigItem, DictData, DictType, Post};

use crate::application::{ConfigListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter, SystemError, SystemResult};

use super::handlers::SystemExportQuery;

const POST_HEADERS: &[&str] = &["岗位序号", "岗位编码", "岗位名称", "岗位排序", "状态", "备注", "创建时间"];
const DICT_TYPE_HEADERS: &[&str] = &["字典主键", "字典名称", "字典类型", "状态", "备注", "创建时间"];
const DICT_DATA_HEADERS: &[&str] = &[
    "字典编码",
    "字典排序",
    "字典标签",
    "字典键值",
    "字典类型",
    "样式属性",
    "表格回显样式",
    "是否默认",
    "状态",
    "备注",
    "创建时间",
];
const CONFIG_HEADERS: &[&str] = &["参数主键", "参数名称", "参数键名", "参数键值", "系统内置", "公开读取", "备注", "创建时间"];

pub fn export_posts_xlsx(items: &[Post]) -> SystemResult<Vec<u8>> {
    write_xlsx("岗位数据", POST_HEADERS, &items.iter().map(post_row).collect::<Vec<_>>()).map_err(excel_error)
}

pub fn export_dict_types_xlsx(items: &[DictType]) -> SystemResult<Vec<u8>> {
    write_xlsx("字典类型", DICT_TYPE_HEADERS, &items.iter().map(dict_type_row).collect::<Vec<_>>()).map_err(excel_error)
}

pub fn export_dict_data_xlsx(items: &[DictData]) -> SystemResult<Vec<u8>> {
    write_xlsx("字典数据", DICT_DATA_HEADERS, &items.iter().map(dict_data_row).collect::<Vec<_>>()).map_err(excel_error)
}

pub fn export_configs_xlsx(items: &[ConfigItem]) -> SystemResult<Vec<u8>> {
    write_xlsx("参数数据", CONFIG_HEADERS, &items.iter().map(config_row).collect::<Vec<_>>()).map_err(excel_error)
}

pub fn post_export_page(query: &SystemExportQuery, page: u64, page_size: u64) -> PostListFilter {
    PostListFilter {
        page: PageRequest { page, page_size },
        post_code: query.post_code.clone(),
        post_name: query.post_name.clone(),
        status: query.status.clone(),
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
        begin_time: query.begin_time.clone(),
        end_time: query.end_time.clone(),
    }
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
