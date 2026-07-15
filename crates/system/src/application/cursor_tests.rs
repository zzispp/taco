use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use kernel::pagination::{CursorDirection, CursorPageRequest};
use rbac::domain::{DataScope, DataScopeFilter};
use serde_json::Value;

use super::*;

#[test]
fn cursor_binds_resource_filters_scope_and_limit() {
    let filter = post_filter("0", 20);
    let codec = SystemCursorCodec::post(&filter).unwrap();
    let token = codec.encode(CursorDirection::Next, &post_boundary("post-1"), &point("post-9")).unwrap();

    assert_invalid(SystemCursorCodec::post(&post_filter("1", 20)).unwrap(), &token, 20);
    assert_invalid(SystemCursorCodec::post(&post_filter("0", 50)).unwrap(), &token, 50);
    assert_invalid(SystemCursorCodec::config(&config_filter(20)).unwrap(), &token, 20);

    let dept_filter = dept_filter();
    let dept_codec = SystemCursorCodec::dept(&dept_filter, Some(&scope("user-1"))).unwrap();
    let dept_token = dept_codec.encode(CursorDirection::Next, &dept_boundary("dept-1"), &point("dept-9")).unwrap();
    assert_invalid(SystemCursorCodec::dept(&dept_filter, Some(&scope("user-2"))).unwrap(), &dept_token, 20);
}

#[test]
fn malformed_unsupported_and_semantically_invalid_cursors_are_rejected() {
    let codec = SystemCursorCodec::post(&post_filter("0", 20)).unwrap();
    assert_invalid(codec, "not-a-cursor", 20);

    let codec = SystemCursorCodec::post(&post_filter("0", 20)).unwrap();
    let valid = codec.encode(CursorDirection::Next, &post_boundary("post-1"), &point("post-9")).unwrap();
    assert_invalid(codec, &mutate_version(&valid), 20);

    let codec = SystemCursorCodec::post(&post_filter("0", 20)).unwrap();
    let wrong_kind = codec.encode(CursorDirection::Next, &dept_boundary("dept-1"), &point("post-9")).unwrap();
    assert_invalid(codec, &wrong_kind, 20);

    let codec = SystemCursorCodec::post(&post_filter("0", 20)).unwrap();
    let blank_id = codec.encode(CursorDirection::Next, &post_boundary(" "), &point("post-9")).unwrap();
    assert_invalid(codec, &blank_id, 20);

    let codec = SystemCursorCodec::post(&post_filter("0", 20)).unwrap();
    let invalid_snapshot = TimeIdPoint {
        time_micros: i64::MAX,
        id: "post-9".into(),
    };
    let invalid_time = codec.encode(CursorDirection::Next, &post_boundary("post-1"), &invalid_snapshot).unwrap();
    assert_invalid(codec, &invalid_time, 20);

    let dept_filter = dept_filter();
    let codec = SystemCursorCodec::dept(&dept_filter, None).unwrap();
    let invalid_parent = SystemBoundary::Dept {
        parent_id: " ".into(),
        order_num: 1,
        dept_id: "dept-1".into(),
    };
    let invalid_parent = codec.encode(CursorDirection::Next, &invalid_parent, &point("dept-9")).unwrap();
    assert_invalid(codec, &invalid_parent, 20);
}

fn assert_invalid(codec: SystemCursorCodec, cursor: &str, limit: u64) {
    let request = CursorPageRequest {
        limit,
        cursor: Some(cursor.into()),
    };
    assert!(matches!(codec.decode(&request), Err(SystemError::InvalidCursor)));
}

fn mutate_version(cursor: &str) -> String {
    let bytes = URL_SAFE_NO_PAD.decode(cursor).unwrap();
    let mut payload = serde_json::from_slice::<Value>(&bytes).unwrap();
    payload["version"] = Value::from(99);
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload).unwrap())
}

fn point(id: &str) -> TimeIdPoint {
    TimeIdPoint { time_micros: 0, id: id.into() }
}

fn post_boundary(id: &str) -> SystemBoundary {
    SystemBoundary::Post {
        post_sort: 1,
        post_id: id.into(),
    }
}

fn dept_boundary(id: &str) -> SystemBoundary {
    SystemBoundary::Dept {
        parent_id: "0".into(),
        order_num: 1,
        dept_id: id.into(),
    }
}

fn post_filter(status: &str, limit: u64) -> PostListFilter {
    PostListFilter {
        page: CursorPageRequest { limit, cursor: None },
        post_code: None,
        post_name: None,
        status: Some(status.into()),
        remark: None,
        begin_time: None,
        end_time: None,
    }
}

fn config_filter(limit: u64) -> ConfigListFilter {
    ConfigListFilter {
        page: CursorPageRequest { limit, cursor: None },
        config_name: None,
        config_key: None,
        config_type: None,
        public_read: None,
        begin_time: None,
        end_time: None,
    }
}

fn dept_filter() -> DeptListFilter {
    DeptListFilter {
        page: CursorPageRequest { limit: 20, cursor: None },
        dept_name: None,
        leader: None,
        phone: None,
        email: None,
        status: None,
        begin_time: None,
        end_time: None,
    }
}

fn scope(user_id: &str) -> DataScopeFilter {
    DataScopeFilter {
        data_scope: DataScope::SelfOnly,
        user_id: user_id.into(),
        dept_id: Some("dept-1".into()),
        dept_ids: Vec::new(),
    }
}
