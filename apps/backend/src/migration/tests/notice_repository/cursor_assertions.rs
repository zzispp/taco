use kernel::pagination::CursorPageRequest;
use system::notice::{NoticeListFilter, NoticeReader, NoticeReaderFilter, NoticeSummary, NoticeUseCase};

const NOTICE_BATCH_LIMIT: u64 = 2;
const READER_BATCH_LIMIT: u64 = 1;

pub(super) async fn assert_notice_cursor_order(service: &impl NoticeUseCase, expected: [&str; 3]) {
    let first = service.page_notices(unfiltered_notice_filter(None)).await.unwrap();
    assert_eq!(notice_ids(&first.items), expected[..2]);
    assert!(first.has_next);
    assert!(!first.has_previous);

    let second = service.page_notices(unfiltered_notice_filter(first.next_cursor.clone())).await.unwrap();
    assert_eq!(notice_ids(&second.items), expected[2..]);
    assert!(!second.has_next);
    assert!(second.has_previous);

    let returned = service.page_notices(unfiltered_notice_filter(second.previous_cursor.clone())).await.unwrap();
    assert_eq!(notice_ids(&returned.items), expected[..2]);
    assert!(returned.has_next);
    assert!(!returned.has_previous);
}

pub(super) async fn assert_reader_cursor_order(service: &impl NoticeUseCase, notice_id: &str) {
    let first = service.page_readers(notice_id, unfiltered_reader_filter(None)).await.unwrap();
    assert_eq!(reader_names(&first.items), ["admin"]);
    assert!(first.has_next);
    assert!(!first.has_previous);

    let second = service
        .page_readers(notice_id, unfiltered_reader_filter(first.next_cursor.clone()))
        .await
        .unwrap();
    assert_eq!(reader_names(&second.items), ["taco"]);
    assert!(!second.has_next);
    assert!(second.has_previous);

    let returned = service
        .page_readers(notice_id, unfiltered_reader_filter(second.previous_cursor.clone()))
        .await
        .unwrap();
    assert_eq!(reader_names(&returned.items), ["admin"]);
    assert!(returned.has_next);
    assert!(!returned.has_previous);
}

fn unfiltered_notice_filter(cursor: Option<String>) -> NoticeListFilter {
    NoticeListFilter {
        page: CursorPageRequest {
            limit: NOTICE_BATCH_LIMIT,
            cursor,
        },
        notice_title: None,
        create_by: None,
        notice_type: None,
    }
}

fn unfiltered_reader_filter(cursor: Option<String>) -> NoticeReaderFilter {
    NoticeReaderFilter {
        page: CursorPageRequest {
            limit: READER_BATCH_LIMIT,
            cursor,
        },
        search_value: None,
    }
}

fn notice_ids(items: &[NoticeSummary]) -> Vec<&str> {
    items.iter().map(|item| item.notice_id.as_str()).collect()
}

fn reader_names(items: &[NoticeReader]) -> Vec<&str> {
    items.iter().map(|item| item.user_name.as_str()).collect()
}
