use sqlx::FromRow;

#[derive(Clone, Debug, FromRow)]
pub struct DeptRecord {
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

#[derive(Clone, Debug, FromRow)]
pub struct PostRecord {
    pub post_id: String,
    pub post_code: String,
    pub post_name: String,
    pub post_sort: i64,
    pub status: String,
    pub remark: Option<String>,
    pub create_time: String,
}

#[derive(Clone, Debug, FromRow)]
pub struct DictTypeRecord {
    pub dict_id: String,
    pub dict_name: String,
    pub dict_type: String,
    pub status: String,
    pub remark: Option<String>,
    pub create_time: String,
}

#[derive(Clone, Debug, FromRow)]
pub struct DictDataRecord {
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

#[derive(Clone, Debug, FromRow)]
pub struct ConfigRecord {
    pub config_id: String,
    pub config_name: String,
    pub config_key: String,
    pub config_value: String,
    pub config_type: String,
    pub remark: Option<String>,
    pub create_time: String,
}
