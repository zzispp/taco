use async_trait::async_trait;

use super::models::{
    BeginUploadSessionCommand, CreateFolderCommand, DirectoryTrailEntry, ExistingObject, FileAccessScope, FileEntryView, FileListQuery, FileOverviewView,
    FilePage, FileReadRequest, FileSpaceQuery, FileSpaceView, ProviderCleanupCandidate, ProviderCleanupKind, TrashCleanupBatch, UpdateEntryCommand,
    UpdateSpaceCommand, UploadCleanupCandidate, UploadCommand, UploadCompletionResult, UploadPartClaimRequest, UploadPartClaimResult, UploadSessionData,
};
use crate::domain::{DirectoryId, FileId, ProviderCapacity, ProviderKey, SpaceId, StoredObjectId, UploadId};
use crate::{
    FileResult,
    application::{
        BeginUpload, ByteRange, CompleteUpload, ObjectKey, ObjectRead, PartReceipt, ProviderPartReceipt, ProviderUploadRef, StoredObject, UploadPart,
        UploadSession,
    },
};

/// Stores opaque bytes while keeping naming, ownership, and policy in the file context.
///
/// Implementations must not interpret an object key as user-controlled filesystem input.
/// Uploads are completed explicitly so callers can persist metadata only after bytes are
/// durable. Implementations must clean staging data on abort and expose provider failures.
#[async_trait]
pub trait FileProvider: Send + Sync + 'static {
    fn provider_key(&self) -> crate::domain::ProviderKey;
    fn minimum_part_size(&self) -> crate::domain::ByteSize;
    async fn begin_upload(&self, request: BeginUpload) -> FileResult<UploadSession>;
    async fn write_part(&self, request: UploadPart) -> FileResult<ProviderPartReceipt>;
    async fn complete_upload(&self, request: CompleteUpload) -> FileResult<StoredObject>;
    async fn abort_upload(&self, provider_upload_ref: &ProviderUploadRef) -> FileResult<()>;
    async fn read_range(&self, key: &ObjectKey, range: Option<ByteRange>) -> FileResult<ObjectRead>;
    async fn delete(&self, key: &ObjectKey) -> FileResult<()>;
    async fn stat(&self, key: &ObjectKey) -> FileResult<StoredObject>;
    async fn capacity(&self) -> FileResult<ProviderCapacity>;
}

