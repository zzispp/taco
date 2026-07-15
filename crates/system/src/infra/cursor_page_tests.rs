use kernel::pagination::{CursorDirection, CursorPage, CursorPageRequest};

use super::{
    cursor_page::{PageBuildContext, PageNavigation, build_page, navigation},
    record::PostRecord,
};
use crate::application::{PostListFilter, SystemCursorCodec, SystemDecodedCursor, TimeIdPoint, point};
use crate::domain::Post;

#[test]
fn pages_are_symmetric_when_business_sort_values_match() {
    let filter = filter();
    let codec = SystemCursorCodec::post(&filter).unwrap();
    let snapshot = point(time::OffsetDateTime::UNIX_EPOCH, "4".into()).unwrap();
    let first_navigation = navigation(None, filter.page.limit);
    let first = build_page(records(&["1", "2", "3"]), context(&codec, &snapshot, &first_navigation)).unwrap();
    assert_eq!(ids(&first), ["1", "2"]);
    assert!(first.previous_cursor.is_none());

    let next = decode(&codec, first.next_cursor.as_deref().unwrap(), filter.page.limit);
    let next_navigation = navigation(Some(&next), filter.page.limit);
    let second = build_page(records(&["3", "4"]), context(&codec, &snapshot, &next_navigation)).unwrap();
    assert_eq!(ids(&second), ["3", "4"]);
    assert!(second.next_cursor.is_none());

    let previous = decode(&codec, second.previous_cursor.as_deref().unwrap(), filter.page.limit);
    let previous_navigation = navigation(Some(&previous), filter.page.limit);
    let returned = build_page(records(&["2", "1"]), context(&codec, &snapshot, &previous_navigation)).unwrap();
    assert_eq!(ids(&returned), ["1", "2"]);
    assert!(returned.previous_cursor.is_none());
    assert!(returned.next_cursor.is_some());
}

#[test]
fn empty_forward_batch_preserves_reverse_recovery_cursor() {
    let filter = filter();
    let codec = SystemCursorCodec::post(&filter).unwrap();
    let snapshot = point(time::OffsetDateTime::UNIX_EPOCH, "4".into()).unwrap();
    let cursor = codec
        .encode(
            CursorDirection::Next,
            &crate::application::SystemBoundary::Post {
                post_sort: 1,
                post_id: "4".into(),
            },
            &snapshot,
        )
        .unwrap();
    let decoded = decode(&codec, &cursor, filter.page.limit);
    let page_navigation = navigation(Some(&decoded), filter.page.limit);
    let page = build_page::<PostRecord>(Vec::new(), context(&codec, &snapshot, &page_navigation)).unwrap();

    assert!(page.next_cursor.is_none());
    assert!(page.previous_cursor.is_some());
}

fn context<'a>(codec: &'a SystemCursorCodec, snapshot: &'a TimeIdPoint, navigation: &'a PageNavigation) -> PageBuildContext<'a> {
    PageBuildContext { codec, snapshot, navigation }
}

fn decode(codec: &SystemCursorCodec, cursor: &str, limit: u64) -> SystemDecodedCursor {
    codec
        .decode(&CursorPageRequest {
            limit,
            cursor: Some(cursor.into()),
        })
        .unwrap()
        .unwrap()
}

fn filter() -> PostListFilter {
    PostListFilter {
        page: CursorPageRequest { limit: 2, cursor: None },
        post_code: None,
        post_name: None,
        status: None,
        remark: None,
        begin_time: None,
        end_time: None,
    }
}

fn records(ids: &[&str]) -> Vec<PostRecord> {
    ids.iter().map(|id| record(id)).collect()
}

fn record(id: &str) -> PostRecord {
    PostRecord {
        post_id: id.into(),
        post_code: format!("post-{id}"),
        post_name: format!("Post {id}"),
        post_sort: 1,
        status: "0".into(),
        remark: None,
        create_time: time::OffsetDateTime::UNIX_EPOCH,
    }
}

fn ids(page: &CursorPage<Post>) -> Vec<&str> {
    page.items.iter().map(|post| post.post_id.as_str()).collect()
}
