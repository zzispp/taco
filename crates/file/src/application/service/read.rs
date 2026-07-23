use async_trait::async_trait;
use kernel::pagination::{CursorPage, CursorPageRequest};

use crate::application::{
    DirectoryTrailEntry, FileAccessScope, FileContent, FileContentMetadata, FileListQuery, FileManagementRepository, FilePage, FileReadRequest, FileSpaceQuery,
    FileSpaceView, ProviderSummary, bounded_text_preview, supports_inline_preview, supports_thumbnail, thumbnail_content,
};
use crate::domain::{DirectoryId, FileId, SpaceId};
use crate::{FileError, FileResult};

use super::FileService;

#[async_trait]
pub trait FileReadUseCase: Send + Sync {
    async fn list_entries(&self, actor: FileAccessScope, query: FileListQuery, page: CursorPageRequest) -> FileResult<FilePage>;
    async fn get_entry(&self, actor: FileAccessScope, id: FileId) -> FileResult<crate::application::FileEntryView>;
    async fn directory_trail(&self, actor: FileAccessScope, directory_id: DirectoryId) -> FileResult<Vec<DirectoryTrailEntry>>;
    async fn overview(&self, actor: FileAccessScope, space_id: Option<SpaceId>) -> FileResult<crate::application::FileOverviewView>;
    async fn list_spaces(&self, actor: FileAccessScope, query: FileSpaceQuery, page: CursorPageRequest) -> FileResult<CursorPage<FileSpaceView>>;
    async fn content(&self, actor: FileAccessScope, request: FileReadRequest) -> FileResult<FileContent>;
    async fn preview(&self, actor: FileAccessScope, request: FileReadRequest) -> FileResult<FileContent>;
    async fn thumbnail(&self, actor: FileAccessScope, id: FileId) -> FileResult<FileContent>;
    async fn provider_summaries(&self) -> FileResult<Vec<ProviderSummary>>;
}

#[async_trait]
impl<R> FileReadUseCase for FileService<R>
where
    R: FileManagementRepository,
{
    async fn list_entries(&self, actor: FileAccessScope, query: FileListQuery, page: CursorPageRequest) -> FileResult<FilePage> {
        self.repository.list_entries(&actor, query, page).await
    }

    async fn get_entry(&self, actor: FileAccessScope, id: FileId) -> FileResult<crate::application::FileEntryView> {
        self.repository.find_entry(&actor, id).await?.ok_or(FileError::NotFound)
    }

    async fn directory_trail(&self, actor: FileAccessScope, directory_id: DirectoryId) -> FileResult<Vec<DirectoryTrailEntry>> {
        let trail = self.repository.directory_trail(&actor, directory_id).await?;
        (!trail.is_empty()).then_some(trail).ok_or(FileError::NotFound)
    }

    async fn overview(&self, actor: FileAccessScope, space_id: Option<SpaceId>) -> FileResult<crate::application::FileOverviewView> {
        let config = self.config.file_management_config().await?;
        self.repository.overview(&actor, space_id, config.default_space_quota_bytes).await
    }

    async fn list_spaces(&self, actor: FileAccessScope, query: FileSpaceQuery, page: CursorPageRequest) -> FileResult<CursorPage<FileSpaceView>> {
        let config = self.config.file_management_config().await?;
        self.repository.list_spaces(&actor, query, page, config.default_space_quota_bytes).await
    }

    async fn content(&self, actor: FileAccessScope, request: FileReadRequest) -> FileResult<FileContent> {
        self.load_content(&actor, request).await
    }

    async fn preview(&self, actor: FileAccessScope, request: FileReadRequest) -> FileResult<FileContent> {
        let requested_range = request.range.is_some();
        let mut content = self.load_content(&actor, request).await?;
        if !supports_inline_preview(Some(&content.metadata.content_type)) {
            return Err(FileError::Forbidden);
        }
        if supports_thumbnail(Some(&content.metadata.content_type)) {
            if requested_range {
                return Err(FileError::RangeNotSatisfiable);
            }
            let content_type = content.metadata.content_type.clone();
            content.body = self.image_previews.validate_inline(&content_type, content.body).await?;
            content.metadata.accept_ranges = false;
        }
        Ok(bounded_text_preview(content))
    }

    async fn thumbnail(&self, actor: FileAccessScope, id: FileId) -> FileResult<FileContent> {
        let content = self.load_content(&actor, FileReadRequest { id, range: None }).await?;
        if !supports_thumbnail(Some(&content.metadata.content_type)) {
            return Err(FileError::Forbidden);
        }
        let thumbnail = self.image_previews.generate_thumbnail(&content.metadata.content_type, content.body).await?;
        Ok(thumbnail_content(content.metadata.name, thumbnail))
    }

    async fn provider_summaries(&self) -> FileResult<Vec<ProviderSummary>> {
        let mut result = Vec::with_capacity(self.providers.len());
        for (key, provider) in &self.providers {
            result.push(ProviderSummary {
                key: key.clone(),
                capacity: provider.capacity().await?,
            });
        }
        Ok(result)
    }
}

impl<R> FileService<R>
where
    R: FileManagementRepository,
{
    async fn load_content(&self, actor: &FileAccessScope, request: FileReadRequest) -> FileResult<FileContent> {
        let (entry, provider_key, key) = self.repository.read_content(actor, request.clone()).await?.ok_or(FileError::NotFound)?;
        let provider = self.provider(&provider_key)?;
        let range = match request.range {
            Some(range) => Some(range.resolve(provider.stat(&key).await?.size)?),
            None => None,
        };
        let read = provider.read_range(&key, range).await?;
        Ok(FileContent {
            metadata: FileContentMetadata {
                name: entry.name,
                content_type: entry
                    .mime_type
                    .ok_or_else(|| FileError::Infrastructure("stored file is missing its content type".into()))?,
                size: read.object.size,
                range: read.range,
                truncated: false,
                accept_ranges: true,
            },
            body: read.body,
        })
    }
}
