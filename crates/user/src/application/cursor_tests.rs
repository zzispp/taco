use kernel::pagination::{CursorDirection, CursorPageRequest};
use rbac::domain::{DataScope, DataScopeFilter};

use super::{
    AppError, OnlineCursorCodec, OnlineCursorPoint, OnlineCursorSnapshot, OnlineDecodedCursor, OnlineSessionPageRequest, UserCursorCodec, UserCursorPoint,
    UserListFilter,
};
use crate::application::OnlineSessionSearch;

#[test]
fn user_cursor_rejects_filter_scope_and_limit_changes() {
    let filter = user_filter("0", 20);
    let first_scope = scope("user-1");
    let codec = UserCursorCodec::new(&filter, Some(&first_scope)).unwrap();
    let point = UserCursorPoint {
        create_time_nanos: 1,
        user_id: "user-1".into(),
    };
    let cursor = codec.encode(CursorDirection::Next, &point, &point).unwrap();

    assert_invalid(&UserCursorCodec::new(&user_filter("1", 20), Some(&first_scope)).unwrap(), &cursor);
    assert_invalid(&UserCursorCodec::new(&filter, Some(&scope("user-2"))).unwrap(), &cursor);
    assert_invalid(&UserCursorCodec::new(&user_filter("0", 50), Some(&first_scope)).unwrap(), &cursor);
}

#[test]
fn user_cursor_rejects_empty_boundary_and_snapshot_ids() {
    let filter = user_filter("0", 20);
    let codec = UserCursorCodec::new(&filter, None).unwrap();
    let valid = UserCursorPoint {
        create_time_nanos: 1,
        user_id: "user-1".into(),
    };
    let empty = UserCursorPoint {
        create_time_nanos: 1,
        user_id: String::new(),
    };

    assert_invalid(&codec, &codec.encode(CursorDirection::Next, &empty, &valid).unwrap());
    assert_invalid(&codec, &codec.encode(CursorDirection::Next, &valid, &empty).unwrap());
}

#[test]
fn malformed_online_cursor_is_explicitly_invalid() {
    let codec = OnlineCursorCodec::new(&online_request()).unwrap();

    assert!(matches!(decode_online(&codec, "broken"), Err(AppError::InvalidCursor)));
}

#[test]
fn online_cursor_rejects_empty_ids_and_out_of_range_milliseconds() {
    let codec = OnlineCursorCodec::new(&online_request()).unwrap();
    let valid = online_point(1, "token-1");
    let empty = online_point(1, "");
    let extreme = online_point(i64::MAX, "token-1");

    assert_invalid_online(invalid_online_case(&codec, (&empty, &valid), 1));
    assert_invalid_online(invalid_online_case(&codec, (&valid, &empty), 1));
    assert_invalid_online(invalid_online_case(&codec, (&extreme, &valid), 1));
    assert_invalid_online(invalid_online_case(&codec, (&valid, &extreme), 1));
    assert_invalid_online(invalid_online_case(&codec, (&valid, &valid), i64::MAX));
}

fn assert_invalid(codec: &UserCursorCodec, cursor: &str) {
    let request = CursorPageRequest {
        limit: codec.limit,
        cursor: Some(cursor.into()),
    };
    assert!(matches!(codec.decode(&request), Err(AppError::InvalidCursor)));
}

struct InvalidOnlineCase<'a> {
    codec: &'a OnlineCursorCodec,
    boundary: &'a OnlineCursorPoint,
    head: &'a OnlineCursorPoint,
    as_of_millis: i64,
}

fn invalid_online_case<'a>(codec: &'a OnlineCursorCodec, points: (&'a OnlineCursorPoint, &'a OnlineCursorPoint), as_of_millis: i64) -> InvalidOnlineCase<'a> {
    InvalidOnlineCase {
        codec,
        boundary: points.0,
        head: points.1,
        as_of_millis,
    }
}

fn assert_invalid_online(case: InvalidOnlineCase<'_>) {
    let snapshot = OnlineCursorSnapshot {
        as_of_millis: case.as_of_millis,
        head: case.head.clone(),
    };
    let cursor = case.codec.encode(CursorDirection::Next, case.boundary, &snapshot).unwrap();
    assert!(matches!(decode_online(case.codec, &cursor), Err(AppError::InvalidCursor)));
}

fn decode_online(codec: &OnlineCursorCodec, cursor: &str) -> Result<Option<OnlineDecodedCursor>, AppError> {
    codec.decode(&CursorPageRequest {
        limit: 20,
        cursor: Some(cursor.into()),
    })
}

fn online_point(login_time_millis: i64, token_id: &str) -> OnlineCursorPoint {
    OnlineCursorPoint {
        login_time_millis,
        token_id: token_id.into(),
    }
}

fn user_filter(status: &str, limit: u64) -> UserListFilter {
    UserListFilter {
        page: CursorPageRequest { limit, cursor: None },
        username: None,
        nick_name: None,
        phonenumber: None,
        email: None,
        sex: None,
        status: Some(status.into()),
        dept_id: None,
        dept_name: None,
        post_ids: vec![],
        role_ids: vec![],
        begin_time: None,
        end_time: None,
    }
}

fn scope(user_id: &str) -> DataScopeFilter {
    DataScopeFilter {
        data_scope: DataScope::SelfOnly,
        user_id: user_id.into(),
        dept_id: Some("dept-1".into()),
        dept_ids: vec![],
    }
}

fn online_request() -> OnlineSessionPageRequest {
    OnlineSessionPageRequest {
        page: CursorPageRequest::default(),
        search: OnlineSessionSearch::default(),
        scope: None,
    }
}
