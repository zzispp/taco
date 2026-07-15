use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use storage::Database;

use crate::{
    application::{AuditCursorQuery, AuditCursorSlice, AuditExportSession, AuditRepository, AuditResult, LoginCursorBoundary, OperationCursorBoundary},
    domain::{LoginLog, LoginLogFilter, NewLoginLog, NewOperationLog, OperationLogDetail, OperationLogFilter, OperationLogSummary},
};

use super::{command, export_session::StorageAuditExportSession, query};

#[derive(Clone)]
pub struct StorageAuditRepository {
    database: Database,
}

impl StorageAuditRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn clear_operations_with_audit(&self, record: &AuditOutboxRecord) -> AuditResult<()> {
        command::clear_operations_with_audit(self.database.pool(), record).await
    }

    pub async fn clear_logins_with_audit(&self, record: &AuditOutboxRecord) -> AuditResult<()> {
        command::clear_logins_with_audit(self.database.pool(), record).await
    }
}

#[async_trait]
impl AuditRepository for StorageAuditRepository {
    async fn insert_operation(&self, log: NewOperationLog) -> AuditResult<()> {
        command::insert_operation(self.database.pool(), log).await
    }

    async fn page_operations(
        &self,
        filter: OperationLogFilter,
        page: AuditCursorQuery<OperationCursorBoundary>,
    ) -> AuditResult<AuditCursorSlice<OperationLogSummary>> {
        query::page_operations(self.database.pool(), filter, page).await
    }

    async fn begin_export(&self) -> AuditResult<Box<dyn AuditExportSession>> {
        Ok(Box::new(StorageAuditExportSession::begin(self.database.pool()).await?))
    }

    async fn find_operation(&self, id: &str) -> AuditResult<Option<OperationLogDetail>> {
        query::find_operation(self.database.pool(), id).await
    }

    async fn delete_operations(&self, ids: &[String]) -> AuditResult<()> {
        command::delete_operations(self.database.pool(), ids).await
    }

    async fn delete_operations_with_audit(&self, ids: &[String], record: &AuditOutboxRecord) -> AuditResult<()> {
        command::delete_operations_with_audit(self.database.pool(), ids, record).await
    }

    async fn clear_operations(&self) -> AuditResult<()> {
        command::clear_operations(self.database.pool()).await
    }

    async fn clear_operations_with_audit(&self, record: &AuditOutboxRecord) -> AuditResult<()> {
        StorageAuditRepository::clear_operations_with_audit(self, record).await
    }

    async fn insert_login(&self, log: NewLoginLog) -> AuditResult<()> {
        command::insert_login(self.database.pool(), self.database.next_id(), log).await
    }

    async fn page_logins(&self, filter: LoginLogFilter, page: AuditCursorQuery<LoginCursorBoundary>) -> AuditResult<AuditCursorSlice<LoginLog>> {
        query::page_logins(self.database.pool(), filter, page).await
    }

    async fn delete_logins(&self, ids: &[String]) -> AuditResult<()> {
        command::delete_logins(self.database.pool(), ids).await
    }

    async fn delete_logins_with_audit(&self, ids: &[String], record: &AuditOutboxRecord) -> AuditResult<()> {
        command::delete_logins_with_audit(self.database.pool(), ids, record).await
    }

    async fn clear_logins(&self) -> AuditResult<()> {
        command::clear_logins(self.database.pool()).await
    }

    async fn clear_logins_with_audit(&self, record: &AuditOutboxRecord) -> AuditResult<()> {
        StorageAuditRepository::clear_logins_with_audit(self, record).await
    }
}
