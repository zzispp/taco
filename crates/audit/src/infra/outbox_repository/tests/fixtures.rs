use audit_contract::{
    ActorSnapshot, AuditOutboxEvent, AuditOutboxRecord, AuditStatus, BusinessType, LoginEventType, OperationAuditEvent, OperatorType, SecurityAuditEvent,
};
use time::OffsetDateTime;

pub(super) fn operation_record(id: &str, business_type: BusinessType) -> AuditOutboxRecord {
    AuditOutboxRecord {
        id: id.into(),
        occurred_at: OffsetDateTime::now_utc(),
        event: AuditOutboxEvent::Operation(OperationAuditEvent {
            title_key: "audit.module.operation_log".into(),
            business_type,
            handler: "audit::test".into(),
            request_method: "DELETE".into(),
            operator_type: OperatorType::Manage,
            actor: ActorSnapshot {
                user_id: Some("admin".into()),
                username: "admin".into(),
                department_id: None,
                department_name: String::new(),
            },
            operation_url: "/api/system/operation-logs".into(),
            operation_ip: "198.51.100.10".into(),
            status: AuditStatus::Success,
            request_id: "request-outbox-test".into(),
            request_params: "{}".into(),
            response_result: String::new(),
            error_message: String::new(),
            cost_time_ms: 0,
        }),
    }
}

pub(super) fn security_record(id: &str, event_type: LoginEventType) -> AuditOutboxRecord {
    AuditOutboxRecord {
        id: id.into(),
        occurred_at: OffsetDateTime::now_utc(),
        event: AuditOutboxEvent::Security(security_event(event_type)),
    }
}

pub(super) fn security_event(event_type: LoginEventType) -> SecurityAuditEvent {
    SecurityAuditEvent {
        request_id: "request-security-test".into(),
        route: "/api/auth/sign-in".into(),
        user_id: None,
        username: "alice".into(),
        ip_address: "198.51.100.10".into(),
        browser: "browser".into(),
        os: "os".into(),
        status: AuditStatus::Failure,
        event_type,
        message_key: "messages.user.login_failure".into(),
        message_params: Default::default(),
    }
}
