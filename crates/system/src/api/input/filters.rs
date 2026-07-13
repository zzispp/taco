use kernel::pagination::PageRequest;

use crate::application::{ConfigListFilter, DeptListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter, SystemResult};

use super::{DEPT_TREE_PAGE_SIZE, DeptTreeQuery, SystemListQuery, page, time_range::created_time_range};

const DEPT_TREE_PAGE: PageRequest = PageRequest {
    page: 1,
    page_size: DEPT_TREE_PAGE_SIZE,
};

pub(in crate::api) fn dept_list_filter(query: SystemListQuery) -> SystemResult<DeptListFilter> {
    let range = created_time_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    Ok(DeptListFilter {
        page: page(query.page, query.page_size),
        dept_name: query.dept_name,
        leader: query.leader,
        phone: query.phone,
        email: query.email,
        status: query.status,
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}

pub(in crate::api) fn dept_tree_filter(query: DeptTreeQuery) -> SystemResult<DeptListFilter> {
    let range = created_time_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    Ok(DeptListFilter {
        page: DEPT_TREE_PAGE,
        dept_name: query.dept_name,
        leader: query.leader,
        phone: query.phone,
        email: query.email,
        status: query.status,
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}

pub(in crate::api) fn post_list_filter(query: SystemListQuery) -> SystemResult<PostListFilter> {
    let range = created_time_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    Ok(PostListFilter {
        page: page(query.page, query.page_size),
        post_code: query.post_code,
        post_name: query.post_name,
        status: query.status,
        remark: query.remark,
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}

pub(in crate::api) fn dict_type_list_filter(query: SystemListQuery) -> SystemResult<DictTypeListFilter> {
    let range = created_time_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    Ok(DictTypeListFilter {
        page: page(query.page, query.page_size),
        dict_name: query.dict_name,
        dict_type: query.dict_type,
        status: query.status,
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}

pub(in crate::api) fn dict_data_list_filter(query: SystemListQuery) -> SystemResult<DictDataListFilter> {
    let range = created_time_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    Ok(DictDataListFilter {
        page: page(query.page, query.page_size),
        dict_type: query.dict_type,
        dict_label: query.dict_label,
        status: query.status,
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}

pub(in crate::api) fn config_list_filter(query: SystemListQuery) -> SystemResult<ConfigListFilter> {
    let range = created_time_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    Ok(ConfigListFilter {
        page: page(query.page, query.page_size),
        config_name: query.config_name,
        config_key: query.config_key,
        config_type: query.config_type,
        public_read: query.public_read,
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}
