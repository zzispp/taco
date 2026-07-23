use async_trait::async_trait;

use crate::FileResult;
use crate::application::{FileManagementRepository, FileTrashCleanupReport, FileUploadCleanupReport};

use super::FileService;

#[async_trait]
pub trait FileCleanupUseCase: Send + Sync {
    async fn purge_trash(&self, retention_days: u64, batch_size: u64) -> FileResult<FileTrashCleanupReport>;
    async fn cleanup_upload_sessions(&self, batch_size: u64) -> FileResult<FileUploadCleanupReport>;
}

#[async_trait]
impl<R> FileCleanupUseCase for FileService<R>
where
    R: FileManagementRepository,
{
    async fn purge_trash(&self, retention_days: u64, batch_size: u64) -> FileResult<FileTrashCleanupReport> {
        let retried = self.process_provider_cleanups(batch_size).await?;
        let batch = self.repository.expired_trash(retention_days, batch_size).await?;
        let mut deleted_objects = 0_u64;
        let mut failed_objects = 0_u64;
        for object in batch.objects {
            if self.delete_managed_object(&object).await? {
                deleted_objects += 1;
            } else {
                failed_objects += 1;
            }
        }
        Ok(FileTrashCleanupReport {
            purged_entries: batch.purged_entries,
            blocked_roots: batch.blocked_roots,
            deleted_objects,
            failed_objects,
            retried_provider_cleanups: retried.completed,
            failed_provider_cleanups: retried.failed,
        })
    }

    async fn cleanup_upload_sessions(&self, batch_size: u64) -> FileResult<FileUploadCleanupReport> {
        let config = self.config.file_management_config().await?;
        let sessions = self
            .repository
            .expired_upload_sessions(config.upload_session_inactivity_days, batch_size)
            .await?;
        let retried = self.process_provider_cleanups(batch_size).await?;
        let mut report = FileUploadCleanupReport {
            retried_provider_cleanups: retried.completed,
            failed_provider_cleanups: retried.failed,
            ..FileUploadCleanupReport::default()
        };
        for session in sessions {
            if self.cleanup_claimed_session(&session).await? {
                report.reconciled_sessions += 1;
            } else {
                report.expired_sessions += 1;
            }
        }
        Ok(report)
    }
}
