use kernel::pagination::{CursorDirection, CursorPageRequest};
use time::OffsetDateTime;

use super::*;

fn filter(limit: u64) -> NoticeListFilter {
    NoticeListFilter {
        page: CursorPageRequest { limit, cursor: None },
        notice_title: Some("release".into()),
        create_by: Some("admin".into()),
        notice_type: Some("1".into()),
    }
}

fn record(timestamp: i64, id: &str) -> NoticeSummaryRecord {
    NoticeSummaryRecord {
        notice_id: id.into(),
        notice_title: "release".into(),
        notice_type: "1".into(),
        status: "0".into(),
        create_by: "admin".into(),
        create_time: OffsetDateTime::from_unix_timestamp(timestamp).unwrap(),
    }
}

fn context<'a>(codec: &'a NoticeCursorCodec, snapshot: &'a TimeIdPoint, window: &'a PageWindow) -> PageContext<'a> {
    PageContext { codec, snapshot, window }
}

#[test]
fn filtered_queries_use_native_time_keysets_without_offset() {
    let snapshot = point(OffsetDateTime::UNIX_EPOCH, "snapshot".into()).unwrap();
    let window = page_window(None, &snapshot, 20).unwrap();
    let mut notices = notice_query(&filter(20));
    push_window(
        &mut notices,
        &window,
        WindowColumns {
            time: "create_time",
            id: "notice_id",
        },
    )
    .unwrap();
    let mut readers = reader_query(
        "notice-1",
        &NoticeReaderFilter {
            page: CursorPageRequest::default(),
            search_value: Some("alice".into()),
        },
    );
    push_window(
        &mut readers,
        &window,
        WindowColumns {
            time: "r.read_time",
            id: "r.user_id",
        },
    )
    .unwrap();
    let notice_sql = notices.into_sql();
    let reader_sql = readers.into_sql();

    assert!(notice_sql.as_str().contains("notice_title ILIKE"));
    assert!(notice_sql.as_str().contains("(create_time,notice_id)<="));
    assert!(notice_sql.as_str().contains("ORDER BY create_time DESC,notice_id DESC"));
    assert!(reader_sql.as_str().contains("r.notice_id="));
    assert!(reader_sql.as_str().contains("u.user_name ILIKE"));
    assert!(reader_sql.as_str().contains("(r.read_time,r.user_id)<="));
    assert!(reader_sql.as_str().contains("ORDER BY r.read_time DESC,r.user_id DESC"));
    for sql in [notice_sql, reader_sql] {
        assert!(!sql.as_str().contains("::text AS"));
        assert!(!sql.as_str().contains("OFFSET"));
        assert!(!sql.as_str().contains("COUNT("));
    }
}

#[test]
fn descending_pages_are_symmetric_and_keep_duplicate_time_ids() {
    let filter = filter(2);
    let codec = NoticeCursorCodec::notices(&filter).unwrap();
    let snapshot = record(3, "notice-c").point().unwrap();
    let first_window = page_window(None, &snapshot, 2).unwrap();
    let first = build_page(
        vec![record(3, "notice-c"), record(3, "notice-b"), record(3, "notice-a")],
        context(&codec, &snapshot, &first_window),
    )
    .unwrap();

    assert_eq!(
        first.items.iter().map(|item| item.notice_id.as_str()).collect::<Vec<_>>(),
        ["notice-c", "notice-b"]
    );
    assert!(first.previous_cursor.is_none());
    let next = codec.decode(first.next_cursor.as_deref()).unwrap().unwrap();
    assert_eq!(next.boundary.id, "notice-b");

    let next_window = page_window(Some(&next), &snapshot, 2).unwrap();
    let second = build_page(vec![record(3, "notice-a")], context(&codec, &snapshot, &next_window)).unwrap();
    let previous = codec.decode(second.previous_cursor.as_deref()).unwrap().unwrap();
    assert_eq!(previous.direction, CursorDirection::Previous);

    let previous_window = page_window(Some(&previous), &snapshot, 2).unwrap();
    let restored = build_page(vec![record(3, "notice-b"), record(3, "notice-c")], context(&codec, &snapshot, &previous_window)).unwrap();
    assert_eq!(
        restored.items.iter().map(|item| item.notice_id.as_str()).collect::<Vec<_>>(),
        ["notice-c", "notice-b"]
    );
}

#[test]
fn empty_batch_preserves_reverse_recovery_cursor() {
    let filter = filter(2);
    let codec = NoticeCursorCodec::notices(&filter).unwrap();
    let snapshot = record(3, "notice-3").point().unwrap();
    let boundary = record(1, "notice-1").point().unwrap();
    let token = codec.encode(CursorDirection::Next, &boundary, &snapshot).unwrap();
    let decoded = codec.decode(Some(&token)).unwrap().unwrap();
    let window = page_window(Some(&decoded), &snapshot, 2).unwrap();
    let page = build_page::<NoticeSummaryRecord>(Vec::new(), context(&codec, &snapshot, &window)).unwrap();

    assert!(page.next_cursor.is_none());
    assert!(page.previous_cursor.is_some());
}