/// Metadata and authorization port for the File Management bounded context.
/// Implementations perform SQL transactions and never call a storage Provider.
#[async_trait]
pub trait FileManagementRepository: Send + Sync + 'static {
    async fn list_entries(&self, actor: &FileAccessScope, query: FileListQuery, page: kernel::pagination::CursorPageRequest) -> FileResult<FilePage>;
    async fn find_entry(&self, actor: &FileAccessScope, id: FileId) -> FileResult<Option<FileEntryView>>;
    async fn directory_trail(&self, actor: &FileAccessScope, directory_id: DirectoryId) -> FileResult<Vec<DirectoryTrailEntry>>;
    async fn overview(&self, actor: &FileAccessScope, space_id: Option<SpaceId>, default_quota: crate::domain::ByteSize) -> FileResult<FileOverviewView>;
    async fn list_spaces(
        &self,
        actor: &FileAccessScope,
        query: FileSpaceQuery,
        page: kernel::pagination::CursorPageRequest,
        default_quota: crate::domain::ByteSize,
    ) -> FileResult<kernel::pagination::CursorPage<FileSpaceView>>;
    async fn ensure_space(&self, owner_user_id: &str, owner_dept_id: Option<&str>) -> FileResult<SpaceId>;
    async fn ensure_target_space(&self, actor: &FileAccessScope, space_id: &SpaceId) -> FileResult<SpaceId>;
    async fn ensure_avatar_folder(&self, owner_user_id: &str, owner_dept_id: Option<&str>) -> FileResult<DirectoryId>;
    async fn create_folder(&self, actor: &FileAccessScope, command: CreateFolderCommand) -> FileResult<FileEntryView>;
    async fn update_entry(&self, actor: &FileAccessScope, command: UpdateEntryCommand) -> FileResult<FileEntryView>;
    async fn trash(&self, actor: &FileAccessScope, ids: &[FileId]) -> FileResult<()>;
    async fn restore(&self, actor: &FileAccessScope, ids: &[FileId]) -> FileResult<()>;
    async fn purge(&self, actor: &FileAccessScope, ids: &[FileId]) -> FileResult<crate::application::PurgeBatch>;
    async fn read_content(&self, actor: &FileAccessScope, request: FileReadRequest) -> FileResult<Option<(FileEntryView, ProviderKey, ObjectKey)>>;
    async fn find_reusable_object(
        &self,
        actor: &FileAccessScope,
        space_id: SpaceId,
        digest: crate::domain::ContentDigest,
        size: crate::domain::ByteSize,
    ) -> FileResult<Option<ExistingObject>>;
    async fn reserve_upload(&self, space_id: SpaceId, bytes: crate::domain::ByteSize, default_quota: crate::domain::ByteSize) -> FileResult<()>;
    async fn release_upload(&self, space_id: SpaceId, bytes: crate::domain::ByteSize) -> FileResult<()>;
    async fn create_uploaded_file(&self, actor: &FileAccessScope, command: UploadCommand, object: StoredObject) -> FileResult<FileEntryView>;
    async fn create_reused_file(&self, actor: &FileAccessScope, command: UploadCommand, object: ExistingObject) -> FileResult<FileEntryView>;
    async fn update_space(
        &self,
        actor: &FileAccessScope,
        space_id: SpaceId,
        command: UpdateSpaceCommand,
        default_quota: crate::domain::ByteSize,
    ) -> FileResult<FileSpaceView>;
    async fn claim_upload_cancellation(&self, owner_user_id: &str, session_id: UploadId) -> FileResult<String>;
    async fn cancel_upload(&self, owner_user_id: &str, session_id: UploadId, claim_token: &str) -> FileResult<()>;
    async fn expired_trash(&self, retention_days: u64, batch_size: u64) -> FileResult<TrashCleanupBatch>;
    async fn finalize_cleanup_object(&self, object_id: StoredObjectId) -> FileResult<()>;
    async fn record_provider_cleanup(
        &self,
        provider_key: &ProviderKey,
        kind: ProviderCleanupKind,
        object_key: Option<&ObjectKey>,
        upload_ref: Option<&ProviderUploadRef>,
    ) -> FileResult<()>;
    async fn claim_provider_cleanups(&self, batch_size: u64) -> FileResult<Vec<ProviderCleanupCandidate>>;
    async fn finalize_provider_cleanup(&self, cleanup_id: &str, claim_token: &str) -> FileResult<()>;
    async fn release_provider_cleanup(&self, cleanup_id: &str, claim_token: &str, error_code: &str) -> FileResult<()>;
    async fn expired_upload_sessions(&self, inactivity_days: u64, batch_size: u64) -> FileResult<Vec<UploadCleanupCandidate>>;
    async fn finalize_expired_upload(&self, session_id: UploadId, claim_token: &str) -> FileResult<()>;
    async fn release_upload_cleanup_claim(&self, session_id: UploadId, claim_token: &str) -> FileResult<()>;
    async fn create_upload_session(
        &self,
        actor: &FileAccessScope,
        command: BeginUploadSessionCommand,
        provider_session: UploadSession,
    ) -> FileResult<UploadSessionData>;
    async fn find_upload_intent(
        &self,
        actor: &FileAccessScope,
        owner_user_id: &str,
        space_id: SpaceId,
        idempotency_key: &str,
    ) -> FileResult<Option<(UploadSessionData, Vec<PartReceipt>)>>;
    async fn get_upload_session(&self, actor: &FileAccessScope, session_id: UploadId) -> FileResult<Option<(UploadSessionData, Vec<PartReceipt>)>>;
    async fn claim_upload_part(&self, actor: &FileAccessScope, request: UploadPartClaimRequest) -> FileResult<UploadPartClaimResult>;
    async fn complete_upload_part(&self, actor: &FileAccessScope, receipt: PartReceipt, claim_token: &str) -> FileResult<()>;
    async fn release_upload_part_claim(&self, session_id: UploadId, part_number: crate::domain::PartNumber, claim_token: &str) -> FileResult<()>;
    async fn begin_upload_completion(&self, actor: &FileAccessScope, session_id: UploadId) -> FileResult<(UploadSessionData, Vec<PartReceipt>)>;
    async fn reopen_upload_completion(&self, actor: &FileAccessScope, session_id: UploadId) -> FileResult<()>;
    async fn abort_upload_completion(
        &self,
        owner_user_id: &str,
        session_id: UploadId,
        object: StoredObject,
    ) -> FileResult<super::models::UploadCompletionTermination>;
    async fn abort_upload_completion_without_object(&self, owner_user_id: &str, session_id: UploadId)
    -> FileResult<super::models::UploadCompletionTermination>;
    async fn abort_claimed_upload_completion(
        &self,
        session_id: UploadId,
        claim_token: &str,
        object: StoredObject,
    ) -> FileResult<super::models::UploadCompletionTermination>;
    async fn finish_upload_session(&self, actor: &FileAccessScope, session_id: UploadId, object: StoredObject) -> FileResult<UploadCompletionResult>;
    async fn finish_claimed_upload_session(&self, session_id: UploadId, claim_token: &str, object: StoredObject) -> FileResult<UploadCompletionResult>;
    async fn create_reused_upload(
        &self,
        actor: &FileAccessScope,
        command: BeginUploadSessionCommand,
        object: ExistingObject,
        part_size: crate::domain::ByteSize,
    ) -> FileResult<FileEntryView>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileError;
    use crate::domain::ByteSize;

    #[test]
    fn object_keys_reject_path_escape_components() {
        for value in ["", "/absolute", "../escape", "a/../b", "a\\b"] {
            assert!(ObjectKey::new(value).is_err(), "{value:?} should be rejected");
        }
    }

    #[test]
    fn byte_ranges_are_half_open_and_bounded() {
        let range = ByteRange::new(2, 5).unwrap();
        assert_eq!(range.byte_len(), 3);
        assert!(range.within(ByteSize::from_bytes(5)).is_ok());
        assert!(range.within(ByteSize::from_bytes(4)).is_err());
    }

    #[test]
    fn provider_references_accept_non_uuid_native_values() {
        let upload_ref = ProviderUploadRef::new("oss-upload/native+opaque==").unwrap();
        let part_ref = crate::application::ProviderPartRef::new("etag:\"abc123\"").unwrap();

        assert_eq!(upload_ref.as_str(), "oss-upload/native+opaque==");
        assert_eq!(part_ref.as_str(), "etag:\"abc123\"");
        assert_eq!(
            ProviderUploadRef::new("  ").unwrap_err(),
            FileError::InvalidInput(crate::error::keys::PROVIDER_UPLOAD_REF_INVALID)
        );
        assert_eq!(
            crate::application::ProviderPartRef::new("\n").unwrap_err(),
            FileError::InvalidInput(crate::error::keys::PROVIDER_PART_REF_INVALID)
        );
    }
}
