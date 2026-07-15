pub(super) mod login;
pub(super) mod operation;

use audit_contract::{AuditOutboxRecord, OperationAuditContext};
use axum::extract::Extension;

use crate::application::{AuditError, AuditResult};

pub(super) use login::{clear_login_logs, delete_login_log, delete_login_logs, export_login_logs, list_login_logs, unlock_login};
pub(super) use operation::{clear_operation_logs, delete_operation_log, delete_operation_logs, export_operation_logs, get_operation_log, list_operation_logs};

use super::AuditApiError;

type ApiResult<T> = Result<T, AuditApiError>;

const MISSING_OPERATION_AUDIT_CONTEXT: &str = "operation audit context is missing";

pub(super) fn required_operation_audit_context(context: Option<Extension<OperationAuditContext>>) -> AuditResult<OperationAuditContext> {
    context
        .map(|context| context.0)
        .ok_or_else(|| AuditError::Infrastructure(MISSING_OPERATION_AUDIT_CONTEXT.into()))
}

pub(super) fn successful_operation_record(context: &OperationAuditContext) -> AuditResult<AuditOutboxRecord> {
    context
        .success_record()
        .map_err(|error| AuditError::Infrastructure(error.to_string()))?
        .ok_or_else(|| AuditError::Infrastructure("authenticated operation audit actor is missing".into()))
}

#[cfg(test)]
mod tests {
    use crate::application::AuditError;

    use super::{MISSING_OPERATION_AUDIT_CONTEXT, required_operation_audit_context};

    #[test]
    fn missing_operation_audit_context_is_an_explicit_infrastructure_error() {
        let result = required_operation_audit_context(None);

        assert!(matches!(result, Err(AuditError::Infrastructure(message)) if message == MISSING_OPERATION_AUDIT_CONTEXT));
    }
}
