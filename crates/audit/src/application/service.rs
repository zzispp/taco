use std::sync::Arc;

use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use kernel::pagination::{CursorPage, CursorPageRequest};

use crate::domain::{LoginLog, LoginLogFilter, OperationLogDetail, OperationLogFilter, OperationLogSummary};

use super::{
    AuditError, AuditExportSession, AuditRepository, AuditResult, AuditUseCase, login_cursor_page, login_cursor_query, operation_cursor_page,
    operation_cursor_query, validation,
};

#[derive(Clone)]
pub struct AuditService {
    repository: Arc<dyn AuditRepository>,
}

impl AuditService {
    pub fn new(repository: Arc<dyn AuditRepository>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl AuditUseCase for AuditService {
    async fn page_operations(&self, filter: OperationLogFilter, page: CursorPageRequest) -> AuditResult<CursorPage<OperationLogSummary>> {
        let query = operation_cursor_query(&filter, &page)?;
        let slice = self.repository.page_operations(filter.clone(), query.clone()).await?;
        operation_cursor_page(&filter, &query, slice)
    }

    async fn begin_export(&self) -> AuditResult<Box<dyn AuditExportSession>> {
        self.repository.begin_export().await
    }

    async fn operation_detail(&self, id: &str) -> AuditResult<OperationLogDetail> {
        self.repository.find_operation(id).await?.ok_or(AuditError::NotFound)
    }

    async fn delete_operation(&self, id: String) -> AuditResult<()> {
        self.repository.delete_operations(&validation::validate_ids(vec![id])?).await
    }

    async fn delete_operations(&self, ids: Vec<String>) -> AuditResult<()> {
        self.repository.delete_operations(&validation::validate_ids(ids)?).await
    }

    async fn delete_operations_with_audit(&self, ids: Vec<String>, record: AuditOutboxRecord) -> AuditResult<()> {
        self.repository.delete_operations_with_audit(&validation::validate_ids(ids)?, &record).await
    }

    async fn clear_operations(&self) -> AuditResult<()> {
        self.repository.clear_operations().await
    }

    async fn clear_operations_with_audit(&self, record: AuditOutboxRecord) -> AuditResult<()> {
        self.repository.clear_operations_with_audit(&record).await
    }

    async fn page_logins(&self, filter: LoginLogFilter, page: CursorPageRequest) -> AuditResult<CursorPage<LoginLog>> {
        let query = login_cursor_query(&filter, &page)?;
        let slice = self.repository.page_logins(filter.clone(), query.clone()).await?;
        login_cursor_page(&filter, &query, slice)
    }

    async fn delete_login(&self, id: String) -> AuditResult<()> {
        self.repository.delete_logins(&validation::validate_ids(vec![id])?).await
    }

    async fn delete_logins(&self, ids: Vec<String>) -> AuditResult<()> {
        self.repository.delete_logins(&validation::validate_ids(ids)?).await
    }

    async fn delete_logins_with_audit(&self, ids: Vec<String>, record: AuditOutboxRecord) -> AuditResult<()> {
        self.repository.delete_logins_with_audit(&validation::validate_ids(ids)?, &record).await
    }

    async fn clear_logins(&self) -> AuditResult<()> {
        self.repository.clear_logins().await
    }

    async fn clear_logins_with_audit(&self, record: AuditOutboxRecord) -> AuditResult<()> {
        self.repository.clear_logins_with_audit(&record).await
    }
}
