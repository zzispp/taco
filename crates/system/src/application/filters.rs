use kernel::pagination::PageRequest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeptListFilter {
    pub page: PageRequest,
    pub dept_name: Option<String>,
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostListFilter {
    pub page: PageRequest,
    pub post_code: Option<String>,
    pub post_name: Option<String>,
    pub status: Option<String>,
    pub remark: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DictTypeListFilter {
    pub page: PageRequest,
    pub dict_name: Option<String>,
    pub dict_type: Option<String>,
    pub status: Option<String>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DictDataListFilter {
    pub page: PageRequest,
    pub dict_type: Option<String>,
    pub dict_label: Option<String>,
    pub status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigListFilter {
    pub page: PageRequest,
    pub config_name: Option<String>,
    pub config_key: Option<String>,
    pub config_type: Option<String>,
    pub public_read: Option<bool>,
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
}
