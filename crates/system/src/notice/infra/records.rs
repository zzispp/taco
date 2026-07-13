use sqlx::FromRow;

#[derive(Clone, Debug, FromRow)]
pub struct NoticeRecord {
    pub notice_id: String,
    pub notice_title: String,
    pub notice_type: String,
    pub notice_content: String,
    pub status: String,
    pub create_by: String,
    pub create_time: String,
    pub update_by: Option<String>,
    pub update_time: Option<String>,
    pub remark: Option<String>,
}

#[derive(Clone, Debug, FromRow)]
pub struct NoticeSummaryRecord {
    pub notice_id: String,
    pub notice_title: String,
    pub notice_type: String,
    pub status: String,
    pub create_by: String,
    pub create_time: String,
}

#[derive(Clone, Debug, FromRow)]
pub struct NoticeTopRecord {
    pub notice_id: String,
    pub notice_title: String,
    pub notice_type: String,
    pub create_by: String,
    pub create_time: String,
    pub is_read: bool,
}

#[derive(Clone, Debug, FromRow)]
pub struct NoticeReaderRecord {
    pub user_id: String,
    pub user_name: String,
    pub nick_name: String,
    pub dept_name: Option<String>,
    pub phonenumber: Option<String>,
    pub read_time: String,
}
