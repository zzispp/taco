use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use kernel::pagination::CursorPage;
use storage::{Database, StorageError};

use crate::application::{SystemError, SystemResult};

use super::{
    application::NoticeRepository,
    audited::AuditedNoticeRepository,
    domain::{Notice, NoticeInput, NoticeListFilter, NoticeReader, NoticeReaderFilter, NoticeSummary, NoticeTopResponse, ReplaceNoticeCommand},
};

mod mapping;
mod queries;
mod records;

use queries::NoticeQueries;

#[derive(Clone)]
pub struct StorageNoticeRepository {
    queries: NoticeQueries,
}

impl StorageNoticeRepository {
    pub fn new(database: Database) -> Self {
        Self {
            queries: NoticeQueries::new(database),
        }
    }
}

#[async_trait]
impl NoticeRepository for StorageNoticeRepository {
    async fn page_notices(&self, filter: NoticeListFilter) -> SystemResult<CursorPage<NoticeSummary>> {
        self.queries.page_notices(filter).await
    }

    async fn find_notice(&self, id: &str) -> SystemResult<Option<Notice>> {
        self.queries.find_notice(id).await.map_err(map_storage_error)
    }

    async fn create_notice(&self, input: NoticeInput, operator: &str) -> SystemResult<Notice> {
        self.queries.create_notice(input, operator).await.map_err(map_storage_error)
    }

    async fn replace_notice(&self, id: &str, input: NoticeInput, operator: &str) -> SystemResult<Notice> {
        self.queries.replace_notice(id, input, operator).await.map_err(map_storage_error)
    }

    async fn delete_notice(&self, id: &str) -> SystemResult<()> {
        self.queries.delete_notice(id).await.map_err(map_storage_error)
    }

    async fn delete_notices(&self, ids: &[String]) -> SystemResult<()> {
        self.queries.delete_notices(ids).await.map_err(map_storage_error)
    }

    async fn top_notices(&self, user_id: &str, limit: u64) -> SystemResult<NoticeTopResponse> {
        self.queries.top_notices(user_id, limit).await.map_err(map_storage_error)
    }

    async fn mark_read(&self, notice_id: &str, user_id: &str) -> SystemResult<()> {
        self.queries.mark_read(notice_id, user_id).await.map_err(map_storage_error)
    }

    async fn mark_all_read(&self, user_id: &str) -> SystemResult<()> {
        self.queries.mark_all_read(user_id).await.map_err(map_storage_error)
    }

    async fn page_readers(&self, notice_id: &str, filter: NoticeReaderFilter) -> SystemResult<CursorPage<NoticeReader>> {
        self.queries.page_readers(notice_id, filter).await
    }
}

#[async_trait]
impl AuditedNoticeRepository for StorageNoticeRepository {
    async fn create_notice_with_audit(&self, input: NoticeInput, operator: &str, audit: &AuditOutboxRecord) -> SystemResult<Notice> {
        self.queries.create_notice_with_audit(input, operator, audit).await.map_err(map_storage_error)
    }

    async fn replace_notice_with_audit(&self, command: &ReplaceNoticeCommand, audit: &AuditOutboxRecord) -> SystemResult<Notice> {
        self.queries.replace_notice_with_audit(command, audit).await.map_err(map_storage_error)
    }

    async fn delete_notice_with_audit(&self, id: &str, audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.queries.delete_notice_with_audit(id, audit).await.map_err(map_storage_error)
    }

    async fn delete_notices_with_audit(&self, ids: &[String], audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.queries.delete_notices_with_audit(ids, audit).await.map_err(map_storage_error)
    }

    async fn mark_read_with_audit(&self, notice_id: &str, user_id: &str, audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.queries.mark_read_with_audit(notice_id, user_id, audit).await.map_err(map_storage_error)
    }

    async fn mark_all_read_with_audit(&self, user_id: &str, audit: &AuditOutboxRecord) -> SystemResult<()> {
        self.queries.mark_all_read_with_audit(user_id, audit).await.map_err(map_storage_error)
    }
}

fn map_storage_error(error: StorageError) -> SystemError {
    match error {
        StorageError::NotFound => SystemError::NotFound,
        StorageError::Conflict(_) => SystemError::Conflict(kernel::error::LocalizedError::new("errors.common.conflict")),
        StorageError::UniqueViolation { message, .. } => SystemError::Infrastructure(message),
        StorageError::Database(message) => SystemError::Infrastructure(message),
    }
}
