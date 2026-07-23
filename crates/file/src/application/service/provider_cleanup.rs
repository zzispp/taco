use crate::application::{CleanupObject, FileManagementRepository, ProviderCleanupCandidate, ProviderCleanupKind, ProviderUploadRef, StoredObject};
use crate::domain::ProviderKey;
use crate::{FileError, FileResult};

use super::FileService;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct ProviderCleanupRun {
    pub(super) completed: u64,
    pub(super) failed: u64,
}

pub(super) enum AbortUploadOutcome {
    Aborted,
    Queued(FileError),
}

impl<R> FileService<R>
where
    R: FileManagementRepository,
{
    pub(super) async fn process_provider_cleanups(&self, batch_size: u64) -> FileResult<ProviderCleanupRun> {
        let candidates = self.repository.claim_provider_cleanups(batch_size).await?;
        let mut report = ProviderCleanupRun::default();
        for candidate in candidates {
            if self.process_provider_cleanup(&candidate).await? {
                report.completed += 1;
            } else {
                report.failed += 1;
            }
        }
        Ok(report)
    }

    pub(super) async fn delete_managed_object(&self, object: &CleanupObject) -> FileResult<bool> {
        match self.provider(&object.provider_key)?.delete(&object.object_key).await {
            Ok(()) | Err(FileError::NotFound) => {
                self.repository.finalize_cleanup_object(object.object_id).await?;
                Ok(true)
            }
            Err(_) => {
                self.repository
                    .record_provider_cleanup(&object.provider_key, ProviderCleanupKind::Object, Some(&object.object_key), None)
                    .await?;
                Ok(false)
            }
        }
    }

    pub(super) async fn record_orphan_object(&self, object: &StoredObject) -> FileResult<()> {
        self.repository
            .record_provider_cleanup(&object.provider_key, ProviderCleanupKind::Object, Some(&object.key), None)
            .await
    }

    pub(super) async fn abort_or_enqueue_upload(&self, provider_key: &ProviderKey, upload_ref: &ProviderUploadRef) -> FileResult<AbortUploadOutcome> {
        let provider = match self.provider(provider_key) {
            Ok(provider) => provider,
            Err(error) => {
                self.repository
                    .record_provider_cleanup(provider_key, ProviderCleanupKind::Upload, None, Some(upload_ref))
                    .await?;
                return Ok(AbortUploadOutcome::Queued(error));
            }
        };
        match provider.abort_upload(upload_ref).await {
            Ok(()) | Err(FileError::NotFound) | Err(FileError::UploadNotFound) => Ok(AbortUploadOutcome::Aborted),
            Err(error) => {
                self.repository
                    .record_provider_cleanup(provider_key, ProviderCleanupKind::Upload, None, Some(upload_ref))
                    .await?;
                Ok(AbortUploadOutcome::Queued(error))
            }
        }
    }

    async fn process_provider_cleanup(&self, candidate: &ProviderCleanupCandidate) -> FileResult<bool> {
        match self.execute_provider_cleanup(candidate).await {
            Ok(()) => {
                self.repository.finalize_provider_cleanup(&candidate.cleanup_id, &candidate.claim_token).await?;
                Ok(true)
            }
            Err(error) => {
                self.repository
                    .release_provider_cleanup(&candidate.cleanup_id, &candidate.claim_token, cleanup_error_code(&error))
                    .await?;
                Ok(false)
            }
        }
    }

    async fn execute_provider_cleanup(&self, candidate: &ProviderCleanupCandidate) -> FileResult<()> {
        let result = match candidate.kind {
            ProviderCleanupKind::Object => {
                let key = candidate
                    .object_key
                    .as_ref()
                    .ok_or(FileError::Infrastructure("object cleanup is missing its object key".into()))?;
                self.provider(&candidate.provider_key)?.delete(key).await
            }
            ProviderCleanupKind::Upload => {
                let upload_ref = candidate
                    .upload_ref
                    .as_ref()
                    .ok_or(FileError::Infrastructure("upload cleanup is missing its provider reference".into()))?;
                self.provider(&candidate.provider_key)?.abort_upload(upload_ref).await
            }
        };
        match result {
            Err(FileError::NotFound) | Err(FileError::UploadNotFound) => Ok(()),
            other => other,
        }
    }
}

const fn cleanup_error_code(error: &FileError) -> &'static str {
    match error {
        FileError::ProviderUnavailable { .. } => "provider_unavailable",
        FileError::ProviderIo { .. } => "provider_io",
        FileError::NotFound | FileError::UploadNotFound => "provider_resource_not_found",
        _ => "provider_cleanup_failed",
    }
}
