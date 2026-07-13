use kernel::pagination::PageRequest;
use serde::Deserialize;

mod export_filters;
mod filters;
mod time_range;

pub(super) use export_filters::{
    ConfigExportFilter, DictDataExportFilter, DictTypeExportFilter, PostExportFilter, config_export_filter, dict_data_export_filter, dict_type_export_filter,
    post_export_filter,
};
pub(super) use filters::{config_list_filter, dept_list_filter, dept_tree_filter, dict_data_list_filter, dict_type_list_filter, post_list_filter};

pub(super) const DEPT_TREE_PAGE_SIZE: u64 = 100_000;

#[derive(Debug, Deserialize)]
pub(super) struct SystemListQuery {
    pub page: u64,
    pub page_size: u64,
    pub dept_name: Option<String>,
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub post_code: Option<String>,
    pub post_name: Option<String>,
    pub remark: Option<String>,
    pub dict_name: Option<String>,
    pub dict_type: Option<String>,
    pub dict_label: Option<String>,
    pub config_name: Option<String>,
    pub config_key: Option<String>,
    pub config_type: Option<String>,
    pub public_read: Option<bool>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct SystemExportQuery {
    pub post_code: Option<String>,
    pub post_name: Option<String>,
    pub remark: Option<String>,
    pub dict_name: Option<String>,
    pub dict_type: Option<String>,
    pub dict_label: Option<String>,
    pub config_name: Option<String>,
    pub config_key: Option<String>,
    pub config_type: Option<String>,
    pub public_read: Option<bool>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub(super) struct DeptTreeQuery {
    pub dept_name: Option<String>,
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

const fn page(page: u64, page_size: u64) -> PageRequest {
    PageRequest { page, page_size }
}
