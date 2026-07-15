use audit_contract::{AuditOutboxRecord, AuditOutboxRecorder, OperationAuditContext};
use axum::Extension;

use crate::{api::SystemApiError, application::SystemError};

const MISSING_OPERATION_AUDIT_ACTOR: &str = "operation audit actor is missing";
const MISSING_OPERATION_AUDIT_CONTEXT: &str = "operation audit context is missing";

pub(crate) struct SuccessfulOperationAudit {
    context: OperationAuditContext,
    record: AuditOutboxRecord,
}

impl SuccessfulOperationAudit {
    pub(crate) fn record(&self) -> AuditOutboxRecord {
        self.record.clone()
    }

    pub(crate) fn mark_persisted(&self) {
        self.context.mark_persisted();
    }
}

pub(crate) fn successful_operation_audit(context: Option<Extension<OperationAuditContext>>) -> Result<SuccessfulOperationAudit, SystemApiError> {
    let Extension(context) = context.ok_or_else(|| SystemApiError(SystemError::Infrastructure(MISSING_OPERATION_AUDIT_CONTEXT.into())))?;
    let record = context
        .success_record()
        .map_err(|error| SystemApiError(SystemError::Infrastructure(error.to_string())))?
        .ok_or_else(|| SystemApiError(SystemError::Infrastructure(MISSING_OPERATION_AUDIT_ACTOR.into())))?;
    Ok(SuccessfulOperationAudit { context, record })
}

pub(crate) async fn record_successful_operation(
    recorder: &dyn AuditOutboxRecorder,
    context: Option<Extension<OperationAuditContext>>,
) -> Result<(), SystemApiError> {
    let audit = successful_operation_audit(context)?;
    recorder
        .record(audit.record())
        .await
        .map_err(|error| SystemApiError(SystemError::Infrastructure(format!("operation audit outbox recording failed: {error}"))))?;
    audit.mark_persisted();
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use audit_contract::{ActorSnapshot, AuditOutboxEvent, AuditOutboxResult, BusinessType, OperationAuditSeed, OperationRequestSnapshot, OperatorType};
    use time::OffsetDateTime;

    use super::{AuditOutboxRecord, AuditOutboxRecorder, Extension, OperationAuditContext, record_successful_operation};

    #[derive(Default)]
    struct MemoryRecorder(Arc<Mutex<Vec<AuditOutboxRecord>>>);

    #[async_trait]
    impl AuditOutboxRecorder for MemoryRecorder {
        async fn record(&self, record: AuditOutboxRecord) -> AuditOutboxResult<()> {
            self.0.lock().unwrap().push(record);
            Ok(())
        }
    }

    struct EmptyRequestSnapshot;

    impl OperationRequestSnapshot for EmptyRequestSnapshot {
        fn request_params(&self) -> String {
            String::new()
        }
    }

    #[tokio::test]
    async fn records_a_completed_cross_resource_operation_before_marking_context_persisted() {
        let context = audit_context();
        let recorder = MemoryRecorder::default();

        record_successful_operation(&recorder, Some(Extension(context.clone()))).await.unwrap();

        let records = recorder.0.lock().unwrap().clone();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].id, "event-1");
        assert!(matches!(records[0].event, AuditOutboxEvent::Operation(_)));
        assert!(context.is_persisted());
    }

    fn audit_context() -> OperationAuditContext {
        let context = OperationAuditContext::new(
            OperationAuditSeed {
                id: "event-1".into(),
                occurred_at: OffsetDateTime::UNIX_EPOCH,
                title_key: "audit.module.config".into(),
                business_type: BusinessType::Export,
                handler: "system::export_configs".into(),
                request_method: "POST".into(),
                operator_type: OperatorType::Manage,
                operation_url: "/api/system/configs/export".into(),
                operation_ip: "198.51.100.10".into(),
                request_id: "request-1".into(),
            },
            Arc::new(EmptyRequestSnapshot),
        );
        context
            .set_actor(ActorSnapshot {
                user_id: Some("user-1".into()),
                username: "alice".into(),
                department_id: None,
                department_name: String::new(),
            })
            .unwrap();
        context
    }
}
