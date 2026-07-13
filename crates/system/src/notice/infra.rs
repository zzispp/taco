use async_trait::async_trait;
use kernel::pagination::Page;
use storage::{Database, StorageError};

use crate::application::{SystemError, SystemResult};

use super::{
    application::NoticeRepository,
    domain::{Notice, NoticeInput, NoticeListFilter, NoticeReader, NoticeReaderFilter, NoticeSummary, NoticeTopResponse},
};

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
    async fn page_notices(&self, filter: NoticeListFilter) -> SystemResult<Page<NoticeSummary>> {
        self.queries.page_notices(filter).await.map_err(map_storage_error)
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

    async fn page_readers(&self, notice_id: &str, filter: NoticeReaderFilter) -> SystemResult<Page<NoticeReader>> {
        self.queries.page_readers(notice_id, filter).await.map_err(map_storage_error)
    }
}

fn map_storage_error(error: StorageError) -> SystemError {
    match error {
        StorageError::NotFound => SystemError::NotFound,
        StorageError::Conflict(_) => SystemError::Conflict(kernel::error::LocalizedError::new("errors.common.conflict")),
        StorageError::Database(message) => SystemError::Infrastructure(message),
    }
}
