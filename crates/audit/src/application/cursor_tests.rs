use kernel::pagination::{CursorDirection, CursorPageRequest};
use time::OffsetDateTime;

use crate::{
    application::{AuditCursorSlice, AuditError, AuditSnapshot, OperationCursorValue, operation_cursor_page, operation_cursor_query},
    domain::{AuditLocation, AuditStatus, BusinessType, OperationLogFilter, OperationLogSummary, OperatorType},
};

#[test]
fn cursor_round_trip_binds_filter_sort_limit_and_snapshot() {
    let filter = OperationLogFilter::default();
    let first_request = CursorPageRequest { limit: 1, cursor: None };
    let first_query = operation_cursor_query(&filter, &first_request).unwrap();
    let snapshot = AuditSnapshot::new(OffsetDateTime::UNIX_EPOCH, "snapshot-id".into());
    let page = operation_cursor_page(
        &filter,
        &first_query,
        AuditCursorSlice {
            items: vec![operation("row-1", 7)],
            snapshot: Some(snapshot.clone()),
            has_next: true,
            has_previous: false,
        },
    )
    .unwrap();
    let next_request = CursorPageRequest {
        limit: 1,
        cursor: page.next_cursor,
    };

    let decoded = operation_cursor_query(&filter, &next_request).unwrap();

    assert_eq!(decoded.direction, CursorDirection::Next);
    assert_eq!(decoded.snapshot, Some(snapshot));
    let boundary = decoded.boundary.unwrap();
    assert_eq!(boundary.id, "row-1");
    assert_eq!(boundary.value, OperationCursorValue::Time("0".into()));
}

#[test]
fn cursor_rejects_filter_limit_and_payload_mismatches() {
    let filter = OperationLogFilter::default();
    let first_request = CursorPageRequest { limit: 1, cursor: None };
    let first_query = operation_cursor_query(&filter, &first_request).unwrap();
    let page = operation_cursor_page(
        &filter,
        &first_query,
        AuditCursorSlice {
            items: vec![operation("row-1", 7)],
            snapshot: Some(AuditSnapshot::new(OffsetDateTime::UNIX_EPOCH, "snapshot-id".into())),
            has_next: true,
            has_previous: false,
        },
    )
    .unwrap();
    let cursor = page.next_cursor.unwrap();
    let mut changed_filter = filter.clone();
    changed_filter.operator_name = Some("other".into());

    assert!(matches!(
        operation_cursor_query(
            &changed_filter,
            &CursorPageRequest {
                limit: 1,
                cursor: Some(cursor.clone())
            }
        ),
        Err(AuditError::InvalidCursor)
    ));
    assert!(matches!(
        operation_cursor_query(
            &filter,
            &CursorPageRequest {
                limit: 2,
                cursor: Some(cursor)
            }
        ),
        Err(AuditError::InvalidCursor)
    ));
    assert!(matches!(
        operation_cursor_query(
            &filter,
            &CursorPageRequest {
                limit: 1,
                cursor: Some("malformed".into())
            }
        ),
        Err(AuditError::InvalidCursor)
    ));
}

#[test]
fn cursor_limit_is_validated_before_storage_access() {
    let filter = OperationLogFilter::default();
    for limit in [0, 101] {
        assert!(matches!(
            operation_cursor_query(&filter, &CursorPageRequest { limit, cursor: None }),
            Err(AuditError::InvalidInput(_))
        ));
    }
}

#[test]
fn empty_next_page_after_cursor_deletion_exposes_a_previous_cursor() {
    let filter = OperationLogFilter::default();
    let first_request = CursorPageRequest { limit: 1, cursor: None };
    let first_query = operation_cursor_query(&filter, &first_request).unwrap();
    let first_page = operation_cursor_page(
        &filter,
        &first_query,
        AuditCursorSlice {
            items: vec![operation("row-1", 7)],
            snapshot: Some(AuditSnapshot::new(OffsetDateTime::UNIX_EPOCH, "snapshot-id".into())),
            has_next: true,
            has_previous: false,
        },
    )
    .unwrap();
    let request = CursorPageRequest {
        limit: 1,
        cursor: first_page.next_cursor,
    };
    let query = operation_cursor_query(&filter, &request).unwrap();
    let page = operation_cursor_page(
        &filter,
        &query,
        AuditCursorSlice {
            items: Vec::new(),
            snapshot: query.snapshot.clone(),
            has_next: false,
            has_previous: false,
        },
    )
    .unwrap();

    assert!(page.items.is_empty());
    assert_eq!(page.next_cursor, None);
    assert!(page.previous_cursor.is_some());
    assert!(!page.has_next);
    assert!(page.has_previous);
    let recovery = operation_cursor_query(
        &filter,
        &CursorPageRequest {
            limit: 1,
            cursor: page.previous_cursor,
        },
    )
    .unwrap();
    assert_eq!(recovery.direction, CursorDirection::Previous);
}

#[test]
fn empty_previous_page_after_cursor_deletion_exposes_a_next_cursor() {
    let filter = OperationLogFilter::default();
    let first_request = CursorPageRequest { limit: 1, cursor: None };
    let first_query = operation_cursor_query(&filter, &first_request).unwrap();
    let page = operation_cursor_page(
        &filter,
        &first_query,
        AuditCursorSlice {
            items: vec![operation("row-1", 7)],
            snapshot: Some(AuditSnapshot::new(OffsetDateTime::UNIX_EPOCH, "snapshot-id".into())),
            has_next: false,
            has_previous: true,
        },
    )
    .unwrap();
    let request = CursorPageRequest {
        limit: 1,
        cursor: page.previous_cursor,
    };
    let query = operation_cursor_query(&filter, &request).unwrap();
    let page = operation_cursor_page(
        &filter,
        &query,
        AuditCursorSlice {
            items: Vec::new(),
            snapshot: query.snapshot.clone(),
            has_next: false,
            has_previous: false,
        },
    )
    .unwrap();

    assert!(page.items.is_empty());
    assert!(page.has_next);
    assert!(page.next_cursor.is_some());
    assert!(!page.has_previous);
}

fn operation(id: &str, cost_time_ms: i64) -> OperationLogSummary {
    OperationLogSummary {
        id: id.into(),
        title_key: "audit.module.user".into(),
        business_type: BusinessType::Update,
        handler: "user::update".into(),
        request_method: "PUT".into(),
        operator_type: OperatorType::Manage,
        operator_name: "admin".into(),
        department_name: "root".into(),
        operation_url: "/api/users/1".into(),
        operation_ip: "198.51.100.7".into(),
        operation_location: AuditLocation::Unknown,
        status: AuditStatus::Success,
        operation_time: OffsetDateTime::UNIX_EPOCH,
        cost_time_ms,
    }
}
