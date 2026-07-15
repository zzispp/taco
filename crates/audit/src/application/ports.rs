use async_trait::async_trait;
use audit_contract::AuditOutboxRecord;
use kernel::pagination::{CursorPage, CursorPageRequest};

use crate::domain::{LoginLog, LoginLogFilter, NewLoginLog, NewOperationLog, OperationLogDetail, OperationLogFilter, OperationLogSummary};

use super::{AuditCursorQuery, AuditCursorSlice, AuditResult, LoginCursorBoundary, OperationCursorBoundary};

/// Persists and queries audit records without owning delivery or retry policy.
///
/// Implementations must preserve exact-delete and export transaction semantics,
/// use parameterized storage access, and return every storage failure.
#[async_trait]
pub trait AuditRepository: Send + Sync + 'static {
    async fn insert_operation(&self, log: NewOperationLog) -> AuditResult<()>;
    async fn page_operations(
        &self,
        filter: OperationLogFilter,
        page: AuditCursorQuery<OperationCursorBoundary>,
    ) -> AuditResult<AuditCursorSlice<OperationLogSummary>>;
    async fn begin_export(&self) -> AuditResult<Box<dyn AuditExportSession>>;
    async fn find_operation(&self, id: &str) -> AuditResult<Option<OperationLogDetail>>;
    async fn delete_operations(&self, ids: &[String]) -> AuditResult<()>;
    async fn delete_operations_with_audit(&self, ids: &[String], record: &AuditOutboxRecord) -> AuditResult<()>;
    async fn clear_operations(&self) -> AuditResult<()>;
    async fn clear_operations_with_audit(&self, record: &AuditOutboxRecord) -> AuditResult<()>;
    async fn insert_login(&self, log: NewLoginLog) -> AuditResult<()>;
    async fn page_logins(&self, filter: LoginLogFilter, page: AuditCursorQuery<LoginCursorBoundary>) -> AuditResult<AuditCursorSlice<LoginLog>>;
    async fn delete_logins(&self, ids: &[String]) -> AuditResult<()>;
    async fn delete_logins_with_audit(&self, ids: &[String], record: &AuditOutboxRecord) -> AuditResult<()>;
    async fn clear_logins(&self) -> AuditResult<()>;
    async fn clear_logins_with_audit(&self, record: &AuditOutboxRecord) -> AuditResult<()>;
}

/// Exposes validated audit management use cases to the HTTP API.
///
/// Implementations enforce pagination and identifier invariants and propagate
/// repository failures without converting them into successful responses.
#[async_trait]
pub trait AuditUseCase: Send + Sync + 'static {
    async fn page_operations(&self, filter: OperationLogFilter, page: CursorPageRequest) -> AuditResult<CursorPage<OperationLogSummary>>;
    async fn begin_export(&self) -> AuditResult<Box<dyn AuditExportSession>>;
    async fn operation_detail(&self, id: &str) -> AuditResult<OperationLogDetail>;
    async fn delete_operation(&self, id: String) -> AuditResult<()>;
    async fn delete_operations(&self, ids: Vec<String>) -> AuditResult<()>;
    async fn delete_operations_with_audit(&self, ids: Vec<String>, record: AuditOutboxRecord) -> AuditResult<()>;
    async fn clear_operations(&self) -> AuditResult<()>;
    async fn clear_operations_with_audit(&self, record: AuditOutboxRecord) -> AuditResult<()>;
    async fn page_logins(&self, filter: LoginLogFilter, page: CursorPageRequest) -> AuditResult<CursorPage<LoginLog>>;
    async fn delete_login(&self, id: String) -> AuditResult<()>;
    async fn delete_logins(&self, ids: Vec<String>) -> AuditResult<()>;
    async fn delete_logins_with_audit(&self, ids: Vec<String>, record: AuditOutboxRecord) -> AuditResult<()>;
    async fn clear_logins(&self) -> AuditResult<()>;
    async fn clear_logins_with_audit(&self, record: AuditOutboxRecord) -> AuditResult<()>;
}

/// Holds one read-only repeatable-read export snapshot until explicitly finished.
///
/// Implementations keep all batches in one snapshot and expose query or commit
/// failures; callers must invoke `finish` after the last page.
#[async_trait]
pub trait AuditExportSession: Send {
    async fn page_operations(
        &mut self,
        filter: OperationLogFilter,
        page: AuditCursorQuery<OperationCursorBoundary>,
    ) -> AuditResult<AuditCursorSlice<OperationLogSummary>>;
    async fn page_logins(&mut self, filter: LoginLogFilter, page: AuditCursorQuery<LoginCursorBoundary>) -> AuditResult<AuditCursorSlice<LoginLog>>;
    async fn finish(self: Box<Self>) -> AuditResult<()>;
}

/// Clears the owning authentication context's login-failure state.
///
/// Implementations delegate to the authentication bounded context and expose
/// its failure without fallback or audit-specific state duplication.
#[async_trait]
pub trait LoginUnlocker: Send + Sync + 'static {
    async fn unlock(&self, username: &str) -> AuditResult<()>;
}
