use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use constants::system::STATUS_NORMAL;
use kernel::pagination::CursorPage;

use crate::{
    application::SystemResult,
    notice::{
        AuditedNoticeRepository, Notice, NoticeInput, NoticeListFilter, NoticeReader, NoticeReaderFilter, NoticeRepository, NoticeSummary, NoticeTopResponse,
        ReplaceNoticeCommand,
    },
};

#[derive(Clone)]
pub(super) struct TestRepository {
    notice: Notice,
    operators: Arc<Mutex<Vec<String>>>,
    audits: Arc<Mutex<Vec<AuditOutboxRecord>>>,
}

impl TestRepository {
    pub(super) fn with_status(status: &str) -> Self {
        Self {
            notice: notice(status),
            operators: Arc::new(Mutex::new(Vec::new())),
            audits: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(super) fn operators(&self) -> Vec<String> {
        self.operators.lock().expect("operators lock").clone()
    }

    pub(super) fn audits(&self) -> Vec<AuditOutboxRecord> {
        self.audits.lock().expect("audits lock").clone()
    }

    fn record_audit(&self, audit: &AuditOutboxRecord) {
        self.audits.lock().expect("audits lock").push(audit.clone());
    }
}

#[async_trait]
impl NoticeRepository for TestRepository {
    async fn page_notices(&self, _filter: NoticeListFilter) -> SystemResult<CursorPage<NoticeSummary>> {
        Ok(CursorPage::new(vec![summary(&self.notice)], None, None))
    }

    async fn find_notice(&self, _id: &str) -> SystemResult<Option<Notice>> {
        Ok(Some(self.notice.clone()))
    }

    async fn create_notice(&self, input: NoticeInput, operator: &str) -> SystemResult<Notice> {
        self.operators.lock().expect("operators lock").push(operator.into());
        Ok(notice_from_input(input, operator))
    }

    async fn replace_notice(&self, _id: &str, input: NoticeInput, operator: &str) -> SystemResult<Notice> {
        self.operators.lock().expect("operators lock").push(operator.into());
        Ok(notice_from_input(input, operator))
    }

    async fn delete_notice(&self, _id: &str) -> SystemResult<()> {
        Ok(())
    }

    async fn delete_notices(&self, _ids: &[String]) -> SystemResult<()> {
        Ok(())
    }

    async fn top_notices(&self, _user_id: &str, _limit: u64) -> SystemResult<NoticeTopResponse> {
        Ok(NoticeTopResponse {
            items: Vec::new(),
            unread_count: 0,
        })
    }

    async fn mark_read(&self, _notice_id: &str, _user_id: &str) -> SystemResult<()> {
        Ok(())
    }

    async fn mark_all_read(&self, _user_id: &str) -> SystemResult<()> {
        Ok(())
    }

    async fn page_readers(&self, _notice_id: &str, _filter: NoticeReaderFilter) -> SystemResult<CursorPage<NoticeReader>> {
        Ok(CursorPage::new(Vec::new(), None, None))
    }
}

#[async_trait]
impl AuditedNoticeRepository for TestRepository {
    async fn create_notice_with_audit(&self, input: NoticeInput, operator: &str, audit: &AuditOutboxRecord) -> SystemResult<Notice> {
        let notice = self.create_notice(input, operator).await?;
        self.record_audit(audit);
        Ok(notice)
    }

    async fn replace_notice_with_audit(&self, command: &ReplaceNoticeCommand, audit: &AuditOutboxRecord) -> SystemResult<Notice> {
        let notice = self.replace_notice(&command.id, command.input.clone(), &command.operator).await?;
        self.record_audit(audit);
        Ok(notice)
    }

    async fn delete_notice_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.delete_notice(id).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn delete_notices_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.delete_notices(ids).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn mark_read_with_audit(&self, notice_id: &str, user_id: &str, audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.mark_read(notice_id, user_id).await?;
        self.record_audit(audit);
        Ok(())
    }

    async fn mark_all_read_with_audit(&self, user_id: &str, audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.mark_all_read(user_id).await?;
        self.record_audit(audit);
        Ok(())
    }
}

fn notice(status: &str) -> Notice {
    Notice {
        notice_id: "notice-1".into(),
        notice_title: "Notice".into(),
        notice_type: "1".into(),
        notice_content: "# Content".into(),
        status: status.into(),
        create_by: "admin".into(),
        create_time: "2026-07-13T00:00:00Z".into(),
        update_by: None,
        update_time: None,
        remark: None,
    }
}

fn notice_from_input(input: NoticeInput, operator: &str) -> Notice {
    Notice {
        notice_title: input.notice_title,
        notice_type: input.notice_type,
        notice_content: input.notice_content,
        status: input.status,
        create_by: operator.into(),
        remark: input.remark,
        ..notice(STATUS_NORMAL)
    }
}

fn summary(notice: &Notice) -> NoticeSummary {
    NoticeSummary {
        notice_id: notice.notice_id.clone(),
        notice_title: notice.notice_title.clone(),
        notice_type: notice.notice_type.clone(),
        status: notice.status.clone(),
        create_by: notice.create_by.clone(),
        create_time: notice.create_time.clone(),
    }
}
