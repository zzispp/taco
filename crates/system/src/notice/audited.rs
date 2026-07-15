use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;

use crate::application::SystemResult;

use super::{Notice, NoticeInput, NoticeRepository, ReplaceNoticeCommand};

/// Persists a notice state change and its operation audit record in the same
/// PostgreSQL transaction.
#[async_trait]
pub trait AuditedNoticeRepository: NoticeRepository {
    async fn create_notice_with_audit(&self, input: NoticeInput, operator: &str, audit: &AuditOutboxRecord) -> SystemResult<Notice>;
    async fn replace_notice_with_audit(&self, command: &ReplaceNoticeCommand, audit: &AuditOutboxRecord) -> SystemResult<Notice>;
    async fn delete_notice_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> SystemResult<()>;
    async fn delete_notices_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> SystemResult<()>;
    async fn mark_read_with_audit(&self, notice_id: &str, user_id: &str, audit: &AuditOutboxRecord) -> SystemResult<()>;
    async fn mark_all_read_with_audit(&self, user_id: &str, audit: &AuditOutboxRecord) -> SystemResult<()>;
}

/// Notice-management writes that require a transactional operation audit
/// record. Ordinary notice reads intentionally do not depend on this port.
#[async_trait]
pub trait NoticeAuditedUseCase: Send + Sync + 'static {
    async fn create_notice_with_audit(&self, input: NoticeInput, operator: String, audit: AuditOutboxRecord) -> SystemResult<Notice>;
    async fn replace_notice_with_audit(&self, command: ReplaceNoticeCommand, audit: AuditOutboxRecord) -> SystemResult<Notice>;
    async fn delete_notice_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()>;
    async fn delete_notices_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SystemResult<()>;
    async fn mark_read_with_audit(&self, notice_id: &str, user_id: &str, audit: AuditOutboxRecord) -> SystemResult<()>;
    async fn mark_all_read_with_audit(&self, user_id: &str, audit: AuditOutboxRecord) -> SystemResult<()>;
}
