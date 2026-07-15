use kernel::pagination::CursorPageRequest;

use super::*;
use crate::application::{RoleUserListFilter, cursor::RoleUserCursorCodec};

#[test]
fn cursor_pages_are_symmetric_with_equal_timestamps() {
    let filter = role_user_filter();
    let codec = RoleUserCursorCodec::new(&filter, None).unwrap();
    let snapshot = point(time::OffsetDateTime::UNIX_EPOCH, "4".into()).unwrap();
    let first_navigation = navigation::<TimeIdPoint, TimeIdPoint>(None, filter.page.limit);
    let first = build_page(records(&["1", "2", "3"]), context(&codec, &snapshot, &first_navigation)).unwrap();
    assert_eq!(user_ids(&first), ["1", "2"]);
    assert!(first.previous_cursor.is_none());

    let next = decode(&codec, first.next_cursor.as_deref().unwrap(), filter.page.limit);
    let next_navigation = navigation(Some(&next), filter.page.limit);
    let second = build_page(records(&["3", "4"]), context(&codec, &snapshot, &next_navigation)).unwrap();
    assert_eq!(user_ids(&second), ["3", "4"]);
    assert!(second.next_cursor.is_none());

    let previous = decode(&codec, second.previous_cursor.as_deref().unwrap(), filter.page.limit);
    let previous_navigation = navigation(Some(&previous), filter.page.limit);
    let returned = build_page(records(&["2", "1"]), context(&codec, &snapshot, &previous_navigation)).unwrap();
    assert_eq!(user_ids(&returned), ["1", "2"]);
    assert!(returned.previous_cursor.is_none());
    assert!(returned.next_cursor.is_some());
}

#[test]
fn empty_forward_batch_preserves_reverse_recovery_cursor() {
    let filter = role_user_filter();
    let codec = RoleUserCursorCodec::new(&filter, None).unwrap();
    let snapshot = point(time::OffsetDateTime::UNIX_EPOCH, "4".into()).unwrap();
    let cursor = codec.encode(CursorDirection::Next, &snapshot, &snapshot).unwrap();
    let decoded = decode(&codec, &cursor, filter.page.limit);
    let page_navigation = navigation(Some(&decoded), filter.page.limit);

    let page = build_page::<RoleUserRecord, _, _, _>(Vec::new(), context(&codec, &snapshot, &page_navigation)).unwrap();

    assert!(page.next_cursor.is_none());
    assert!(page.previous_cursor.is_some());
}

fn context<'a>(
    codec: &'a RoleUserCursorCodec,
    snapshot: &'a TimeIdPoint,
    navigation: &'a PageNavigation<TimeIdPoint>,
) -> PageBuildContext<'a, TimeIdPoint, TimeIdPoint, RoleUserCursorCodec> {
    PageBuildContext { codec, snapshot, navigation }
}

fn decode(codec: &RoleUserCursorCodec, cursor: &str, limit: u64) -> crate::application::cursor::RoleUserCursor {
    codec
        .decode(&CursorPageRequest {
            limit,
            cursor: Some(cursor.into()),
        })
        .unwrap()
        .unwrap()
}

fn role_user_filter() -> RoleUserListFilter {
    RoleUserListFilter {
        page: CursorPageRequest { limit: 2, cursor: None },
        role_id: "role-1".into(),
        username: None,
        phonenumber: None,
        allocated: true,
    }
}

fn records(ids: &[&str]) -> Vec<RoleUserRecord> {
    ids.iter().map(|id| record(id)).collect()
}

fn record(id: &str) -> RoleUserRecord {
    RoleUserRecord {
        user_id: id.into(),
        username: format!("user-{id}"),
        nick_name: format!("User {id}"),
        dept_id: None,
        phonenumber: None,
        email: format!("user-{id}@example.com"),
        status: "0".into(),
        create_time: time::OffsetDateTime::UNIX_EPOCH,
    }
}

fn user_ids(page: &CursorPage<RoleUser>) -> Vec<&str> {
    page.items.iter().map(|user| user.user_id.as_str()).collect()
}
