use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use constants::system::STATUS_NORMAL;
use kernel::error::LocalizedError;

use crate::application::{SystemError, SystemResult};

use super::application::{EMPTY_IDS_KEY, clean_ids, validate_input};
use super::{AuditedNoticeRepository, Notice, NoticeAuditedUseCase, NoticeInput, NoticeService, ReplaceNoticeCommand};

#[async_trait]
impl<R: AuditedNoticeRepository> NoticeAuditedUseCase for NoticeService<R> {
    async fn create_notice_with_audit(&self, input: NoticeInput, operator: String, audit: AuditOutboxRecord) -> SystemResult<Notice> {
        let input = validate_input(input)?;
        self.repository.create_notice_with_audit(input, &operator, &audit).await
    }

    async fn replace_notice_with_audit(&self, command: ReplaceNoticeCommand, audit: AuditOutboxRecord) -> SystemResult<Notice> {
        let command = ReplaceNoticeCommand {
            input: validate_input(command.input)?,
            ..command
        };
        self.repository.replace_notice_with_audit(&command, &audit).await
    }

    async fn delete_notice_with_audit(&self, id: &str, audit: AuditOutboxRecord) -> SystemResult<()> {
        self.repository.delete_notice_with_audit(id, &audit).await
    }

    async fn delete_notices_with_audit(&self, ids: Vec<String>, audit: AuditOutboxRecord) -> SystemResult<()> {
        let ids = clean_ids(ids);
        if ids.is_empty() {
            return Err(SystemError::InvalidInput(LocalizedError::new(EMPTY_IDS_KEY)));
        }
        self.repository.delete_notices_with_audit(&ids, &audit).await
    }

    async fn mark_read_with_audit(&self, notice_id: &str, user_id: &str, audit: AuditOutboxRecord) -> SystemResult<()> {
        let notice = self.repository.find_notice(notice_id).await?.ok_or(SystemError::NotFound)?;
        if notice.status != STATUS_NORMAL {
            return Err(SystemError::NotFound);
        }
        self.repository.mark_read_with_audit(notice_id, user_id, &audit).await
    }

    async fn mark_all_read_with_audit(&self, user_id: &str, audit: AuditOutboxRecord) -> SystemResult<()> {
        self.repository.mark_all_read_with_audit(user_id, &audit).await
    }
}
