use std::collections::BTreeMap;

use crate::domain::{AuditLocation, AuditStatus, BusinessType, LoginEventType, LoginLog, OperationLogSummary, OperatorType};

use super::{ExportCursor, LOGIN_HEADERS, OPERATION_HEADERS, event_key};

#[test]
fn export_cursor_advances_with_a_stable_snapshot_and_no_offset() {
    let mut cursor = ExportCursor::<String>::new(2).unwrap();
    let snapshot = crate::application::AuditSnapshot {
        ingested_at_nanos: "0".into(),
        id: "snapshot-id".into(),
    };
    assert!(cursor.advance("row-2".into(), Some(snapshot.clone()), true).unwrap());
    let request = cursor.request().unwrap();
    assert_eq!(request.boundary.as_deref(), Some("row-2"));
    assert_eq!(request.snapshot, Some(snapshot));
    assert_eq!(request.limit, 2);
}

#[test]
fn operation_export_schema_excludes_permission_guarded_details() {
    for detail_header in [
        "excel.audit.operation.headers.request",
        "excel.audit.operation.headers.response",
        "excel.audit.operation.headers.error",
    ] {
        assert!(!OPERATION_HEADERS.contains(&detail_header));
    }
}

#[test]
fn operation_export_rows_match_the_summary_schema() {
    let summary = OperationLogSummary {
        id: "operation-id".into(),
        title_key: "audit.module.user".into(),
        business_type: BusinessType::Update,
        handler: "user::update".into(),
        request_method: "PUT".into(),
        operator_type: OperatorType::Manage,
        operator_name: "admin".into(),
        department_name: "root".into(),
        operation_url: "/api/system/users/1".into(),
        operation_ip: "198.51.100.7".into(),
        operation_location: AuditLocation::Resolved("test".into()),
        status: AuditStatus::Success,
        operation_time: time::OffsetDateTime::UNIX_EPOCH,
        cost_time_ms: 12,
    };

    let row = super::operation_row(summary, types::http::Locale::ZhCn).unwrap();

    assert_eq!(row.len(), OPERATION_HEADERS.len());
    assert_eq!(row[0], "operation-id");
}

#[test]
fn login_export_rows_include_the_ruoyi_primary_key() {
    let value = LoginLog {
        id: "login-id".into(),
        user_id: None,
        username: "admin".into(),
        ip_address: "198.51.100.7".into(),
        login_location: AuditLocation::Resolved("test".into()),
        browser: "Chrome".into(),
        os: "macOS".into(),
        status: AuditStatus::Success,
        event_type: LoginEventType::LoginSuccess,
        message_key: "audit.event_type.login_success".into(),
        message_params: BTreeMap::new(),
        login_time: time::OffsetDateTime::UNIX_EPOCH,
    };

    let row = super::login_row(value, types::http::Locale::ZhCn).unwrap();

    assert_eq!(row.len(), LOGIN_HEADERS.len());
    assert_eq!(row[0], "login-id");
}

#[test]
fn login_event_export_keys_cover_all_stable_event_types() {
    assert_eq!(
        LoginEventType::ALL.map(event_key),
        [
            "audit.event_type.login_success",
            "audit.event_type.login_failure",
            "audit.event_type.register_success",
            "audit.event_type.register_failure",
            "audit.event_type.logout_success",
            "audit.event_type.logout_failure",
            "audit.event_type.refresh_success",
            "audit.event_type.refresh_failure",
        ]
    );
}

#[test]
fn export_cursor_requires_a_snapshot_for_nonempty_batches() {
    let mut cursor = ExportCursor::<String>::new(2).unwrap();
    assert!(cursor.advance("row".into(), None, true).is_err());
}
