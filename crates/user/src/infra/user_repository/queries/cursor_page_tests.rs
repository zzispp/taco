use kernel::pagination::{CursorDirection, CursorPageRequest};
use time::OffsetDateTime;

use super::*;

#[test]
fn tuple_predicates_cover_equal_timestamps_with_id_tie_breakers() {
    let next = cursor_suffix(13, CursorDirection::Next);
    let previous = cursor_suffix(13, CursorDirection::Previous);

    assert!(next.contains("(u.create_time,u.user_id) > ($15,$16)"));
    assert!(next.contains("create_time ASC,u.user_id ASC"));
    assert!(previous.contains("(u.create_time,u.user_id) < ($15,$16)"));
    assert!(previous.contains("create_time DESC,u.user_id DESC"));
}

#[test]
fn deleted_forward_segment_keeps_a_previous_cursor() {
    let filter = filter();
    let codec = UserCursorCodec::new(&filter, None).unwrap();
    let boundary = UserCursorPoint {
        create_time_nanos: 10,
        user_id: "user-10".into(),
    };
    let snapshot = UserCursorPoint {
        create_time_nanos: 20,
        user_id: "user-20".into(),
    };
    let window = window(boundary.clone(), CursorDirection::Next);

    let cursors = empty_page_cursors(&codec, &snapshot, &window).unwrap();

    assert_eq!(cursors.next, None);
    let decoded = codec
        .decode(&CursorPageRequest {
            limit: 20,
            cursor: cursors.previous,
        })
        .unwrap()
        .unwrap();
    assert_eq!(decoded.direction, CursorDirection::Previous);
    assert_eq!(decoded.boundary, boundary);
}

fn window(boundary: UserCursorPoint, direction: CursorDirection) -> UserPageWindow {
    UserPageWindow {
        snapshot_time: OffsetDateTime::UNIX_EPOCH,
        snapshot_id: "user-20".into(),
        boundary_time: Some(OffsetDateTime::UNIX_EPOCH),
        boundary_id: Some(boundary.user_id.clone()),
        boundary: Some(boundary),
        direction,
        limit: 20,
        from_cursor: true,
    }
}

fn filter() -> UserListFilter {
    UserListFilter {
        page: CursorPageRequest::default(),
        username: None,
        nick_name: None,
        phonenumber: None,
        email: None,
        sex: None,
        status: None,
        dept_id: None,
        dept_name: None,
        post_ids: vec![],
        role_ids: vec![],
        begin_time: None,
        end_time: None,
    }
}
