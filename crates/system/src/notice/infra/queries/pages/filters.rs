use sqlx::{Postgres, QueryBuilder};

use crate::notice::{NoticeListFilter, NoticeReaderFilter};

use super::super::NOTICE_SUMMARY_COLUMNS;

const READER_COLUMNS: &str = "r.user_id,u.user_name,u.nick_name,d.dept_name,u.phonenumber,r.read_time";

pub(super) fn notice_query(filter: &NoticeListFilter) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new(format!("SELECT {NOTICE_SUMMARY_COLUMNS} FROM sys_notice WHERE TRUE"));
    push_like(&mut query, "notice_title", &filter.notice_title);
    push_like(&mut query, "create_by", &filter.create_by);
    if let Some(value) = &filter.notice_type {
        query.push(" AND notice_type=").push_bind(value.clone());
    }
    query
}

pub(super) fn reader_query(notice_id: &str, filter: &NoticeReaderFilter) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new(format!(
        "SELECT {READER_COLUMNS} FROM sys_notice_read r JOIN sys_user u ON u.user_id=r.user_id LEFT JOIN sys_dept d ON d.dept_id=u.dept_id WHERE r.notice_id="
    ));
    query.push_bind(notice_id.to_owned()).push(" AND u.del_flag='0'");
    if let Some(value) = &filter.search_value {
        query.push(" AND (u.user_name ILIKE '%' || ").push_bind(value.clone());
        query.push(" || '%' OR u.nick_name ILIKE '%' || ").push_bind(value.clone()).push(" || '%')");
    }
    query
}

fn push_like(query: &mut QueryBuilder<Postgres>, column: &str, value: &Option<String>) {
    if let Some(value) = value {
        query.push(" AND ").push(column).push(" ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
    }
}
