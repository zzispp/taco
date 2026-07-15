#[path = "codes.rs"]
mod codes;

use std::collections::BTreeMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::AuditOutboxResult;

pub use codes::{AuditStatus, AuditStream, BusinessType, LoginEventType, OperatorType};

pub const AUDIT_OUTBOX_PAYLOAD_VERSION: i16 = 1;
pub const OPERATION_AUDIT_EVENT_TYPE: &str = "operation";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorSnapshot {
    pub user_id: Option<String>,
    pub username: String,
    pub department_id: Option<String>,
    pub department_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationAuditEvent {
    pub title_key: String,
    pub business_type: BusinessType,
    pub handler: String,
    pub request_method: String,
    pub operator_type: OperatorType,
    pub actor: ActorSnapshot,
    pub operation_url: String,
    pub operation_ip: String,
    pub status: AuditStatus,
    pub request_id: String,
    pub request_params: String,
    pub response_result: String,
    pub error_message: String,
    pub cost_time_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityAuditEvent {
    pub request_id: String,
    pub route: String,
    pub user_id: Option<String>,
    pub username: String,
    pub ip_address: String,
    pub browser: String,
    pub os: String,
    pub status: AuditStatus,
    pub event_type: LoginEventType,
    pub message_key: String,
    pub message_params: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "event", rename_all = "snake_case")]
pub enum AuditOutboxEvent {
    Operation(OperationAuditEvent),
    Security(SecurityAuditEvent),
}

impl AuditOutboxEvent {
    pub const fn stream(&self) -> AuditStream {
        match self {
            Self::Operation(_) => AuditStream::Operation,
            Self::Security(_) => AuditStream::Security,
        }
    }

    pub const fn event_type(&self) -> &'static str {
        match self {
            Self::Operation(_) => OPERATION_AUDIT_EVENT_TYPE,
            Self::Security(event) => event.event_type.code(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuditOutboxRecord {
    pub id: String,
    pub occurred_at: OffsetDateTime,
    pub event: AuditOutboxEvent,
}

impl AuditOutboxRecord {
    pub const fn stream(&self) -> AuditStream {
        self.event.stream()
    }

    pub const fn event_type(&self) -> &'static str {
        self.event.event_type()
    }

    pub fn payload(&self) -> AuditOutboxResult<serde_json::Value> {
        serde_json::to_value(&self.event).map_err(|error| crate::AuditOutboxError::InvalidPayload(error.to_string()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationAuditDraft {
    pub id: String,
    pub occurred_at: OffsetDateTime,
    pub title_key: String,
    pub business_type: BusinessType,
    pub handler: String,
    pub request_method: String,
    pub operator_type: OperatorType,
    pub actor: ActorSnapshot,
    pub operation_url: String,
    pub operation_ip: String,
    pub request_id: String,
    pub request_params: String,
}

impl OperationAuditDraft {
    pub fn finish(self, outcome: OperationOutcome) -> AuditOutboxRecord {
        let event = OperationAuditEvent {
            title_key: self.title_key,
            business_type: self.business_type,
            handler: self.handler,
            request_method: self.request_method,
            operator_type: self.operator_type,
            actor: self.actor,
            operation_url: self.operation_url,
            operation_ip: self.operation_ip,
            status: outcome.status,
            request_id: self.request_id,
            request_params: self.request_params,
            response_result: outcome.response_result,
            error_message: outcome.error_message,
            cost_time_ms: outcome.cost_time_ms,
        };
        AuditOutboxRecord {
            id: self.id,
            occurred_at: self.occurred_at,
            event: AuditOutboxEvent::Operation(event),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationOutcome {
    pub status: AuditStatus,
    pub response_result: String,
    pub error_message: String,
    pub cost_time_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Audited<T> {
    pub command: T,
    pub audit: OperationAuditDraft,
}

#[async_trait]
pub trait AuditOutboxRecorder: Send + Sync + 'static {
    async fn record(&self, record: AuditOutboxRecord) -> AuditOutboxResult<()>;
}

#[async_trait]
pub trait SecurityAuditRecorder: Send + Sync + 'static {
    async fn record(&self, record: AuditOutboxRecord) -> AuditOutboxResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_draft_preserves_immutable_snapshot() {
        let draft = OperationAuditDraft {
            id: "event-1".into(),
            occurred_at: OffsetDateTime::UNIX_EPOCH,
            title_key: "audit.module.user".into(),
            business_type: BusinessType::Update,
            handler: "user::replace".into(),
            request_method: "PUT".into(),
            operator_type: OperatorType::Manage,
            actor: ActorSnapshot {
                user_id: Some("user-1".into()),
                username: "alice".into(),
                department_id: Some("dept-1".into()),
                department_name: "Engineering".into(),
            },
            operation_url: "/api/system/users/user-2".into(),
            operation_ip: "198.51.100.10".into(),
            request_id: "request-1".into(),
            request_params: "{\"password\":\"***\"}".into(),
        };

        let event = draft.finish(OperationOutcome {
            status: AuditStatus::Success,
            response_result: String::new(),
            error_message: String::new(),
            cost_time_ms: 12,
        });

        assert_eq!(event.stream(), AuditStream::Operation);
        assert_eq!(event.event_type(), OPERATION_AUDIT_EVENT_TYPE);
        assert_eq!(event.id, "event-1");
        assert_eq!(event.payload().unwrap()["event"]["actor"]["department_name"], "Engineering");
    }
}
