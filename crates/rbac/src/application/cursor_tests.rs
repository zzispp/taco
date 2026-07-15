use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use kernel::pagination::{CursorDirection, CursorPageRequest};
use serde_json::Value;

use super::*;
use crate::domain::{DataScope, DataScopeFilter};

#[test]
fn role_cursor_rejects_filter_scope_and_limit_changes() {
    let filter = role_filter("0", 20);
    let scope = self_scope("user-1");
    let codec = RoleCursorCodec::new(&filter, Some(&scope)).unwrap();
    let cursor = codec.encode(CursorDirection::Next, &role_boundary("role-1"), &valid_point("role-9")).unwrap();

    assert_invalid(RoleCursorCodec::new(&role_filter("1", 20), Some(&scope)).unwrap(), &cursor, 20);
    assert_invalid(RoleCursorCodec::new(&filter, Some(&self_scope("user-2"))).unwrap(), &cursor, 20);
    assert_invalid(RoleCursorCodec::new(&role_filter("0", 50), Some(&scope)).unwrap(), &cursor, 50);
}

#[test]
fn cursor_rejects_malformed_and_unsupported_protocol_versions() {
    let filter = role_filter("0", 20);
    let codec = RoleCursorCodec::new(&filter, None).unwrap();
    assert_invalid(codec, "not-a-cursor", 20);

    let codec = RoleCursorCodec::new(&filter, None).unwrap();
    let valid = codec.encode(CursorDirection::Next, &role_boundary("role-1"), &valid_point("role-9")).unwrap();
    let unsupported = mutate_payload(&valid, |payload| payload["version"] = Value::from(99));
    assert_invalid(codec, &unsupported, 20);
}

#[test]
fn cursor_rejects_semantically_invalid_points() {
    let filter = role_filter("0", 20);
    let codec = RoleCursorCodec::new(&filter, None).unwrap();
    let blank_id = codec.encode(CursorDirection::Next, &role_boundary(" "), &valid_point("role-9")).unwrap();
    assert_invalid(codec, &blank_id, 20);

    let codec = RoleCursorCodec::new(&filter, None).unwrap();
    let out_of_range = TimeIdPoint {
        time_micros: i64::MAX,
        id: "role-1".into(),
    };
    let cursor = codec.encode(CursorDirection::Next, &role_boundary("role-1"), &out_of_range).unwrap();
    assert_invalid(codec, &cursor, 20);
}

fn assert_invalid(codec: RoleCursorCodec, cursor: &str, limit: u64) {
    let request = CursorPageRequest {
        limit,
        cursor: Some(cursor.into()),
    };
    assert!(matches!(codec.decode(&request), Err(RbacError::InvalidCursor)));
}

fn mutate_payload(cursor: &str, mutate: impl FnOnce(&mut Value)) -> String {
    let bytes = URL_SAFE_NO_PAD.decode(cursor).unwrap();
    let mut payload = serde_json::from_slice::<Value>(&bytes).unwrap();
    mutate(&mut payload);
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap())
}

fn valid_point(id: &str) -> TimeIdPoint {
    TimeIdPoint { time_micros: 0, id: id.into() }
}

fn role_boundary(id: &str) -> RoleBoundary {
    RoleBoundary {
        role_sort: 1,
        role_id: id.into(),
    }
}

fn role_filter(status: &str, limit: u64) -> RoleListFilter {
    RoleListFilter {
        page: CursorPageRequest { limit, cursor: None },
        role_name: None,
        role_key: None,
        status: Some(status.into()),
        system: None,
        begin_time: None,
        end_time: None,
    }
}

fn self_scope(user_id: &str) -> DataScopeFilter {
    DataScopeFilter {
        data_scope: DataScope::SelfOnly,
        user_id: user_id.into(),
        dept_id: Some("dept-1".into()),
        dept_ids: Vec::new(),
    }
}
