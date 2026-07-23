use bytes::Bytes;

use crate::application::{
    BeginUpload, CompleteUpload, FileManagementConfig, FileManagementRepository, FileProvider, ProviderPartReceipt, StoredObject, UploadCommand, UploadPart,
    UploadSession,
};
use crate::domain::{PartNumber, StoredObjectId};
use crate::{FileError, FileResult};

use super::{FileService, max_part_size, provider_cleanup::AbortUploadOutcome};

pub(super) struct ProviderUploadRequest<'a> {
    pub(super) command: &'a UploadCommand,
    pub(super) provider: &'a dyn FileProvider,
    pub(super) object_id: StoredObjectId,
    pub(super) config: FileManagementConfig,
}

impl<R> FileService<R>
where
    R: FileManagementRepository,
{
    pub(super) async fn upload_provider(&self, request: ProviderUploadRequest<'_>) -> FileResult<StoredObject> {
        let ProviderUploadRequest {
            command,
            provider,
            object_id,
            config,
        } = request;
        let session = provider
            .begin_upload(BeginUpload {
                stored_object_id: object_id,
                expected_size: command.size(),
                expected_digest: command.digest(),
                part_size: max_part_size(provider.minimum_part_size(), config.upload_part_bytes),
            })
            .await?;
        let parts = match self.write_parts(provider, &session, &command.bytes).await {
            Ok(parts) => parts,
            Err(error) => return self.fail_upload(&session, error).await,
        };
        match provider
            .complete_upload(CompleteUpload {
                provider_upload_ref: session.provider_upload_ref.clone(),
                parts,
            })
            .await
        {
            Ok(object) => Ok(object),
            Err(error) => self.reconcile_completion_error(provider, &session, error).await,
        }
    }

    async fn reconcile_completion_error(&self, provider: &dyn FileProvider, session: &UploadSession, error: FileError) -> FileResult<StoredObject> {
        match provider.stat(&session.key).await {
            Ok(object) => Ok(object),
            Err(FileError::NotFound) => self.fail_upload(session, error).await,
            Err(_) => {
                self.record_orphan_object(&expected_object(session)).await?;
                self.fail_upload(session, error).await
            }
        }
    }

    async fn fail_upload(&self, session: &UploadSession, error: FileError) -> FileResult<StoredObject> {
        match self.abort_or_enqueue_upload(&session.provider_key, &session.provider_upload_ref).await? {
            AbortUploadOutcome::Aborted => Err(error),
            AbortUploadOutcome::Queued(cleanup_error) => Err(cleanup_error),
        }
    }

    async fn write_parts(&self, provider: &dyn FileProvider, session: &UploadSession, bytes: &Bytes) -> FileResult<Vec<ProviderPartReceipt>> {
        let chunk_size = usize::try_from(session.part_size.bytes()).map_err(|_| FileError::SizeMismatch)?;
        let mut receipts = Vec::new();
        for (index, chunk) in bytes.chunks(chunk_size).enumerate() {
            let part_number = PartNumber::new(u32::try_from(index + 1).map_err(|_| FileError::InvalidPart)?)?;
            let digest = crate::domain::ContentDigest::from_bytes(chunk);
            let bytes = Bytes::copy_from_slice(chunk);
            let body = Box::pin(futures_util::stream::once(async move { Ok(bytes) }));
            receipts.push(
                provider
                    .write_part(UploadPart {
                        provider_upload_ref: session.provider_upload_ref.clone(),
                        part_number,
                        expected_digest: digest,
                        body,
                    })
                    .await?,
            );
        }
        Ok(receipts)
    }
}

fn expected_object(session: &UploadSession) -> StoredObject {
    StoredObject {
        id: session.stored_object_id,
        provider_key: session.provider_key.clone(),
        key: session.key.clone(),
        size: session.expected_size,
        digest: Some(session.expected_digest),
    }
}
