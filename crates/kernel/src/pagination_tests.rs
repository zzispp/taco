use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};

use super::{CursorContext, CursorDecodeError, CursorDirection, CursorPage, CursorPageRequest, CursorRequestError, cursor_fingerprint, decode_cursor};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
struct Boundary {
    created_at: String,
    id: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
struct Snapshot {
    created_at: String,
    id: i64,
}

#[test]
fn cursor_round_trip_preserves_boundary_snapshot_and_direction() {
    let cursor_context = context(20);
    let boundary = Boundary {
        created_at: "2026-07-15T01:02:03.004Z".to_owned(),
        id: 42,
    };
    let snapshot = Snapshot {
        created_at: "2026-07-15T02:00:00.000Z".to_owned(),
        id: 99,
    };

    let cursor = cursor_context.encode(CursorDirection::Next, &boundary, &snapshot).unwrap();
    let decoded = decode_cursor::<Boundary, Snapshot>(&cursor, &cursor_context).unwrap();

    assert_eq!(decoded.direction, CursorDirection::Next);
    assert_eq!(decoded.boundary, boundary);
    assert_eq!(decoded.snapshot, snapshot);
    assert!(!cursor.contains('='));
}

#[test]
fn cursor_rejects_malformed_version_and_context_mismatch() {
    let cursor_context = context(20);
    let boundary = Boundary {
        created_at: "2026-07-15T01:02:03.004Z".to_owned(),
        id: 42,
    };
    let snapshot = Snapshot {
        created_at: "2026-07-15T02:00:00.000Z".to_owned(),
        id: 99,
    };
    let cursor = cursor_context.encode(CursorDirection::Previous, &boundary, &snapshot).unwrap();
    let unsupported = cursor_with_version(&cursor, 2);

    assert_eq!(
        decode_cursor::<Boundary, Snapshot>("not base64!", &cursor_context),
        Err(CursorDecodeError::Malformed)
    );
    assert_eq!(
        decode_cursor::<Boundary, Snapshot>(&unsupported, &cursor_context),
        Err(CursorDecodeError::UnsupportedVersion)
    );
    assert_eq!(
        decode_cursor::<Boundary, Snapshot>(&cursor, &context(50)),
        Err(CursorDecodeError::ContextMismatch)
    );
}

#[test]
fn fingerprint_is_stable_for_equivalent_serializable_values() {
    let left = serde_json::json!({"status": "active", "name": "alice"});
    let right = serde_json::json!({"name": "alice", "status": "active"});

    assert_eq!(cursor_fingerprint(&left).unwrap(), cursor_fingerprint(&right).unwrap());
}

#[test]
fn cursor_wire_contract_uses_limit_and_bidirectional_links() {
    let request = CursorPageRequest::default();
    assert_eq!(request.limit, 20);
    assert_eq!(request.cursor, None);
    assert_eq!(request.validate(), Ok(()));
    assert_eq!(CursorPageRequest { limit: 0, cursor: None }.validate(), Err(CursorRequestError::InvalidLimit));
    assert_eq!(CursorPageRequest { limit: 101, cursor: None }.validate(), Err(CursorRequestError::InvalidLimit));

    let page = CursorPage::new(vec![1, 2], Some("next".to_owned()), None);
    assert_eq!(page.items, vec![1, 2]);
    assert_eq!(page.next_cursor.as_deref(), Some("next"));
    assert_eq!(page.previous_cursor, None);
    assert!(page.has_next);
    assert!(!page.has_previous);
    assert_eq!(
        serde_json::to_value(page).unwrap(),
        serde_json::json!({
            "items": [1, 2],
            "next_cursor": "next",
            "previous_cursor": null,
            "has_next": true,
            "has_previous": false
        })
    );
}

fn cursor_with_version(cursor: &str, version: u8) -> String {
    let bytes = URL_SAFE_NO_PAD.decode(cursor).unwrap();
    let mut value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    value["version"] = version.into();
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(&value).unwrap())
}

fn context(limit: u64) -> CursorContext<'static> {
    CursorContext {
        resource: "audit.operations",
        sort: "created_at:desc,id:desc",
        filter_fingerprint: "filter",
        scope_fingerprint: "scope",
        limit,
    }
}
