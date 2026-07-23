use async_trait::async_trait;

use crate::application::{
    BeginUploadResult, BeginUploadSessionCommand, FileAccessScope, FileEntryView, FileManagementRepository, PartReceipt, UploadCommand, UploadPart,
    UploadPartClaimRequest, UploadPartClaimResult, UploadPartCommand, UploadReceipt, UploadSessionResponse,
};
use crate::{FileError, FileResult};

use super::provider_upload::ProviderUploadRequest;
use super::{FileService, ensure_session_owner, expected_part_size, session_response, validate_upload};

#[async_trait]
pub trait FileUploadUseCase: Send + Sync {
    async fn upload(&self, actor: FileAccessScope, command: UploadCommand) -> FileResult<UploadReceipt>;
    async fn store_avatar_asset(&self, actor: FileAccessScope, command: UploadCommand) -> FileResult<UploadReceipt>;
    async fn begin_upload_session(&self, actor: FileAccessScope, command: BeginUploadSessionCommand) -> FileResult<BeginUploadResult>;
    async fn get_upload_session(&self, actor: FileAccessScope, session_id: crate::domain::UploadId) -> FileResult<UploadSessionResponse>;
    async fn write_upload_part(&self, actor: FileAccessScope, command: UploadPartCommand) -> FileResult<PartReceipt>;
    async fn complete_upload_session(&self, actor: FileAccessScope, session_id: crate::domain::UploadId) -> FileResult<FileEntryView>;
    async fn cancel_upload_session(&self, actor: FileAccessScope, session_id: crate::domain::UploadId) -> FileResult<()>;
}

#[async_trait]
impl<R> FileUploadUseCase for FileService<R>
where
    R: FileManagementRepository,
{
    async fn upload(&self, actor: FileAccessScope, command: UploadCommand) -> FileResult<UploadReceipt> {
        let config = self.config.file_management_config().await?;
        validate_upload(&command, config.max_file_bytes)?;
        self.repository.ensure_target_space(&actor, &command.space_id).await?;
        if let Some(receipt) = self.try_reuse_upload(&actor, &command, config.default_space_quota_bytes).await? {
            return Ok(receipt);
        }
        self.create_uploaded_entry(&actor, command, config).await
    }

    async fn store_avatar_asset(&self, actor: FileAccessScope, command: UploadCommand) -> FileResult<UploadReceipt> {
        if actor.user_id != command.actor_user_id || actor.user_id != command.space_id.as_str() {
            return Err(FileError::Forbidden);
        }
        let parent_id = self.repository.ensure_avatar_folder(&actor.user_id, actor.department_id.as_deref()).await?;
        self.upload(actor, UploadCommand { parent_id, ..command }).await
    }

    async fn begin_upload_session(&self, actor: FileAccessScope, command: BeginUploadSessionCommand) -> FileResult<BeginUploadResult> {
        self.begin_managed_upload_session(actor, command).await
    }

    async fn get_upload_session(&self, actor: FileAccessScope, session_id: crate::domain::UploadId) -> FileResult<UploadSessionResponse> {
        let data = self.repository.get_upload_session(&actor, session_id).await?.ok_or(FileError::UploadNotFound)?;
        if data.0.owner_user_id != actor.user_id && !actor.can_manage_uploads {
            return Err(FileError::Forbidden);
        }
        Ok(session_response(data))
    }

    async fn write_upload_part(&self, actor: FileAccessScope, command: UploadPartCommand) -> FileResult<PartReceipt> {
        match self.claim_upload_part(&actor, command).await? {
            UploadPartWork::Completed(receipt) => Ok(receipt),
            UploadPartWork::Claimed(work) => self.write_claimed_part(&actor, *work).await,
        }
    }

    async fn complete_upload_session(&self, actor: FileAccessScope, session_id: crate::domain::UploadId) -> FileResult<FileEntryView> {
        self.complete_managed_upload(actor, session_id).await
    }

    async fn cancel_upload_session(&self, actor: FileAccessScope, session_id: crate::domain::UploadId) -> FileResult<()> {
        self.cancel_managed_upload(actor, session_id).await
    }
}

struct ClaimedUploadPart {
    session: crate::application::UploadSessionData,
    command: UploadPartCommand,
    token: String,
}

enum UploadPartWork {
    Completed(PartReceipt),
    Claimed(Box<ClaimedUploadPart>),
}

