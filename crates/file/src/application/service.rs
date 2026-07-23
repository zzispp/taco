use std::{collections::BTreeMap, sync::Arc};

use crate::application::{
    BeginUploadSessionCommand, FileAccessScope, FileManagementConfigProvider, FileManagementRepository, FileProvider, ImagePreviewProcessor, PartReceipt,
    UploadCleanupCandidate, UploadCommand, UploadSessionData, UploadSessionResponse,
};
use crate::domain::{ByteSize, PartNumber, ProviderKey};
use crate::error::keys;
use crate::{FileError, FileResult};

const IDEMPOTENCY_KEY_MAX_CHARS: usize = 255;

mod cleanup;
mod provider_cleanup;
mod provider_upload;
mod read;
mod upload;
mod upload_begin;
mod upload_cancel;
mod upload_completion;
mod write;

pub use cleanup::FileCleanupUseCase;
pub use read::FileReadUseCase;
pub use upload::FileUploadUseCase;
pub use write::FileWriteUseCase;

pub struct FileServiceDependencies {
    pub provider: Arc<dyn FileProvider>,
    pub config: Arc<dyn FileManagementConfigProvider>,
    pub image_previews: Arc<dyn ImagePreviewProcessor>,
}

/// Application service coordinating metadata transactions and opaque Providers.
pub struct FileService<R> {
    pub(super) repository: R,
    providers: BTreeMap<ProviderKey, Arc<dyn FileProvider>>,
    active_provider: ProviderKey,
    pub(super) config: Arc<dyn FileManagementConfigProvider>,
    pub(super) image_previews: Arc<dyn ImagePreviewProcessor>,
}

impl<R> FileService<R>
where
    R: FileManagementRepository,
{
    pub fn new(repository: R, dependencies: FileServiceDependencies) -> Self {
        let FileServiceDependencies {
            provider,
            config,
            image_previews,
        } = dependencies;
        let key = provider.provider_key();
        let providers = BTreeMap::from([(key.clone(), provider)]);
        Self {
            repository,
            providers,
            active_provider: key,
            config,
            image_previews,
        }
    }

    pub fn with_provider(mut self, provider: Arc<dyn FileProvider>) -> Self {
        self.providers.insert(provider.provider_key(), provider);
        self
    }

    pub fn repository(&self) -> &R {
        &self.repository
    }

    async fn cleanup_claimed_session(&self, session: &UploadCleanupCandidate) -> FileResult<bool> {
        let provider = self.provider(&session.provider_key)?;
        if session.state == "completing" {
            match provider.stat(&session.provider_object_key).await {
                Ok(object) => {
                    return self.finish_claimed_completion(session, object).await;
                }
                Err(FileError::NotFound) => {}
                Err(error) => {
                    self.repository.release_upload_cleanup_claim(session.session_id, &session.claim_token).await?;
                    return Err(error);
                }
            }
        }
        if let Err(error) = self.repository.finalize_expired_upload(session.session_id, &session.claim_token).await {
            self.repository.release_upload_cleanup_claim(session.session_id, &session.claim_token).await?;
            return Err(error);
        }
        match self.abort_or_enqueue_upload(&session.provider_key, &session.provider_upload_ref).await? {
            provider_cleanup::AbortUploadOutcome::Aborted => Ok(false),
            provider_cleanup::AbortUploadOutcome::Queued(error) => Err(error),
        }
    }

    fn provider(&self, key: &ProviderKey) -> FileResult<&dyn FileProvider> {
        self.providers.get(key).map(|provider| provider.as_ref()).ok_or(FileError::ProviderUnavailable {
            operation: "resolve file provider",
        })
    }
}

pub trait FileUseCase: FileReadUseCase + FileWriteUseCase + FileUploadUseCase {}

impl<T> FileUseCase for T where T: FileReadUseCase + FileWriteUseCase + FileUploadUseCase + ?Sized {}

fn max_part_size(provider_minimum: ByteSize, configured: ByteSize) -> ByteSize {
    if provider_minimum > configured { provider_minimum } else { configured }
}

fn validate_upload(command: &UploadCommand, max_file_size: ByteSize) -> FileResult<()> {
    let size = command.size();
    if size == ByteSize::ZERO {
        return Err(FileError::InvalidInput(keys::EMPTY_FILE));
    }
    if size > max_file_size {
        return Err(FileError::InvalidInput(keys::FILE_SIZE_EXCEEDED));
    }
    Ok(())
}

