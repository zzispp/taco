use kernel::pagination::{CursorDirection, CursorPageRequest};
use time::OffsetDateTime;

use super::*;
use crate::application::{OnlineSessionPageRequest, OnlineSessionSearch};

#[test]
fn deleted_previous_segment_keeps_a_next_cursor() {
    let request = OnlineSessionPageRequest {
        page: CursorPageRequest::default(),
        search: OnlineSessionSearch::default(),
        scope: None,
    };
    let codec = OnlineCursorCodec::new(&request).unwrap();
    let boundary = OnlineCursorPoint {
        login_time_millis: 10,
        token_id: "token-10".into(),
    };
    let snapshot = OnlineCursorSnapshot {
        as_of_millis: 20,
        head: OnlineCursorPoint {
            login_time_millis: 20,
            token_id: "token-20".into(),
        },
    };
    let window = window(boundary.clone(), CursorDirection::Previous);

    let (next, previous) = empty_page_cursors(&codec, &snapshot, &window).unwrap();

    assert_eq!(previous, None);
    let decoded = codec.decode(&CursorPageRequest { limit: 20, cursor: next }).unwrap().unwrap();
    assert_eq!(decoded.direction, CursorDirection::Next);
    assert_eq!(decoded.boundary, boundary);
}

fn window(boundary: OnlineCursorPoint, direction: CursorDirection) -> OnlinePageWindow {
    OnlinePageWindow {
        as_of: OffsetDateTime::UNIX_EPOCH,
        head_time: OffsetDateTime::UNIX_EPOCH,
        head_id: "token-20".into(),
        boundary_time: Some(OffsetDateTime::UNIX_EPOCH),
        boundary_id: Some(boundary.token_id.clone()),
        boundary: Some(boundary),
        direction,
        limit: 20,
        from_cursor: true,
    }
}