impl<R> FileService<R>
where
    R: FileManagementRepository,
{
    async fn try_reuse_upload(
        &self,
        actor: &FileAccessScope,
        command: &UploadCommand,
        default_quota: crate::domain::ByteSize,
    ) -> FileResult<Option<UploadReceipt>> {
        let space_id = command.space_id.clone();
        let size = command.size();
        let Some(existing) = self.repository.find_reusable_object(actor, space_id.clone(), command.digest(), size).await? else {
            return Ok(None);
        };
        self.repository.reserve_upload(space_id.clone(), size, default_quota).await?;
        let entry = match self.repository.create_reused_file(actor, command.clone(), existing).await {
            Ok(entry) => entry,
            Err(error) => {
                self.repository.release_upload(space_id, size).await?;
                return Err(error);
            }
        };
        Ok(Some(UploadReceipt {
            entry,
            reused_object: true,
            session_id: None,
        }))
    }

    async fn create_uploaded_entry(
        &self,
        actor: &FileAccessScope,
        command: UploadCommand,
        config: crate::application::FileManagementConfig,
    ) -> FileResult<UploadReceipt> {
        let space_id = command.space_id.clone();
        let size = command.size();
        self.repository.reserve_upload(space_id.clone(), size, config.default_space_quota_bytes).await?;
        let provider = self.provider(&self.active_provider)?;
        let object = match self
            .upload_provider(ProviderUploadRequest {
                command: &command,
                provider,
                object_id: crate::domain::StoredObjectId::new(),
                config,
            })
            .await
        {
            Ok(object) => object,
            Err(error) => {
                self.repository.release_upload(space_id, size).await?;
                return Err(error);
            }
        };
        self.persist_uploaded_entry(actor, command, object).await
    }

    async fn persist_uploaded_entry(
        &self,
        actor: &FileAccessScope,
        command: UploadCommand,
        object: crate::application::StoredObject,
    ) -> FileResult<UploadReceipt> {
        let space_id = command.space_id.clone();
        let size = command.size();
        let orphan = object.clone();
        let entry = match self.repository.create_uploaded_file(actor, command, object).await {
            Ok(entry) => entry,
            Err(error) => {
                let cleanup_result = self.record_orphan_object(&orphan).await;
                self.repository.release_upload(space_id, size).await?;
                cleanup_result?;
                return Err(error);
            }
        };
        Ok(UploadReceipt {
            entry,
            reused_object: false,
            session_id: None,
        })
    }

    async fn claim_upload_part(&self, actor: &FileAccessScope, command: UploadPartCommand) -> FileResult<UploadPartWork> {
        let (session, _) = self
            .repository
            .get_upload_session(actor, command.session_id)
            .await?
            .ok_or(FileError::UploadNotFound)?;
        ensure_session_owner(actor, &session)?;
        let expected_size = expected_part_size(&session, command.part_number)?;
        let claim = self
            .repository
            .claim_upload_part(
                actor,
                UploadPartClaimRequest {
                    session_id: command.session_id,
                    part_number: command.part_number,
                    digest: command.digest,
                    expected_size,
                },
            )
            .await?;
        match claim {
            UploadPartClaimResult::Claimed { token } => Ok(UploadPartWork::Claimed(Box::new(ClaimedUploadPart { session, command, token }))),
            UploadPartClaimResult::Completed(receipt) => Ok(UploadPartWork::Completed(receipt)),
        }
    }

    async fn write_claimed_part(&self, actor: &FileAccessScope, work: ClaimedUploadPart) -> FileResult<PartReceipt> {
        let provider = self.provider(&work.session.provider_key)?;
        let receipt = match provider
            .write_part(UploadPart {
                provider_upload_ref: work.session.provider_upload_ref.clone(),
                part_number: work.command.part_number,
                expected_digest: work.command.digest,
                body: work.command.body,
            })
            .await
        {
            Ok(receipt) => receipt,
            Err(error) => {
                self.repository
                    .release_upload_part_claim(work.command.session_id, work.command.part_number, &work.token)
                    .await?;
                return Err(error);
            }
        };
        let receipt = PartReceipt {
            session_id: work.command.session_id,
            part_number: work.command.part_number,
            provider_part_ref: receipt.provider_part_ref,
            size: receipt.size,
            digest: receipt.digest,
        };
        if let Err(error) = self.repository.complete_upload_part(actor, receipt.clone(), &work.token).await {
            self.repository
                .release_upload_part_claim(work.command.session_id, work.command.part_number, &work.token)
                .await?;
            return Err(error);
        }
        Ok(receipt)
    }
}
