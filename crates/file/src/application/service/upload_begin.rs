use crate::application::{BeginUpload, BeginUploadResult, BeginUploadSessionCommand, FileAccessScope, FileManagementRepository, UploadSession};
use crate::{FileError, FileResult};

use super::{FileService, max_part_size, session_response, validate_intent, validate_session};

struct ReuseIntent {
    command: BeginUploadSessionCommand,
    object: crate::application::ExistingObject,
    part_size: crate::domain::ByteSize,
    default_quota: crate::domain::ByteSize,
}

struct ProviderIntent {
    command: BeginUploadSessionCommand,
    config: crate::application::FileManagementConfig,
}

struct FailedProviderIntent<'a> {
    command: &'a BeginUploadSessionCommand,
    provider_session: &'a UploadSession,
    error: FileError,
}

impl<R> FileService<R>
where
    R: FileManagementRepository,
{
    pub(super) async fn begin_managed_upload_session(&self, actor: FileAccessScope, command: BeginUploadSessionCommand) -> FileResult<BeginUploadResult> {
        let config = self.config.file_management_config().await?;
        validate_session(&command, config.max_file_bytes)?;
        self.repository.ensure_target_space(&actor, &command.space_id).await?;
        if let Some(result) = self.existing_intent_result(&actor, &command).await? {
            return Ok(result);
        }
        if let Some(object) = self
            .repository
            .find_reusable_object(&actor, command.space_id.clone(), command.digest, command.size)
            .await?
        {
            return self
                .begin_reused_intent(
                    &actor,
                    ReuseIntent {
                        command,
                        object,
                        part_size: config.upload_part_bytes,
                        default_quota: config.default_space_quota_bytes,
                    },
                )
                .await;
        }
        self.begin_provider_intent(&actor, ProviderIntent { command, config }).await
    }

    async fn existing_intent_result(&self, actor: &FileAccessScope, command: &BeginUploadSessionCommand) -> FileResult<Option<BeginUploadResult>> {
        let existing = self
            .repository
            .find_upload_intent(actor, &actor.user_id, command.space_id.clone(), &command.idempotency_key)
            .await?;
        let Some(existing) = existing else {
            return Ok(None);
        };
        validate_intent(&existing.0, command)?;
        self.existing_intent_response(actor, existing).await.map(Some)
    }

    async fn existing_intent_response(
        &self,
        actor: &FileAccessScope,
        existing: (crate::application::UploadSessionData, Vec<crate::application::PartReceipt>),
    ) -> FileResult<BeginUploadResult> {
        if existing.0.state == "completed" {
            let id = existing.0.result_entry_id.ok_or(FileError::UploadResultUnavailable)?;
            let entry = self.repository.find_entry(actor, id).await?.ok_or(FileError::UploadResultUnavailable)?;
            return Ok(BeginUploadResult::Completed { entry: Box::new(entry) });
        }
        if matches!(existing.0.state.as_str(), "aborted" | "expired") {
            return Err(FileError::UploadIntentTerminal);
        }
        Ok(BeginUploadResult::UploadRequired {
            session: session_response(existing),
        })
    }

    async fn begin_reused_intent(&self, actor: &FileAccessScope, intent: ReuseIntent) -> FileResult<BeginUploadResult> {
        self.repository
            .reserve_upload(intent.command.space_id.clone(), intent.command.size, intent.default_quota)
            .await?;
        let result = self
            .repository
            .create_reused_upload(actor, intent.command.clone(), intent.object, intent.part_size)
            .await;
        match result {
            Ok(entry) => Ok(BeginUploadResult::Completed { entry: Box::new(entry) }),
            Err(error) => self.release_and_resolve_competing_intent(actor, &intent.command, error).await,
        }
    }

    async fn begin_provider_intent(&self, actor: &FileAccessScope, intent: ProviderIntent) -> FileResult<BeginUploadResult> {
        let ProviderIntent { command, config } = intent;
        self.repository
            .reserve_upload(command.space_id.clone(), command.size, config.default_space_quota_bytes)
            .await?;
        let provider = match self.provider(&self.active_provider) {
            Ok(provider) => provider,
            Err(error) => return self.release_upload_after_begin_failure(&command, error).await,
        };
        let provider_session = match provider
            .begin_upload(BeginUpload {
                stored_object_id: crate::domain::StoredObjectId::new(),
                expected_size: command.size,
                expected_digest: command.digest,
                part_size: max_part_size(provider.minimum_part_size(), config.upload_part_bytes),
            })
            .await
        {
            Ok(session) => session,
            Err(error) => return self.release_upload_after_begin_failure(&command, error).await,
        };
        match self.repository.create_upload_session(actor, command.clone(), provider_session.clone()).await {
            Ok(data) => self.upload_required_response(actor, data.id).await,
            Err(error) => {
                self.cleanup_and_resolve_competing_intent(
                    actor,
                    FailedProviderIntent {
                        command: &command,
                        provider_session: &provider_session,
                        error,
                    },
                )
                .await
            }
        }
    }

    async fn upload_required_response(&self, actor: &FileAccessScope, session_id: crate::domain::UploadId) -> FileResult<BeginUploadResult> {
        let session = self.repository.get_upload_session(actor, session_id).await?.ok_or(FileError::UploadNotFound)?;
        Ok(BeginUploadResult::UploadRequired {
            session: session_response(session),
        })
    }

    async fn release_upload_after_begin_failure(&self, command: &BeginUploadSessionCommand, error: FileError) -> FileResult<BeginUploadResult> {
        self.repository.release_upload(command.space_id.clone(), command.size).await?;
        Err(error)
    }

    async fn cleanup_and_resolve_competing_intent(&self, actor: &FileAccessScope, failed: FailedProviderIntent<'_>) -> FileResult<BeginUploadResult> {
        self.repository.release_upload(failed.command.space_id.clone(), failed.command.size).await?;
        let _cleanup_outcome = self
            .abort_or_enqueue_upload(&failed.provider_session.provider_key, &failed.provider_session.provider_upload_ref)
            .await?;
        // Cleanup is durable once it has been queued; the idempotency response is the winning session.
        self.competing_intent_or_error(actor, failed.command, failed.error).await
    }

    async fn release_and_resolve_competing_intent(
        &self,
        actor: &FileAccessScope,
        command: &BeginUploadSessionCommand,
        error: FileError,
    ) -> FileResult<BeginUploadResult> {
        self.repository.release_upload(command.space_id.clone(), command.size).await?;
        self.competing_intent_or_error(actor, command, error).await
    }

    async fn competing_intent_or_error(
        &self,
        actor: &FileAccessScope,
        command: &BeginUploadSessionCommand,
        original: FileError,
    ) -> FileResult<BeginUploadResult> {
        self.existing_intent_result(actor, command).await?.ok_or(original)
    }
}
