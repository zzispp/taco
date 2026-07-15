use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use audit_contract::{
    AuditOutboxError, AuditOutboxEvent, AuditOutboxRecord, AuditOutboxRecorder, AuditOutboxResult, OperationAuditEvent, SecurityAuditEvent,
    SecurityAuditRecorder,
};

use crate::test_support::MemoryOnlineSessionStore;

#[derive(Default)]
pub(crate) struct MemoryOperationAuditRecorder {
    events: Mutex<Vec<OperationAuditEvent>>,
    record_failure: Mutex<Option<String>>,
}

impl MemoryOperationAuditRecorder {
    pub(crate) fn events(&self) -> Vec<OperationAuditEvent> {
        self.events.lock().unwrap().clone()
    }

    pub(crate) fn fail_with(&self, message: &str) {
        *self.record_failure.lock().unwrap() = Some(message.into());
    }
}

#[async_trait]
impl AuditOutboxRecorder for MemoryOperationAuditRecorder {
    async fn record(&self, record: AuditOutboxRecord) -> AuditOutboxResult<()> {
        if let Some(message) = self.record_failure.lock().unwrap().clone() {
            return Err(AuditOutboxError::Infrastructure(message));
        }
        let AuditOutboxEvent::Operation(event) = record.event else {
            return Err(AuditOutboxError::InvalidPayload("expected operation audit event".into()));
        };
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}

pub(crate) struct MemorySecurityAuditRecorder {
    events: Mutex<Vec<SecurityAuditEvent>>,
    session_counts: Mutex<Vec<usize>>,
    record_failure: Mutex<Option<String>>,
    sessions: Arc<MemoryOnlineSessionStore>,
}

impl MemorySecurityAuditRecorder {
    pub(crate) fn new(sessions: Arc<MemoryOnlineSessionStore>) -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            session_counts: Mutex::new(Vec::new()),
            record_failure: Mutex::new(None),
            sessions,
        }
    }

    pub(crate) fn events(&self) -> Vec<SecurityAuditEvent> {
        self.events.lock().unwrap().clone()
    }

    pub(crate) fn session_counts(&self) -> Vec<usize> {
        self.session_counts.lock().unwrap().clone()
    }

    pub(crate) fn fail_with(&self, message: &str) {
        *self.record_failure.lock().unwrap() = Some(message.into());
    }
}

#[async_trait]
impl SecurityAuditRecorder for MemorySecurityAuditRecorder {
    async fn record(&self, record: AuditOutboxRecord) -> AuditOutboxResult<()> {
        if let Some(message) = self.record_failure.lock().unwrap().clone() {
            return Err(AuditOutboxError::Infrastructure(message));
        }
        let AuditOutboxEvent::Security(event) = record.event else {
            return Err(AuditOutboxError::InvalidPayload("expected security audit event".into()));
        };
        self.session_counts.lock().unwrap().push(self.sessions.sessions().len());
        self.events.lock().unwrap().push(event);
        Ok(())
    }
}
