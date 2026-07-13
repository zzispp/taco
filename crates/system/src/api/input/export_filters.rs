use kernel::pagination::PageRequest;
use time::OffsetDateTime;

use crate::application::{ConfigListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter, SystemResult};

use super::{SystemExportQuery, time_range::created_time_range};

pub(in crate::api) struct PostExportFilter {
    post_code: Option<String>,
    post_name: Option<String>,
    status: Option<String>,
    remark: Option<String>,
    begin_time: Option<OffsetDateTime>,
    end_time: Option<OffsetDateTime>,
}

impl PostExportFilter {
    pub(in crate::api) fn page_filter(&self, page: PageRequest) -> PostListFilter {
        PostListFilter {
            page,
            post_code: self.post_code.clone(),
            post_name: self.post_name.clone(),
            status: self.status.clone(),
            remark: self.remark.clone(),
            begin_time: self.begin_time,
            end_time: self.end_time,
        }
    }
}

pub(in crate::api) fn post_export_filter(query: SystemExportQuery) -> SystemResult<PostExportFilter> {
    let range = created_time_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    Ok(PostExportFilter {
        post_code: query.post_code,
        post_name: query.post_name,
        status: query.status,
        remark: query.remark,
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}

pub(in crate::api) struct DictTypeExportFilter {
    dict_name: Option<String>,
    dict_type: Option<String>,
    status: Option<String>,
    begin_time: Option<OffsetDateTime>,
    end_time: Option<OffsetDateTime>,
}

impl DictTypeExportFilter {
    pub(in crate::api) fn page_filter(&self, page: PageRequest) -> DictTypeListFilter {
        DictTypeListFilter {
            page,
            dict_name: self.dict_name.clone(),
            dict_type: self.dict_type.clone(),
            status: self.status.clone(),
            begin_time: self.begin_time,
            end_time: self.end_time,
        }
    }
}

pub(in crate::api) fn dict_type_export_filter(query: SystemExportQuery) -> SystemResult<DictTypeExportFilter> {
    let range = created_time_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    Ok(DictTypeExportFilter {
        dict_name: query.dict_name,
        dict_type: query.dict_type,
        status: query.status,
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}

pub(in crate::api) struct DictDataExportFilter {
    dict_type: Option<String>,
    dict_label: Option<String>,
    status: Option<String>,
    begin_time: Option<OffsetDateTime>,
    end_time: Option<OffsetDateTime>,
}

impl DictDataExportFilter {
    pub(in crate::api) fn page_filter(&self, page: PageRequest) -> DictDataListFilter {
        DictDataListFilter {
            page,
            dict_type: self.dict_type.clone(),
            dict_label: self.dict_label.clone(),
            status: self.status.clone(),
            begin_time: self.begin_time,
            end_time: self.end_time,
        }
    }
}

pub(in crate::api) fn dict_data_export_filter(query: SystemExportQuery) -> SystemResult<DictDataExportFilter> {
    let range = created_time_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    Ok(DictDataExportFilter {
        dict_type: query.dict_type,
        dict_label: query.dict_label,
        status: query.status,
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}

pub(in crate::api) struct ConfigExportFilter {
    config_name: Option<String>,
    config_key: Option<String>,
    config_type: Option<String>,
    public_read: Option<bool>,
    begin_time: Option<OffsetDateTime>,
    end_time: Option<OffsetDateTime>,
}

impl ConfigExportFilter {
    pub(in crate::api) fn page_filter(&self, page: PageRequest) -> ConfigListFilter {
        ConfigListFilter {
            page,
            config_name: self.config_name.clone(),
            config_key: self.config_key.clone(),
            config_type: self.config_type.clone(),
            public_read: self.public_read,
            begin_time: self.begin_time,
            end_time: self.end_time,
        }
    }
}

pub(in crate::api) fn config_export_filter(query: SystemExportQuery) -> SystemResult<ConfigExportFilter> {
    let range = created_time_range(query.begin_time.as_deref(), query.end_time.as_deref())?;
    Ok(ConfigExportFilter {
        config_name: query.config_name,
        config_key: query.config_key,
        config_type: query.config_type,
        public_read: query.public_read,
        begin_time: range.begin_time,
        end_time: range.end_time,
    })
}
