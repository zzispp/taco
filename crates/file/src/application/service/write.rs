use async_trait::async_trait;

use crate::FileResult;
use crate::application::{
    CreateFolderCommand, FileAccessScope, FileEntryView, FileManagementRepository, FileSpaceView, PurgeObjectResult, PurgeReport, UpdateEntryCommand,
    UpdateSpaceCommand, normalize_batch_ids,
};
use crate::domain::{FileId, SpaceId};

use super::FileService;

#[async_trait]
pub trait FileWriteUseCase: Send + Sync {
    async fn create_folder(&self, actor: FileAccessScope, command: CreateFolderCommand) -> FileResult<FileEntryView>;
    async fn update_entry(&self, actor: FileAccessScope, command: UpdateEntryCommand) -> FileResult<FileEntryView>;
    async fn trash(&self, actor: FileAccessScope, ids: Vec<FileId>) -> FileResult<()>;
    async fn restore(&self, actor: FileAccessScope, ids: Vec<FileId>) -> FileResult<()>;
    async fn purge(&self, actor: FileAccessScope, ids: Vec<FileId>) -> FileResult<PurgeReport>;
    async fn update_space(&self, actor: FileAccessScope, space_id: SpaceId, command: UpdateSpaceCommand) -> FileResult<FileSpaceView>;
}

#[async_trait]
impl<R> FileWriteUseCase for FileService<R>
where
    R: FileManagementRepository,
{
    async fn create_folder(&self, actor: FileAccessScope, command: CreateFolderCommand) -> FileResult<FileEntryView> {
        self.repository.ensure_target_space(&actor, &command.space_id).await?;
        self.repository.create_folder(&actor, command).await
    }

    async fn update_entry(&self, actor: FileAccessScope, command: UpdateEntryCommand) -> FileResult<FileEntryView> {
        self.repository.update_entry(&actor, command).await
    }

    async fn trash(&self, actor: FileAccessScope, ids: Vec<FileId>) -> FileResult<()> {
        self.repository.trash(&actor, &normalize_batch_ids(ids)?).await
    }

    async fn restore(&self, actor: FileAccessScope, ids: Vec<FileId>) -> FileResult<()> {
        self.repository.restore(&actor, &normalize_batch_ids(ids)?).await
    }

    async fn purge(&self, actor: FileAccessScope, ids: Vec<FileId>) -> FileResult<PurgeReport> {
        let batch = self.repository.purge(&actor, &normalize_batch_ids(ids)?).await?;
        let mut objects = Vec::with_capacity(batch.objects.len());
        for object in batch.objects {
            let deleted = self.delete_managed_object(&object).await?;
            let result = PurgeObjectResult {
                object_id: object.object_id.to_string(),
                deleted,
                error_code: (!deleted).then_some("provider_delete_queued".into()),
            };
            objects.push(result);
        }
        Ok(PurgeReport {
            purged_entries: batch.purged_entries,
            objects,
        })
    }

    async fn update_space(&self, actor: FileAccessScope, space_id: SpaceId, command: UpdateSpaceCommand) -> FileResult<FileSpaceView> {
        let config = self.config.file_management_config().await?;
        self.repository.update_space(&actor, space_id, command, config.default_space_quota_bytes).await
    }
}
