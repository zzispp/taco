use audit_contract::{AuditOutboxRecord, OperationAuditContext};
use axum::extract::Extension;

use crate::api::SchedulerApiError;
use crate::application::SchedulerError;

const MISSING_OPERATION_AUDIT_ACTOR: &str = "authenticated operation audit actor is missing";
const MISSING_OPERATION_AUDIT_CONTEXT: &str = "operation audit context is missing";

pub(super) struct SuccessfulOperationAudit {
    context: OperationAuditContext,
    record: AuditOutboxRecord,
}

impl SuccessfulOperationAudit {
    pub(super) fn record(&self) -> AuditOutboxRecord {
        self.record.clone()
    }

    pub(super) fn mark_persisted(&self) {
        self.context.mark_persisted();
    }
}

pub(super) fn successful_operation_audit(context: Option<Extension<OperationAuditContext>>) -> Result<SuccessfulOperationAudit, SchedulerApiError> {
    let Extension(context) = context.ok_or_else(|| SchedulerApiError(SchedulerError::Infrastructure(MISSING_OPERATION_AUDIT_CONTEXT.into())))?;
    let record = context
        .success_record()
        .map_err(|error| SchedulerApiError(SchedulerError::Infrastructure(error.to_string())))?
        .ok_or_else(|| SchedulerApiError(SchedulerError::Infrastructure(MISSING_OPERATION_AUDIT_ACTOR.into())))?;
    Ok(SuccessfulOperationAudit { context, record })
}
