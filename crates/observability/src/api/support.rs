use audit_contract::{AuditOutboxRecord, OperationAuditContext};
use axum::extract::Extension;

use crate::application::{ObservabilityError, ObservabilityResult};

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

pub(super) fn successful_operation_audit(context: Option<Extension<OperationAuditContext>>) -> ObservabilityResult<SuccessfulOperationAudit> {
    let Extension(context) = context.ok_or_else(|| ObservabilityError::Infrastructure("operation audit context is missing".into()))?;
    let record = context
        .success_record()
        .map_err(|error| ObservabilityError::Infrastructure(format!("operation audit record creation failed: {error}")))?
        .ok_or_else(|| ObservabilityError::Infrastructure("operation audit actor is missing".into()))?;
    Ok(SuccessfulOperationAudit { context, record })
}