fn validate_session(command: &BeginUploadSessionCommand, max_file_size: ByteSize) -> FileResult<()> {
    if command.size == ByteSize::ZERO {
        return Err(FileError::InvalidInput(keys::EMPTY_FILE));
    }
    if command.size > max_file_size {
        return Err(FileError::InvalidInput(keys::FILE_SIZE_EXCEEDED));
    }
    if command.idempotency_key.trim().is_empty() {
        return Err(FileError::InvalidInput(keys::IDEMPOTENCY_KEY_REQUIRED));
    }
    if command.idempotency_key.chars().any(char::is_control) {
        return Err(FileError::InvalidInput(keys::IDEMPOTENCY_KEY_INVALID));
    }
    if command.idempotency_key.chars().count() > IDEMPOTENCY_KEY_MAX_CHARS {
        return Err(FileError::InvalidInput(keys::IDEMPOTENCY_KEY_TOO_LONG));
    }
    Ok(())
}

fn validate_intent(existing: &UploadSessionData, requested: &BeginUploadSessionCommand) -> FileResult<()> {
    if existing.space_id != requested.space_id
        || existing.parent_id != requested.parent_id
        || existing.name != requested.name
        || existing.size != requested.size
        || existing.digest != requested.digest
        || existing.content_type != requested.content_type
    {
        return Err(FileError::NameConflict);
    }
    Ok(())
}

fn ensure_session_owner(actor: &FileAccessScope, session: &UploadSessionData) -> FileResult<()> {
    if session.owner_user_id == actor.user_id {
        Ok(())
    } else {
        Err(FileError::Forbidden)
    }
}

fn expected_part_size(session: &UploadSessionData, part_number: PartNumber) -> FileResult<ByteSize> {
    let offset = u64::from(part_number.value() - 1)
        .checked_mul(session.part_size.bytes())
        .ok_or(FileError::SizeMismatch)?;
    if offset >= session.size.bytes() {
        return Err(FileError::InvalidPart);
    }
    Ok(ByteSize::from_bytes((session.size.bytes() - offset).min(session.part_size.bytes())))
}

fn session_response(value: (UploadSessionData, Vec<PartReceipt>)) -> UploadSessionResponse {
    let (session, parts) = value;
    UploadSessionResponse {
        id: session.id.to_string(),
        space_id: session.space_id.to_string(),
        parent_id: (session.parent_id != crate::domain::DirectoryId::ROOT).then(|| session.parent_id.to_string()),
        file_name: session.name.as_str().to_owned(),
        declared_size_bytes: session.size.bytes(),
        declared_sha256: session.digest.to_hex(),
        content_type: session.content_type,
        part_size: session.part_size.bytes(),
        state: session.state,
        parts: parts
            .into_iter()
            .map(|part| crate::application::PartReceiptResponse {
                part_number: part.part_number.value(),
                size_bytes: part.size.bytes(),
                sha256: part.digest.to_hex(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod validation_tests {
    use super::validate_session;
    use crate::FileError;
    use crate::application::BeginUploadSessionCommand;
    use crate::domain::{ByteSize, ContentDigest, DirectoryId, EntryName, SpaceId};
    use crate::error::keys;

    #[test]
    fn upload_intent_keys_respect_the_persistence_boundary() {
        let mut command = upload_command("x".repeat(256));
        assert_eq!(
            validate_session(&command, ByteSize::from_bytes(10)).unwrap_err(),
            FileError::InvalidInput(keys::IDEMPOTENCY_KEY_TOO_LONG)
        );
        command.idempotency_key = "invalid\nkey".into();
        assert_eq!(
            validate_session(&command, ByteSize::from_bytes(10)).unwrap_err(),
            FileError::InvalidInput(keys::IDEMPOTENCY_KEY_INVALID)
        );
    }

    fn upload_command(idempotency_key: String) -> BeginUploadSessionCommand {
        BeginUploadSessionCommand {
            space_id: SpaceId::new("actor").unwrap(),
            parent_id: DirectoryId::ROOT,
            name: EntryName::new("file.txt").unwrap(),
            size: ByteSize::from_bytes(4),
            digest: ContentDigest::from_bytes(b"data"),
            content_type: "text/plain".into(),
            actor_user_id: "actor".into(),
            idempotency_key,
        }
    }
}
