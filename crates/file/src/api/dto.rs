use kernel::pagination::{CursorPageRequest, DEFAULT_CURSOR_LIMIT};
use rbac::{
    api::CurrentUser,
    domain::{DataScope, DataScopeFilter},
};
use serde::Deserialize;

use crate::application::{
    BatchIds, CreateFolderCommand, FileAccessScope, FileListQuery, FileScopeMode, FileSpaceQuery, UpdateEntryCommand, UpdateSpaceCommand,
};
use crate::domain::{ByteSize, ContentDigest};
use crate::domain::{DirectoryId, EntryName, FileId, SpaceId, TagName};
use crate::error::keys;
use crate::{FileError, FileResult};

const UPLOAD_MANAGEMENT_PERMISSION: &str = "file:upload:manage";

mod patch;
mod read_range;
use patch::NullablePatch;
pub use read_range::parse_read_range;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileListParams {
    pub limit: Option<u64>,
    pub cursor: Option<String>,
    pub space_id: Option<String>,
    pub parent_id: Option<String>,
    pub kind: Option<String>,
    pub search: Option<String>,
    pub extension: Option<String>,
    pub mime_type: Option<String>,
    pub tag: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub trashed: Option<bool>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

impl FileListParams {
    pub fn into_query(self) -> FileResult<(FileListQuery, CursorPageRequest)> {
        let page = cursor_page(self.limit, self.cursor)?;
        Ok((
            FileListQuery {
                cursor: page.cursor.clone(),
                space_id: self.space_id.map(SpaceId::new).transpose()?,
                parent_id: self.parent_id.map(|value| DirectoryId::parse(&value)).transpose()?,
                kind: self.kind,
                search: self.search,
                extension: self.extension,
                mime_type: self.mime_type,
                tag: self.tag,
                start_time: self.start_time,
                end_time: self.end_time,
                trashed: self.trashed,
                sort_by: self.sort_by,
                sort_order: self.sort_order,
            },
            page,
        ))
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileSpaceParams {
    pub limit: Option<u64>,
    pub cursor: Option<String>,
    pub owner_user_id: Option<String>,
    pub search: Option<String>,
    pub status: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

impl FileSpaceParams {
    pub fn into_query(self) -> FileResult<(FileSpaceQuery, CursorPageRequest)> {
        let page = cursor_page(self.limit, self.cursor)?;
        Ok((
            FileSpaceQuery {
                cursor: page.cursor.clone(),
                owner_user_id: self.owner_user_id,
                search: self.search,
                status: self.status,
                sort_by: self.sort_by,
                sort_order: self.sort_order,
            },
            page,
        ))
    }
}

fn cursor_page(limit: Option<u64>, cursor: Option<String>) -> FileResult<CursorPageRequest> {
    let page = CursorPageRequest {
        limit: limit.unwrap_or(DEFAULT_CURSOR_LIMIT),
        cursor,
    };
    page.validate().map_err(|_| FileError::InvalidInput(keys::CURSOR_LIMIT_INVALID))?;
    Ok(page)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateFolderPayload {
    pub space_id: String,
    pub parent_id: Option<String>,
    pub name: String,
}

impl CreateFolderPayload {
    pub fn into_command(self, actor: &CurrentUser) -> FileResult<CreateFolderCommand> {
        Ok(CreateFolderCommand {
            space_id: SpaceId::new(self.space_id)?,
            parent_id: self.parent_id.map(|value| DirectoryId::parse(&value)).transpose()?.unwrap_or(DirectoryId::ROOT),
            name: EntryName::new(self.name)?,
            actor_user_id: actor.id.clone(),
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateFilePayload {
    pub name: Option<String>,
    #[serde(default)]
    pub parent_id: NullablePatch<String>,
    pub tags: Option<Vec<String>>,
}

impl UpdateFilePayload {
    pub fn into_command(self, id: FileId, actor: &CurrentUser) -> FileResult<UpdateEntryCommand> {
        Ok(UpdateEntryCommand {
            id,
            name: self.name.map(EntryName::new).transpose()?,
            parent_id: match self.parent_id {
                NullablePatch::Missing => None,
                NullablePatch::Null => Some(DirectoryId::ROOT),
                NullablePatch::Value(value) => Some(DirectoryId::parse(&value)?),
            },
            tags: self.tags.map(|tags| tags.into_iter().map(TagName::new).collect()).transpose()?,
            actor_user_id: actor.id.clone(),
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchIdsPayload {
    pub ids: Vec<String>,
}

impl BatchIdsPayload {
    pub fn parse(self) -> FileResult<BatchIds> {
        BatchIds::parse(self.ids)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpaceQuotaPayload {
    pub quota_bytes: Option<u64>,
}

impl From<SpaceQuotaPayload> for UpdateSpaceCommand {
    fn from(value: SpaceQuotaPayload) -> Self {
        Self {
            quota_bytes: value.quota_bytes,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BeginUploadPayload {
    pub space_id: String,
    pub parent_id: Option<String>,
    pub file_name: String,
    pub declared_size_bytes: u64,
    pub declared_sha256: Option<String>,
    pub content_type: Option<String>,
}

impl BeginUploadPayload {
    pub fn into_command(self, actor_user_id: &str, idempotency_key: String) -> FileResult<crate::application::BeginUploadSessionCommand> {
        Ok(crate::application::BeginUploadSessionCommand {
            space_id: SpaceId::new(self.space_id)?,
            parent_id: self.parent_id.map(|value| DirectoryId::parse(&value)).transpose()?.unwrap_or(DirectoryId::ROOT),
            name: EntryName::new(self.file_name)?,
            size: ByteSize::from_bytes(self.declared_size_bytes),
            digest: ContentDigest::from_hex(&self.declared_sha256.ok_or(FileError::InvalidInput(keys::DECLARED_DIGEST_REQUIRED))?)?,
            content_type: normalize_content_type(self.content_type)?,
            actor_user_id: actor_user_id.to_owned(),
            idempotency_key,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompleteUploadPayload {
    pub parts: Vec<PartPayload>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartPayload {
    pub part_number: u32,
    pub size_bytes: u64,
    pub sha256: String,
}

pub fn part_command(
    session_id: crate::domain::UploadId,
    part_number: u32,
    digest: &str,
    body: crate::application::ByteStream,
) -> FileResult<crate::application::UploadPartCommand> {
    Ok(crate::application::UploadPartCommand {
        session_id,
        part_number: crate::domain::PartNumber::new(part_number)?,
        digest: ContentDigest::from_hex(digest)?,
        body,
    })
}

pub fn normalize_content_type(value: Option<String>) -> FileResult<String> {
    let value = value.unwrap_or_else(|| "application/octet-stream".into());
    let value = value.split(';').next().map(str::trim).unwrap_or_default().to_ascii_lowercase();
    if value.is_empty() || value.len() > 255 || value.chars().any(char::is_control) || !value.contains('/') {
        return Err(FileError::InvalidInput(keys::CONTENT_TYPE_INVALID));
    }
    Ok(value)
}

pub fn file_scope(current_user: &CurrentUser, filter: &DataScopeFilter) -> FileAccessScope {
    let (mode, departments) = match filter.data_scope {
        DataScope::All => (FileScopeMode::All, Vec::new()),
        DataScope::SelfOnly => (FileScopeMode::SelfOnly, Vec::new()),
        DataScope::Department => (FileScopeMode::Department, Vec::new()),
        DataScope::DepartmentAndChildren => (FileScopeMode::DepartmentAndChildren, Vec::new()),
        DataScope::Custom => (FileScopeMode::Custom, filter.dept_ids.clone()),
    };
    let can_manage = current_user.permissions.iter().any(|permission| permission == UPLOAD_MANAGEMENT_PERMISSION);
    FileAccessScope::scoped(current_user.id.clone(), mode, filter.dept_id.clone(), departments).with_upload_management(can_manage)
}

pub fn parse_file_id(id: String) -> FileResult<FileId> {
    FileId::parse(&id)
}

pub fn parse_directory_id(id: String) -> FileResult<DirectoryId> {
    DirectoryId::parse(&id)
}

pub fn parse_space_id(id: String) -> FileResult<SpaceId> {
    SpaceId::new(id)
}

pub fn parse_upload_id(id: String) -> FileResult<crate::domain::UploadId> {
    crate::domain::UploadId::parse(&id)
}

#[cfg(test)]
mod tests;
