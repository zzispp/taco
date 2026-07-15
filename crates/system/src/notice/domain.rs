use kernel::pagination::CursorPageRequest;
use serde::{Deserialize, Serialize};

pub const NOTICE_TYPE_NOTICE: &str = "1";
pub const NOTICE_TYPE_ANNOUNCEMENT: &str = "2";
pub const NOTICE_STATUS_CLOSED: &str = "1";
pub const NOTICE_TOP_LIMIT: u64 = 5;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Notice {
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

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct NoticeInput {
    pub notice_title: String,
    pub notice_type: String,
    pub notice_content: String,
    pub status: String,
    pub remark: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplaceNoticeCommand {
    pub id: String,
    pub input: NoticeInput,
    pub operator: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NoticeSummary {
    pub notice_id: String,
    pub notice_title: String,
    pub notice_type: String,
    pub status: String,
    pub create_by: String,
    pub create_time: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct NoticeListFilter {
    pub page: CursorPageRequest,
    pub notice_title: Option<String>,
    pub create_by: Option<String>,
    pub notice_type: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct NoticeReaderFilter {
    pub page: CursorPageRequest,
    pub search_value: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NoticeTopItem {
    pub notice_id: String,
    pub notice_title: String,
    pub notice_type: String,
    pub create_by: String,
    pub create_time: String,
    pub is_read: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NoticeTopResponse {
    pub items: Vec<NoticeTopItem>,
    pub unread_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct NoticeReader {
    pub user_id: String,
    pub user_name: String,
    pub nick_name: String,
    pub dept_name: Option<String>,
    pub phonenumber: Option<String>,
    pub read_time: String,
}
