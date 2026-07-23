use crate::application::{
    CompleteUpload, FileAccessScope, FileEntryView, FileManagementRepository, PartReceipt, ProviderPartReceipt, StoredObject, UploadCleanupCandidate,
    UploadCompletionTermination, UploadSessionData,
};
use crate::domain::UploadId;
use crate::error::keys;
use crate::{FileError, FileResult};

use super::{FileService, ensure_session_owner};

impl<R> FileService<R>
where
    R: FileManagementRepository,
{
    pub(super) async fn complete_managed_upload(&self, actor: FileAccessScope, session_id: UploadId) -> FileResult<FileEntryView> {
        let config = self.config.file_management_config().await?;
        let (existing, _) = self.repository.get_upload_session(&actor, session_id).await?.ok_or(FileError::UploadNotFound)?;
        ensure_session_owner(&actor, &existing)?;
        if existing.state == "completed" {
            return self.completed_entry(&actor, &existing).await;
        }
        if existing.state == "completing" {
            return self.completed_or_in_progress(&actor, session_id).await;
        }
        if existing.size > config.max_file_bytes {
            return Err(FileError::InvalidInput(keys::FILE_SIZE_EXCEEDED));
        }
        let (session, parts) = match self.repository.begin_upload_completion(&actor, session_id).await {
            Ok(value) => value,
            Err(FileError::UploadCompletionInProgress) => return self.completed_or_in_progress(&actor, session_id).await,
            Err(error) => return Err(error),
        };
        let object = match self.complete_provider_object(&session, parts).await {
            Ok(object) => object,
            Err(failure) => return self.handle_provider_completion_failure(&actor, session_id, &session, failure).await,
        };
        self.persist_upload_completion(&actor, session_id, object).await
    }

    async fn completed_entry(&self, actor: &FileAccessScope, session: &UploadSessionData) -> FileResult<FileEntryView> {
        let id = session.result_entry_id.ok_or(FileError::UploadResultUnavailable)?;
        self.completed_entry_by_id(actor, id).await
    }

    async fn completed_entry_by_id(&self, actor: &FileAccessScope, id: crate::domain::FileId) -> FileResult<FileEntryView> {
        self.repository.find_entry(actor, id).await?.ok_or(FileError::UploadResultUnavailable)
    }

    async fn completed_or_in_progress(&self, actor: &FileAccessScope, session_id: UploadId) -> FileResult<FileEntryView> {
        let (session, _) = self.repository.get_upload_session(actor, session_id).await?.ok_or(FileError::UploadNotFound)?;
        if session.state == "completed" {
            return self.completed_entry(actor, &session).await;
        }
        Err(FileError::UploadCompletionInProgress)
    }

    async fn complete_provider_object(&self, session: &UploadSessionData, parts: Vec<PartReceipt>) -> Result<StoredObject, ProviderCompletionFailure> {
        let provider = self.provider(&session.provider_key).map_err(ProviderCompletionFailure::from_error)?;
        match provider.stat(&session.provider_object_key).await {
            Ok(object) => Ok(object),
            Err(FileError::NotFound) => {
                match provider
                    .complete_upload(CompleteUpload {
                        provider_upload_ref: session.provider_upload_ref.clone(),
                        parts: parts.into_iter().map(provider_part).collect(),
                    })
                    .await
                {
                    Ok(object) => Ok(object),
                    Err(error) => self.reconcile_provider_completion(provider, session, error).await,
                }
            }
            Err(error) => Err(ProviderCompletionFailure::from_error(error)),
        }
    }

    async fn reconcile_provider_completion(
        &self,
        provider: &dyn crate::application::FileProvider,
        session: &UploadSessionData,
        error: FileError,
    ) -> Result<StoredObject, ProviderCompletionFailure> {
        if is_terminal_completion_error(&error) {
            return Err(ProviderCompletionFailure::from_error(error));
        }
        match provider.stat(&session.provider_object_key).await {
            Ok(object) => Ok(object),
            Err(FileError::NotFound) => Err(ProviderCompletionFailure::from_error(error)),
            Err(_) => Err(ProviderCompletionFailure::from_error(error)),
        }
    }

    async fn handle_provider_completion_failure(
        &self,
        actor: &FileAccessScope,
        session_id: UploadId,
        session: &UploadSessionData,
        failure: ProviderCompletionFailure,
    ) -> FileResult<FileEntryView> {
        if failure.terminal {
            return self.abort_terminal_completion(actor, session_id, session, failure.error).await;
        }
        if failure.reopen {
            self.repository.reopen_upload_completion(actor, session_id).await?;
        }
        Err(failure.error)
    }

    async fn abort_terminal_completion(
        &self,
        actor: &FileAccessScope,
        session_id: UploadId,
        session: &UploadSessionData,
        error: FileError,
    ) -> FileResult<FileEntryView> {
        match self.repository.abort_upload_completion_without_object(&actor.user_id, session_id).await? {
            UploadCompletionTermination::Completed(id) => self.completed_entry_by_id(actor, id).await,
            UploadCompletionTermination::Terminated => match self.abort_or_enqueue_upload(&session.provider_key, &session.provider_upload_ref).await? {
                super::provider_cleanup::AbortUploadOutcome::Aborted => Err(error),
                super::provider_cleanup::AbortUploadOutcome::Queued(cleanup_error) => Err(cleanup_error),
            },
        }
    }

    async fn persist_upload_completion(&self, actor: &FileAccessScope, session_id: UploadId, object: StoredObject) -> FileResult<FileEntryView> {
        let orphan = object.clone();
        match self.repository.finish_upload_session(actor, session_id, object).await {
            Ok(completion) => Ok(completion.entry),
            Err(error) if is_terminal_completion_error(&error) => match self.repository.abort_upload_completion(&actor.user_id, session_id, orphan).await? {
                UploadCompletionTermination::Terminated => Err(error),
                UploadCompletionTermination::Completed(id) => self.completed_entry_by_id(actor, id).await,
            },
            Err(error) => Err(error),
        }
    }

    pub(super) async fn finish_claimed_completion(&self, session: &UploadCleanupCandidate, object: StoredObject) -> FileResult<bool> {
        let orphan = object.clone();
        match self
            .repository
            .finish_claimed_upload_session(session.session_id, &session.claim_token, object)
            .await
        {
            Ok(_) => Ok(true),
            Err(error) if is_terminal_completion_error(&error) => {
                match self
                    .repository
                    .abort_claimed_upload_completion(session.session_id, &session.claim_token, orphan)
                    .await?
                {
                    UploadCompletionTermination::Terminated => Ok(false),
                    UploadCompletionTermination::Completed(_) => Ok(true),
                }
            }
            Err(error) => {
                self.repository.release_upload_cleanup_claim(session.session_id, &session.claim_token).await?;
                Err(error)
            }
        }
    }
}

struct ProviderCompletionFailure {
    error: FileError,
    reopen: bool,
    terminal: bool,
}

impl ProviderCompletionFailure {
    fn from_error(error: FileError) -> Self {
        let terminal = is_terminal_completion_error(&error);
        Self {
            error,
            reopen: !terminal,
            terminal,
        }
    }
}

fn provider_part(part: PartReceipt) -> ProviderPartReceipt {
    ProviderPartReceipt {
        part_number: part.part_number,
        provider_part_ref: part.provider_part_ref,
        size: part.size,
        digest: part.digest,
    }
}

fn is_terminal_completion_error(error: &FileError) -> bool {
    matches!(
        error,
        FileError::NameConflict
            | FileError::InvalidInput(_)
            | FileError::DigestMismatch
            | FileError::SizeMismatch
            | FileError::UploadIncomplete
            | FileError::InvalidPart
            | FileError::UploadPartConflict
    )
}
