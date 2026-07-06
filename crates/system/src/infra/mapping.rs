use storage::StorageError;

use crate::{
    application::SystemError,
    domain::{ConfigItem, Dept, DictData, DictType, Post},
};

use super::record::{ConfigRecord, DeptRecord, DictDataRecord, DictTypeRecord, PostRecord};

pub fn dept(record: DeptRecord) -> Dept {
    Dept {
        dept_id: record.dept_id,
        parent_id: record.parent_id,
        ancestors: record.ancestors,
        dept_name: record.dept_name,
        order_num: record.order_num,
        leader: record.leader,
        phone: record.phone,
        email: record.email,
        status: record.status,
        create_time: record.create_time,
    }
}

pub fn post(record: PostRecord) -> Post {
    Post {
        post_id: record.post_id,
        post_code: record.post_code,
        post_name: record.post_name,
        post_sort: record.post_sort,
        status: record.status,
        remark: record.remark,
        create_time: record.create_time,
    }
}

pub fn dict_type(record: DictTypeRecord) -> DictType {
    DictType {
        dict_id: record.dict_id,
        dict_name: record.dict_name,
        dict_type: record.dict_type,
        status: record.status,
        remark: record.remark,
        create_time: record.create_time,
    }
}

pub fn dict_data(record: DictDataRecord) -> DictData {
    DictData {
        dict_code: record.dict_code,
        dict_sort: record.dict_sort,
        dict_label: record.dict_label,
        dict_value: record.dict_value,
        dict_type: record.dict_type,
        css_class: record.css_class,
        list_class: record.list_class,
        is_default: record.is_default,
        status: record.status,
        remark: record.remark,
        create_time: record.create_time,
    }
}

pub fn config(record: ConfigRecord) -> ConfigItem {
    ConfigItem {
        config_id: record.config_id,
        config_name: record.config_name,
        config_key: record.config_key,
        config_value: record.config_value,
        config_type: record.config_type,
        public_read: record.public_read,
        remark: record.remark,
        create_time: record.create_time,
    }
}

pub fn storage_error(error: StorageError) -> SystemError {
    match error {
        StorageError::NotFound => SystemError::NotFound,
        StorageError::Conflict(message) => SystemError::Conflict(message),
        StorageError::Database(message) => SystemError::Infrastructure(message),
    }
}
