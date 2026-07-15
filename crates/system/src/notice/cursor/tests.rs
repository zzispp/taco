use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use kernel::pagination::{CursorDirection, CursorPageRequest};
use serde_json::Value;

use super::*;

fn notice_filter() -> NoticeListFilter {
    NoticeListFilter {
        page: CursorPageRequest { limit: 10, cursor: None },
        notice_title: Some("release".into()),
        create_by: None,
        notice_type: Some("1".into()),
    }
}

fn reader_filter() -> NoticeReaderFilter {
    NoticeReaderFilter {
        page: CursorPageRequest { limit: 10, cursor: None },
        search_value: Some("alice".into()),
    }
}

fn point(micros: i64, id: &str) -> TimeIdPoint {
    TimeIdPoint {
        time_micros: micros,
        id: id.into(),
    }
}

#[test]
fn cursor_binds_notice_filters_and_limit() {
    let codec = NoticeCursorCodec::notices(&notice_filter()).unwrap();
    let token = codec.encode(CursorDirection::Next, &point(1, "notice-1"), &point(2, "notice-2")).unwrap();
    let mut changed_filter = notice_filter();
    changed_filter.notice_type = Some("2".into());
    let mut changed_limit = notice_filter();
    changed_limit.page.limit = 11;

    assert!(matches!(
        NoticeCursorCodec::notices(&changed_filter).unwrap().decode(Some(&token)),
        Err(SystemError::InvalidCursor)
    ));
    assert!(matches!(
        NoticeCursorCodec::notices(&changed_limit).unwrap().decode(Some(&token)),
        Err(SystemError::InvalidCursor)
    ));
}

#[test]
fn reader_cursor_binds_notice_scope_and_filter() {
    let codec = NoticeCursorCodec::readers("notice-1", &reader_filter()).unwrap();
    let token = codec.encode(CursorDirection::Previous, &point(1, "user-1"), &point(2, "user-2")).unwrap();

    assert!(matches!(
        NoticeCursorCodec::readers("notice-2", &reader_filter()).unwrap().decode(Some(&token)),
        Err(SystemError::InvalidCursor)
    ));
    let mut changed = reader_filter();
    changed.search_value = Some("bob".into());
    assert!(matches!(
        NoticeCursorCodec::readers("notice-1", &changed).unwrap().decode(Some(&token)),
        Err(SystemError::InvalidCursor)
    ));
}

#[test]
fn malformed_and_semantically_invalid_cursors_are_rejected() {
    let codec = NoticeCursorCodec::notices(&notice_filter()).unwrap();
    assert!(matches!(codec.decode(Some("not-a-cursor")), Err(SystemError::InvalidCursor)));

    let valid = codec.encode(CursorDirection::Next, &point(1, "notice-1"), &point(2, "notice-2")).unwrap();
    assert!(matches!(codec.decode(Some(&mutate_version(&valid))), Err(SystemError::InvalidCursor)));
    let empty_id = codec.encode(CursorDirection::Next, &point(1, ""), &point(2, "notice-2")).unwrap();
    let invalid_time = codec
        .encode(CursorDirection::Next, &point(i64::MAX, "notice-1"), &point(2, "notice-2"))
        .unwrap();
    let boundary_after_snapshot = codec.encode(CursorDirection::Next, &point(3, "notice-3"), &point(2, "notice-2")).unwrap();
    assert!(matches!(codec.decode(Some(&empty_id)), Err(SystemError::InvalidCursor)));
    assert!(matches!(codec.decode(Some(&invalid_time)), Err(SystemError::InvalidCursor)));
    assert!(matches!(codec.decode(Some(&boundary_after_snapshot)), Err(SystemError::InvalidCursor)));
}

fn mutate_version(cursor: &str) -> String {
    let bytes = URL_SAFE_NO_PAD.decode(cursor).unwrap();
    let mut payload = serde_json::from_slice::<Value>(&bytes).unwrap();
    payload["version"] = Value::from(99);
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap())
}
