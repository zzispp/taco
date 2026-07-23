use async_trait::async_trait;
use kernel::error::LocalizedError;
use scheduler_macros::scheduled_task;
use serde::{Deserialize, Serialize};

use crate::application::SchedulerError;
use crate::application::task::{
    FileTrashCleanupResult, FileUploadSessionCleanupResult, ScheduledTask, TaskExecutionContext, TaskExecutionDetailPayload, TaskExecutionFailure,
    TaskExecutionOutput, TaskInvocation, TaskParams,
};

pub use super::file_cleanup_params::{CLEANUP_UPLOAD_SESSIONS_TASK_KEY, PURGE_TRASH_TASK_KEY};
use super::file_cleanup_params::{CleanupUploadSessionsParams, PurgeTrashParams};

pub const FILE_TRASH_CLEANUP_DETAIL_KIND: &str = "file_trash_cleanup";
pub const FILE_UPLOAD_SESSION_CLEANUP_DETAIL_KIND: &str = "file_upload_session_cleanup";
pub const FILE_CLEANUP_DETAIL_SCHEMA_VERSION: i16 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileCleanupKind {
    PurgeTrash,
    UploadSessions,
}

impl FileCleanupKind {
    const fn error_key(self) -> &'static str {
        match self {
            Self::PurgeTrash => "errors.scheduler.task_file_purge_trash_failed",
            Self::UploadSessions => "errors.scheduler.task_file_cleanup_upload_sessions_failed",
        }
    }
}

pub fn file_cleanup_failure(kind: FileCleanupKind, diagnostic: impl Into<String>) -> TaskExecutionFailure {
    TaskExecutionFailure::new(LocalizedError::new(kind.error_key()), diagnostic)
}

#[scheduled_task(
    task_key = PURGE_TRASH_TASK_KEY,
    name_key = "scheduler.tasks.file.purge_trash.name",
    group = "SYSTEM",
    group_key = "scheduler.task_groups.system",
    description_key = "scheduler.tasks.file.purge_trash.description",
    repeatable = false,
    lifecycle = scheduler::application::task::TaskLifecyclePolicy::RequiredPausable,
    params = PurgeTrashParams,
)]
#[derive(Default)]
pub struct PurgeTrashTask;

#[async_trait]
impl ScheduledTask for PurgeTrashTask {
    async fn execute(&self, context: TaskExecutionContext, invocation: TaskInvocation) -> Result<TaskExecutionOutput, TaskExecutionFailure> {
        let params = decode_params::<PurgeTrashParams>(&invocation)?;
        let result = context.file_cleanup.purge_trash(params.retention_days, params.batch_size).await?;
        Ok(TaskExecutionOutput::with_detail(FileTrashCleanupReport::from(result)))
    }
}

#[scheduled_task(
    task_key = CLEANUP_UPLOAD_SESSIONS_TASK_KEY,
    name_key = "scheduler.tasks.file.cleanup_upload_sessions.name",
    group = "SYSTEM",
    group_key = "scheduler.task_groups.system",
    description_key = "scheduler.tasks.file.cleanup_upload_sessions.description",
    repeatable = false,
    lifecycle = scheduler::application::task::TaskLifecyclePolicy::RequiredPausable,
    params = CleanupUploadSessionsParams,
)]
#[derive(Default)]
pub struct CleanupUploadSessionsTask;

#[async_trait]
impl ScheduledTask for CleanupUploadSessionsTask {
    async fn execute(&self, context: TaskExecutionContext, invocation: TaskInvocation) -> Result<TaskExecutionOutput, TaskExecutionFailure> {
        let params = decode_params::<CleanupUploadSessionsParams>(&invocation)?;
        let result = context.file_cleanup.cleanup_upload_sessions(params.batch_size).await?;
        Ok(TaskExecutionOutput::with_detail(FileUploadSessionCleanupReport::from(result)))
    }
}

fn decode_params<T>(invocation: &TaskInvocation) -> Result<T, TaskExecutionFailure>
where
    T: for<'de> Deserialize<'de> + TaskParams,
{
    T::validate(&invocation.task_params).map_err(task_params_failure)?;
    invocation.decode_params()
}

fn task_params_failure(error: SchedulerError) -> TaskExecutionFailure {
    TaskExecutionFailure::new(LocalizedError::new("errors.scheduler.invalid_params"), error.to_string())
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FileTrashCleanupReport {
    pub purged_entries: u64,
    pub blocked_roots: u64,
    pub deleted_objects: u64,
    pub failed_objects: u64,
    pub retried_provider_cleanups: u64,
    pub failed_provider_cleanups: u64,
}

impl From<FileTrashCleanupResult> for FileTrashCleanupReport {
    fn from(result: FileTrashCleanupResult) -> Self {
        Self {
            purged_entries: result.purged_entries,
            blocked_roots: result.blocked_roots,
            deleted_objects: result.deleted_objects,
            failed_objects: result.failed_objects,
            retried_provider_cleanups: result.retried_provider_cleanups,
            failed_provider_cleanups: result.failed_provider_cleanups,
        }
    }
}

impl TaskExecutionDetailPayload for FileTrashCleanupReport {
    const KIND: &'static str = FILE_TRASH_CLEANUP_DETAIL_KIND;
    const SCHEMA_VERSION: i16 = FILE_CLEANUP_DETAIL_SCHEMA_VERSION;
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FileUploadSessionCleanupReport {
    pub expired_sessions: u64,
    pub reconciled_sessions: u64,
    pub retried_provider_cleanups: u64,
    pub failed_provider_cleanups: u64,
}

impl From<FileUploadSessionCleanupResult> for FileUploadSessionCleanupReport {
    fn from(result: FileUploadSessionCleanupResult) -> Self {
        Self {
            expired_sessions: result.expired_sessions,
            reconciled_sessions: result.reconciled_sessions,
            retried_provider_cleanups: result.retried_provider_cleanups,
            failed_provider_cleanups: result.failed_provider_cleanups,
        }
    }
}

impl TaskExecutionDetailPayload for FileUploadSessionCleanupReport {
    const KIND: &'static str = FILE_UPLOAD_SESSION_CLEANUP_DETAIL_KIND;
    const SCHEMA_VERSION: i16 = FILE_CLEANUP_DETAIL_SCHEMA_VERSION;
}

#[cfg(test)]
#[path = "file_cleanup_tests.rs"]
mod tests;
