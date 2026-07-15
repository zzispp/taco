use kernel::pagination::CursorPageRequest;

use super::{PostListFilter, filtered_query, post_window, push_window};
use crate::application::{TimeIdPoint, point};

fn filter() -> PostListFilter {
    PostListFilter {
        page: CursorPageRequest::default(),
        post_code: Some("code".into()),
        post_name: Some("name".into()),
        status: None,
        remark: Some("remark".into()),
        begin_time: Some(time::OffsetDateTime::UNIX_EPOCH),
        end_time: Some(time::OffsetDateTime::UNIX_EPOCH),
    }
}

fn query_sql() -> String {
    filtered_query(&filter()).into_string()
}

fn snapshot() -> TimeIdPoint {
    point(time::OffsetDateTime::UNIX_EPOCH, "post-z".into()).unwrap()
}

#[test]
fn post_text_filters_use_case_insensitive_search() {
    let sql = query_sql();

    assert!(sql.contains("post_code ILIKE"));
    assert!(sql.contains("post_name ILIKE"));
    assert!(sql.contains("remark ILIKE"));
}

#[test]
fn post_time_filters_compare_timestamps_without_date_truncation() {
    let sql = query_sql();

    assert!(sql.contains("create_time>="));
    assert!(sql.contains("create_time<="));
    assert!(!sql.contains("::date"));
    assert!(!sql.contains("OFFSET"));
}

#[test]
fn post_page_uses_snapshot_and_business_sort_keyset() {
    let snapshot = snapshot();
    let window = post_window(None, &snapshot, 20).unwrap();
    let mut query = filtered_query(&filter());
    push_window(&mut query, &window).unwrap();
    let sql = query.into_string();

    assert!(sql.contains("(create_time,post_id)<="));
    assert!(sql.contains("ORDER BY post_sort ASC,post_id ASC"));
    assert!(sql.contains("LIMIT"));
    assert!(!sql.contains("OFFSET"));
}
