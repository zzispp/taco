use crate::application::{ConfigListFilter, DeptListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter, SystemResult};

use super::{DeptTreeQuery, SystemListQuery, cursor_page, time_range::created_time_range};

pub(in crate::api) fn dept_list_filter(query: SystemListQuery) -> SystemResult<DeptListFilter> {
    let range = created_time_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    Ok(DeptListFilter {
        page: cursor_page(&query),
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
        page: kernel::pagination::CursorPageRequest::default(),
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
        page: cursor_page(&query),
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
        page: cursor_page(&query),
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
        page: cursor_page(&query),
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
        page: cursor_page(&query),
        config_name: query.config_name,
        config_key: query.config_key,
        config_type: query.config_type,
        public_read: query.public_read,
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}
