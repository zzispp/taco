use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::StreamExt;
use user::{
    application::{AppError, AppResult, AvatarOwner, AvatarProjection, AvatarProjectionStorage, AvatarStorage, NormalizedAvatar},
    domain::AvatarFileId,
};

use crate::{
    FileError,
    application::{FileAccessScope, FileReadRequest, FileUseCase, UploadCommand},
    domain::{DirectoryId, EntryName, FileId, SpaceId},
};

const AVATAR_STORAGE_ERROR: &str = "infra.file.avatar_storage_failed";

#[derive(Clone)]
pub struct ManagedAvatarStorage {
    files: Arc<dyn FileUseCase>,
}

impl ManagedAvatarStorage {
    pub fn new(files: Arc<dyn FileUseCase>) -> Self {
        Self { files }
    }
}

#[async_trait]
impl AvatarStorage for ManagedAvatarStorage {
    async fn store_avatar(&self, owner: AvatarOwner, avatar: NormalizedAvatar) -> AppResult<AvatarFileId> {
        let command = avatar_upload_command(&owner, avatar)?;
        let entry = self.files.store_avatar_asset(owner_scope(&owner), command).await.map_err(file_error)?.entry;
        AvatarFileId::new(entry.id).map_err(|_| AppError::Infrastructure(AVATAR_STORAGE_ERROR.into()))
    }

    async fn trash_avatar(&self, owner: AvatarOwner, file_id: AvatarFileId) -> AppResult<()> {
        let file_id = FileId::parse(file_id.as_str()).map_err(file_error)?;
        self.files.trash(owner_scope(&owner), vec![file_id]).await.map_err(file_error)
    }
}

#[async_trait]
impl AvatarProjectionStorage for ManagedAvatarStorage {
    async fn load_avatar_projection(&self, owner: AvatarOwner, file_id: AvatarFileId) -> AppResult<AvatarProjection> {
        let request = FileReadRequest {
            id: FileId::parse(file_id.as_str()).map_err(file_error)?,
            range: None,
        };
        let content = self.files.content(owner_scope(&owner), request).await.map_err(file_error)?;
        if !content.metadata.content_type.starts_with("image/") {
            return Err(AppError::NotFound);
        }
        Ok(AvatarProjection {
            content_type: content.metadata.content_type,
            content_length: content.metadata.size.bytes(),
            body: Box::pin(content.body.map(|item| item.map_err(file_error))),
        })
    }
}

fn avatar_upload_command(owner: &AvatarOwner, avatar: NormalizedAvatar) -> AppResult<UploadCommand> {
    Ok(UploadCommand {
        space_id: SpaceId::new(owner.user_id.clone()).map_err(file_error)?,
        parent_id: DirectoryId::ROOT,
        name: avatar_name(avatar.extension)?,
        content_type: avatar.content_type.into(),
        bytes: Bytes::from(avatar.bytes),
        actor_user_id: owner.user_id.clone(),
        idempotency_key: None,
    })
}

fn avatar_name(extension: &str) -> AppResult<EntryName> {
    let timestamp = time::OffsetDateTime::now_utc().unix_timestamp_nanos();
    EntryName::new(format!("avatar-{timestamp}.{extension}")).map_err(file_error)
}

fn owner_scope(owner: &AvatarOwner) -> FileAccessScope {
    FileAccessScope::self_only(owner.user_id.clone(), owner.department_id.clone())
}

fn file_error(error: FileError) -> AppError {
    taco_tracing::error_with_fields!("avatar file storage operation failed", &error, component = "file_avatar_storage");
    AppError::Infrastructure(AVATAR_STORAGE_ERROR.into())
}
