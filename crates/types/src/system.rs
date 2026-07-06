use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct TreeSelectNode {
    pub id: String,
    pub label: String,
    pub parent_id: String,
    pub disabled: bool,
    pub children: Vec<TreeSelectNode>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct SortInput {
    pub order_num: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct SortItem {
    pub id: String,
    pub order_num: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct SortBatchInput {
    pub items: Vec<SortItem>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct BatchIdsInput {
    pub ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Dept {
    pub dept_id: String,
    pub parent_id: String,
    pub ancestors: String,
    pub dept_name: String,
    pub order_num: i64,
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub status: String,
    pub create_time: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct DeptInput {
    pub parent_id: String,
    pub dept_name: String,
    pub order_num: i64,
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Post {
    pub post_id: String,
    pub post_code: String,
    pub post_name: String,
    pub post_sort: i64,
    pub status: String,
    pub remark: Option<String>,
    pub create_time: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct PostInput {
    pub post_code: String,
    pub post_name: String,
    pub post_sort: i64,
    pub status: String,
    pub remark: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct DictType {
    pub dict_id: String,
    pub dict_name: String,
    pub dict_type: String,
    pub status: String,
    pub remark: Option<String>,
    pub create_time: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct DictTypeInput {
    pub dict_name: String,
    pub dict_type: String,
    pub status: String,
    pub remark: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct DictData {
    pub dict_code: String,
    pub dict_sort: i64,
    pub dict_label: String,
    pub dict_value: String,
    pub dict_type: String,
    pub css_class: Option<String>,
    pub list_class: Option<String>,
    pub is_default: String,
    pub status: String,
    pub remark: Option<String>,
    pub create_time: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct DictDataInput {
    pub dict_sort: i64,
    pub dict_label: String,
    pub dict_value: String,
    pub dict_type: String,
    pub css_class: Option<String>,
    pub list_class: Option<String>,
    pub is_default: String,
    pub status: String,
    pub remark: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct ConfigItem {
    pub config_id: String,
    pub config_name: String,
    pub config_key: String,
    pub config_value: String,
    pub config_type: String,
    pub public_read: bool,
    pub remark: Option<String>,
    pub create_time: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct ConfigInput {
    pub config_name: String,
    pub config_key: String,
    pub config_value: String,
    pub config_type: String,
    pub public_read: bool,
    pub remark: Option<String>,
}
