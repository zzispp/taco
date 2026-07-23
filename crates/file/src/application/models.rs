use bytes::Bytes;
use kernel::pagination::CursorPage;
use serde::Serialize;

use crate::application::ByteStream;
use crate::domain::{ByteSize, ContentDigest, DirectoryId, EntryName, FileId, ProviderKey, SpaceId, StoredObjectId, TagName, UploadId};
use crate::error::keys;
use crate::{FileError, FileResult};

pub(crate) const MAX_BATCH_FILE_IDS: usize = 100;

mod access;
mod lifecycle;
mod read;

pub use access::*;
pub use lifecycle::*;
pub use read::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateFolderCommand {
    pub space_id: SpaceId,
    pub parent_id: DirectoryId,
    pub name: EntryName,
    pub actor_user_id: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UpdateEntryCommand {
    pub id: FileId,
    pub name: Option<EntryName>,
    pub parent_id: Option<DirectoryId>,
    pub tags: Option<Vec<TagName>>,
    pub actor_user_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadCommand {
    pub space_id: SpaceId,
    pub parent_id: DirectoryId,
    pub name: EntryName,
    pub content_type: String,
    pub bytes: Bytes,
    pub actor_user_id: String,
    pub idempotency_key: Option<String>,
}

impl UploadCommand {
    pub fn digest(&self) -> ContentDigest {
        ContentDigest::from_bytes(&self.bytes)
    }

    pub fn size(&self) -> ByteSize {
        ByteSize::from_bytes(self.bytes.len() as u64)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FileProperties {
    pub checksum_sha256: Option<String>,
    pub extension: Option<String>,
    pub mime_type: Option<String>,
    pub created_by: Option<String>,
    pub provider_key: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FileEntryView {
    pub id: String,
    pub space_id: String,
    pub owner_user_id: String,
    pub owner_name: Option<String>,
    pub parent_id: Option<String>,
    pub name: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub size_bytes: u64,
    pub mime_type: Option<String>,
    pub object_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub trashed_at: Option<String>,
    pub tags: Vec<String>,
    pub properties: FileProperties,
    pub preview_supported: bool,
    pub download_only: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DirectoryTrailEntry {
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FileOverviewView {
    pub space_id: String,
    pub logical_asset_size: u64,
    pub managed_physical_usage: u64,
    pub recycle_bin_size: u64,
    pub temporary_upload_size: u64,
    pub deduplication_savings: u64,
    pub quota_bytes: u64,
    pub quota_reserved_bytes: u64,
    pub type_distribution: Vec<TypeDistributionView>,
    pub recent_entries: Vec<FileEntryView>,
    pub recent_folders: Vec<FileEntryView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TypeDistributionView {
    pub entry_type: String,
    pub bytes: u64,
    pub count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FileSpaceView {
    pub id: String,
    pub owner_user_id: String,
    pub owner_name: String,
    pub department_name: Option<String>,
    pub status: String,
    pub logical_asset_size: u64,
    pub managed_physical_usage: u64,
    pub reserved_bytes: u64,
    pub quota_bytes: u64,
    pub updated_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExistingObject {
    pub object_id: StoredObjectId,
    pub provider_key: ProviderKey,
    pub object_key: String,
    pub size: ByteSize,
    pub digest: ContentDigest,
}

pub type FilePage = CursorPage<FileEntryView>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProviderSummary {
    pub key: ProviderKey,
    pub capacity: crate::domain::ProviderCapacity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadReceipt {
    pub entry: FileEntryView,
    pub reused_object: bool,
    pub session_id: Option<UploadId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchIds {
    pub ids: Vec<FileId>,
}

impl BatchIds {
    pub fn parse(values: Vec<String>) -> FileResult<Self> {
        let ids = values.into_iter().map(|value| FileId::parse(&value)).collect::<FileResult<Vec<_>>>()?;
        Ok(Self {
            ids: normalize_batch_ids(ids)?,
        })
    }
}

pub(crate) fn normalize_batch_ids(ids: impl IntoIterator<Item = FileId>) -> FileResult<Vec<FileId>> {
    let ids = ids.into_iter().collect::<std::collections::BTreeSet<_>>().into_iter().collect::<Vec<_>>();
    if ids.is_empty() {
        return Err(FileError::InvalidInput(keys::FILE_IDS_REQUIRED));
    }
    if ids.len() > MAX_BATCH_FILE_IDS {
        return Err(FileError::InvalidInput(keys::FILE_IDS_TOO_MANY));
    }
    Ok(ids)
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UpdateSpaceCommand {
    pub quota_bytes: Option<u64>,
}

pub fn as_object_stream(bytes: Bytes) -> ByteStream {
    Box::pin(futures_util::stream::once(async move { Ok(bytes) }))
}

#[cfg(test)]
mod tests {
    use crate::domain::FileId;
    use crate::error::keys;
    use crate::{FileError, application::BatchIds};

    #[test]
    fn batch_ids_deduplicate_into_a_deterministic_order() {
        let first = FileId::new();
        let second = FileId::new();
        let batch = BatchIds::parse(vec![second.to_string(), first.to_string(), second.to_string()]).unwrap();

        assert_eq!(batch.ids, vec![first.min(second), first.max(second)]);
    }

    #[test]
    fn batch_ids_reject_more_than_one_hundred_distinct_entries() {
        let ids = (0..101).map(|_| FileId::new().to_string()).collect();

        assert_eq!(BatchIds::parse(ids).unwrap_err(), FileError::InvalidInput(keys::FILE_IDS_TOO_MANY));
    }
}
