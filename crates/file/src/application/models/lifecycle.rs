use serde::Serialize;

use crate::application::{ByteStream, FileEntryView, ObjectKey, PartReceipt, ProviderUploadRef, StoredObject};
use crate::domain::{ByteSize, ContentDigest, DirectoryId, EntryName, FileId, PartNumber, ProviderKey, SpaceId, StoredObjectId, UploadId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BeginUploadSessionCommand {
    pub space_id: SpaceId,
    pub parent_id: DirectoryId,
    pub name: EntryName,
    pub size: ByteSize,
    pub digest: ContentDigest,
    pub content_type: String,
    pub actor_user_id: String,
    pub idempotency_key: String,
}

pub struct UploadPartCommand {
    pub session_id: UploadId,
    pub part_number: PartNumber,
    pub digest: ContentDigest,
    pub body: ByteStream,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadPartClaimRequest {
    pub session_id: UploadId,
    pub part_number: PartNumber,
    pub digest: ContentDigest,
    pub expected_size: ByteSize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UploadPartClaimResult {
    Claimed { token: String },
    Completed(PartReceipt),
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileTrashCleanupReport {
    pub purged_entries: u64,
    pub blocked_roots: u64,
    pub deleted_objects: u64,
    pub failed_objects: u64,
    pub retried_provider_cleanups: u64,
    pub failed_provider_cleanups: u64,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct FileUploadCleanupReport {
    pub expired_sessions: u64,
    pub reconciled_sessions: u64,
    pub retried_provider_cleanups: u64,
    pub failed_provider_cleanups: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CleanupObject {
    pub object_id: StoredObjectId,
    pub provider_key: ProviderKey,
    pub object_key: ObjectKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderCleanupKind {
    Object,
    Upload,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCleanupCandidate {
    pub cleanup_id: String,
    pub provider_key: ProviderKey,
    pub kind: ProviderCleanupKind,
    pub object_key: Option<ObjectKey>,
    pub upload_ref: Option<ProviderUploadRef>,
    pub claim_token: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrashCleanupBatch {
    pub purged_entries: u64,
    pub blocked_roots: u64,
    pub objects: Vec<CleanupObject>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadCleanupCandidate {
    pub session_id: UploadId,
    pub owner_user_id: String,
    pub provider_key: ProviderKey,
    pub provider_upload_ref: ProviderUploadRef,
    pub provider_object_key: ObjectKey,
    pub expected_size: ByteSize,
    pub expected_digest: ContentDigest,
    pub claim_token: String,
    pub state: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadSessionData {
    pub id: UploadId,
    pub owner_user_id: String,
    pub space_id: crate::domain::SpaceId,
    pub parent_id: crate::domain::DirectoryId,
    pub name: crate::domain::EntryName,
    pub size: ByteSize,
    pub digest: ContentDigest,
    pub content_type: String,
    pub part_size: ByteSize,
    pub provider_key: ProviderKey,
    pub provider_upload_ref: ProviderUploadRef,
    pub provider_object_key: ObjectKey,
    pub state: String,
    pub result_entry_id: Option<FileId>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct UploadSessionResponse {
    pub id: String,
    pub space_id: String,
    pub parent_id: Option<String>,
    pub file_name: String,
    pub declared_size_bytes: u64,
    pub declared_sha256: String,
    pub content_type: String,
    pub part_size: u64,
    pub state: String,
    pub parts: Vec<PartReceiptResponse>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PartReceiptResponse {
    pub part_number: u32,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum BeginUploadResult {
    UploadRequired { session: UploadSessionResponse },
    Completed { entry: Box<FileEntryView> },
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PurgeReport {
    pub purged_entries: u64,
    pub objects: Vec<PurgeObjectResult>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct PurgeObjectResult {
    pub object_id: String,
    pub deleted: bool,
    pub error_code: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UploadCompletionResult {
    pub entry: FileEntryView,
    pub object_to_delete: Option<StoredObject>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UploadCompletionTermination {
    Terminated,
    Completed(FileId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PurgeBatch {
    pub purged_entries: u64,
    pub objects: Vec<CleanupObject>,
}
