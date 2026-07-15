use std::sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
};
use std::time::Instant;

use time::OffsetDateTime;

use crate::{ActorSnapshot, AuditOutboxError, AuditOutboxRecord, AuditOutboxResult, BusinessType, OperationAuditDraft, OperationOutcome, OperatorType};

/// Supplies a sanitized request snapshot without exposing HTTP framework types
/// to application-layer command handlers.
pub trait OperationRequestSnapshot: Send + Sync + 'static {
    fn request_params(&self) -> String;
}

/// Immutable request metadata created before authentication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationAuditSeed {
    pub id: String,
    pub occurred_at: OffsetDateTime,
    pub title_key: String,
    pub business_type: BusinessType,
    pub handler: String,
    pub request_method: String,
    pub operator_type: OperatorType,
    pub operation_url: String,
    pub operation_ip: String,
    pub request_id: String,
}

/// A request-scoped bridge between HTTP audit capture and a bounded context's
/// transaction-owning repository.
///
/// Authentication sets the actor exactly once before authorization. A handler
/// can build the success record and append it in its business transaction, then
/// call [`Self::mark_persisted`]. The outer HTTP middleware writes only records
/// that were not already included in a successful business transaction.
#[derive(Clone)]
pub struct OperationAuditContext(Arc<OperationAuditContextInner>);

struct OperationAuditContextInner {
    seed: OperationAuditSeed,
    request_snapshot: Arc<dyn OperationRequestSnapshot>,
    actor: OnceLock<ActorSnapshot>,
    persisted: AtomicBool,
    started: Instant,
}

impl OperationAuditContext {
    pub fn new(seed: OperationAuditSeed, request_snapshot: Arc<dyn OperationRequestSnapshot>) -> Self {
        Self(Arc::new(OperationAuditContextInner {
            seed,
            request_snapshot,
            actor: OnceLock::new(),
            persisted: AtomicBool::new(false),
            started: Instant::now(),
        }))
    }

    pub fn set_actor(&self, actor: ActorSnapshot) -> Result<(), &'static str> {
        self.0.actor.set(actor).map_err(|_| "audit actor context was already set")
    }

    pub fn success_record(&self) -> AuditOutboxResult<Option<AuditOutboxRecord>> {
        let cost_time_ms = elapsed_ms(self.0.started)?;
        Ok(self.record(OperationOutcome {
            status: crate::AuditStatus::Success,
            response_result: String::new(),
            error_message: String::new(),
            cost_time_ms,
        }))
    }

    pub fn record(&self, outcome: OperationOutcome) -> Option<AuditOutboxRecord> {
        let actor = self.0.actor.get()?.clone();
        Some(
            OperationAuditDraft {
                id: self.0.seed.id.clone(),
                occurred_at: self.0.seed.occurred_at,
                title_key: self.0.seed.title_key.clone(),
                business_type: self.0.seed.business_type,
                handler: self.0.seed.handler.clone(),
                request_method: self.0.seed.request_method.clone(),
                operator_type: self.0.seed.operator_type,
                actor,
                operation_url: self.0.seed.operation_url.clone(),
                operation_ip: self.0.seed.operation_ip.clone(),
                request_id: self.0.seed.request_id.clone(),
                request_params: self.0.request_snapshot.request_params(),
            }
            .finish(outcome),
        )
    }

    pub fn mark_persisted(&self) {
        self.0.persisted.store(true, Ordering::Release);
    }

    pub fn is_persisted(&self) -> bool {
        self.0.persisted.load(Ordering::Acquire)
    }
}

fn elapsed_ms(started: Instant) -> AuditOutboxResult<i64> {
    i64::try_from(started.elapsed().as_millis())
        .map_err(|error| AuditOutboxError::Infrastructure(format!("operation audit duration conversion failed: {error}")))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use time::OffsetDateTime;

    use crate::{ActorSnapshot, AuditOutboxEvent, BusinessType, OperatorType};

    use super::{OperationAuditContext, OperationAuditSeed, OperationRequestSnapshot};

    struct StaticSnapshot;

    impl OperationRequestSnapshot for StaticSnapshot {
        fn request_params(&self) -> String {
            "{\"password\":\"***\"}".into()
        }
    }

    #[test]
    fn context_keeps_actor_and_request_snapshot_immutable_for_a_transactional_record() {
        let context = OperationAuditContext::new(seed(), Arc::new(StaticSnapshot));
        context
            .set_actor(ActorSnapshot {
                user_id: Some("user-1".into()),
                username: "alice".into(),
                department_id: Some("dept-1".into()),
                department_name: "Engineering".into(),
            })
            .unwrap();

        let record = context.success_record().unwrap().unwrap();
        let AuditOutboxEvent::Operation(event) = record.event else {
            panic!("expected an operation event");
        };
        assert_eq!(event.actor.username, "alice");
        assert_eq!(event.request_params, r#"{"password":"***"}"#);
        assert!(context.set_actor(ActorSnapshot::default()).is_err());
        assert!(!context.is_persisted());
        context.mark_persisted();
        assert!(context.is_persisted());
    }

    fn seed() -> OperationAuditSeed {
        OperationAuditSeed {
            id: "event-1".into(),
            occurred_at: OffsetDateTime::UNIX_EPOCH,
            title_key: "audit.module.user".into(),
            business_type: BusinessType::Update,
            handler: "user::replace".into(),
            request_method: "PUT".into(),
            operator_type: OperatorType::Manage,
            operation_url: "/api/system/users/user-1".into(),
            operation_ip: "198.51.100.10".into(),
            request_id: "request-1".into(),
        }
    }
}
