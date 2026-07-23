use async_trait::async_trait;
use storage::Database;

use crate::FileResult;
use crate::application::ObjectKey;
use crate::application::{
    CreateFolderCommand, DirectoryTrailEntry, ExistingObject, FileAccessScope, FileEntryView, FileListQuery, FileManagementRepository, FileOverviewView,
    FilePage, FileReadRequest, FileSpaceQuery, FileSpaceView, StoredObject, UpdateEntryCommand, UpdateSpaceCommand, UploadCommand, UploadCompletionResult,
};
use crate::domain::{ContentDigest, DirectoryId, FileId, ProviderKey, SpaceId, StoredObjectId, UploadId};

use super::{
    repository_cleanup, repository_commands, repository_directory_trail, repository_provider_cleanup, repository_queries, repository_session_lifecycle,
    repository_sessions, repository_system_folders, repository_uploads,
};

#[derive(Clone)]
pub struct StorageFileRepository {
    database: Database,
}

impl StorageFileRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub fn database(&self) -> &Database {
        &self.database
    }
}

#[async_trait]
impl FileManagementRepository for StorageFileRepository {
    async fn list_entries(&self, actor: &FileAccessScope, query: FileListQuery, page: kernel::pagination::CursorPageRequest) -> FileResult<FilePage> {
        repository_queries::list_entries(&self.database, actor, query, page).await
    }

    async fn find_entry(&self, actor: &FileAccessScope, id: FileId) -> FileResult<Option<FileEntryView>> {
        repository_queries::find_entry(&self.database, actor, id).await
    }

    async fn directory_trail(&self, actor: &FileAccessScope, directory_id: DirectoryId) -> FileResult<Vec<DirectoryTrailEntry>> {
        repository_directory_trail::directory_trail(&self.database, actor, directory_id).await
    }

    async fn overview(&self, actor: &FileAccessScope, space_id: Option<SpaceId>, default_quota: crate::domain::ByteSize) -> FileResult<FileOverviewView> {
        repository_queries::overview(&self.database, actor, space_id, default_quota).await
    }

    async fn list_spaces(
        &self,
        actor: &FileAccessScope,
        query: FileSpaceQuery,
        page: kernel::pagination::CursorPageRequest,
        default_quota: crate::domain::ByteSize,
    ) -> FileResult<kernel::pagination::CursorPage<FileSpaceView>> {
        repository_queries::list_spaces(&self.database, actor, query, page, default_quota).await
    }

    async fn ensure_space(&self, owner_user_id: &str, owner_dept_id: Option<&str>) -> FileResult<SpaceId> {
        repository_system_folders::ensure_space(&self.database, owner_user_id, owner_dept_id).await
    }

    async fn ensure_target_space(&self, actor: &FileAccessScope, space_id: &SpaceId) -> FileResult<SpaceId> {
        repository_commands::ensure_target_space(&self.database, actor, space_id).await
    }

    async fn ensure_avatar_folder(&self, owner_user_id: &str, owner_dept_id: Option<&str>) -> FileResult<DirectoryId> {
        repository_system_folders::ensure_avatar_folder(&self.database, owner_user_id, owner_dept_id).await
    }

    async fn create_folder(&self, actor: &FileAccessScope, command: CreateFolderCommand) -> FileResult<FileEntryView> {
        repository_commands::create_folder(&self.database, actor, command).await
    }

    async fn update_entry(&self, actor: &FileAccessScope, command: UpdateEntryCommand) -> FileResult<FileEntryView> {
        repository_commands::update_entry(&self.database, actor, command).await
    }

    async fn trash(&self, actor: &FileAccessScope, ids: &[FileId]) -> FileResult<()> {
        repository_commands::trash(&self.database, actor, ids, time::OffsetDateTime::now_utc()).await
    }

    async fn restore(&self, actor: &FileAccessScope, ids: &[FileId]) -> FileResult<()> {
        repository_commands::restore(&self.database, actor, ids, time::OffsetDateTime::now_utc()).await
    }

    async fn purge(&self, actor: &FileAccessScope, ids: &[FileId]) -> FileResult<crate::application::PurgeBatch> {
        repository_commands::purge(&self.database, actor, ids).await
    }

    async fn read_content(&self, actor: &FileAccessScope, request: FileReadRequest) -> FileResult<Option<(FileEntryView, ProviderKey, ObjectKey)>> {
        repository_queries::read_content(&self.database, actor, request).await
    }

    async fn find_reusable_object(
        &self,
        actor: &FileAccessScope,
        space_id: SpaceId,
        digest: ContentDigest,
        size: crate::domain::ByteSize,
    ) -> FileResult<Option<ExistingObject>> {
        repository_queries::find_reusable_object(&self.database, actor, space_id, digest, size).await
    }

    async fn reserve_upload(&self, space_id: SpaceId, bytes: crate::domain::ByteSize, default_quota: crate::domain::ByteSize) -> FileResult<()> {
        repository_uploads::reserve_upload(&self.database, space_id, bytes, default_quota).await
    }

    async fn release_upload(&self, space_id: SpaceId, bytes: crate::domain::ByteSize) -> FileResult<()> {
        repository_uploads::release_upload(&self.database, space_id, bytes).await
    }

    async fn create_uploaded_file(&self, actor: &FileAccessScope, command: UploadCommand, object: StoredObject) -> FileResult<FileEntryView> {
        repository_uploads::create_uploaded_file(&self.database, actor, command, object).await
    }

    async fn create_reused_file(&self, actor: &FileAccessScope, command: UploadCommand, object: ExistingObject) -> FileResult<FileEntryView> {
        repository_uploads::create_reused_file(&self.database, actor, command, object).await
    }

    async fn update_space(
        &self,
        actor: &FileAccessScope,
        space_id: SpaceId,
        command: UpdateSpaceCommand,
        default_quota: crate::domain::ByteSize,
    ) -> FileResult<FileSpaceView> {
        repository_commands::update_space(&self.database, actor, space_id, command, default_quota).await
    }

    async fn claim_upload_cancellation(&self, owner_user_id: &str, session_id: UploadId) -> FileResult<String> {
        repository_session_lifecycle::claim_upload_cancellation(&self.database, owner_user_id, session_id).await
    }

    async fn cancel_upload(&self, owner_user_id: &str, session_id: UploadId, claim_token: &str) -> FileResult<()> {
        repository_session_lifecycle::cancel_upload(&self.database, owner_user_id, session_id, claim_token).await
    }

    async fn expired_trash(&self, retention_days: u64, batch_size: u64) -> FileResult<crate::application::TrashCleanupBatch> {
        repository_cleanup::expired_trash(&self.database, retention_days, batch_size).await
    }

    async fn finalize_cleanup_object(&self, object_id: StoredObjectId) -> FileResult<()> {
        repository_cleanup::finalize_cleanup_object(&self.database, object_id).await
    }

    async fn record_provider_cleanup(
        &self,
        provider_key: &ProviderKey,
        kind: crate::application::ProviderCleanupKind,
        object_key: Option<&ObjectKey>,
        upload_ref: Option<&crate::application::ProviderUploadRef>,
    ) -> FileResult<()> {
        repository_provider_cleanup::record(&self.database, provider_key, kind, object_key, upload_ref).await
    }

    async fn claim_provider_cleanups(&self, batch_size: u64) -> FileResult<Vec<crate::application::ProviderCleanupCandidate>> {
        repository_provider_cleanup::claim(&self.database, batch_size).await
    }

    async fn finalize_provider_cleanup(&self, cleanup_id: &str, claim_token: &str) -> FileResult<()> {
        repository_provider_cleanup::finalize(&self.database, cleanup_id, claim_token).await
    }

    async fn release_provider_cleanup(&self, cleanup_id: &str, claim_token: &str, error_code: &str) -> FileResult<()> {
        repository_provider_cleanup::release(&self.database, cleanup_id, claim_token, error_code).await
    }

    async fn expired_upload_sessions(&self, inactivity_days: u64, batch_size: u64) -> FileResult<Vec<crate::application::UploadCleanupCandidate>> {
        repository_cleanup::expired_upload_sessions(&self.database, inactivity_days, batch_size).await
    }

    async fn finalize_expired_upload(&self, session_id: UploadId, claim_token: &str) -> FileResult<()> {
        repository_cleanup::finalize_expired_upload(&self.database, session_id, claim_token).await
    }

    async fn release_upload_cleanup_claim(&self, session_id: UploadId, claim_token: &str) -> FileResult<()> {
        repository_cleanup::release_upload_cleanup_claim(&self.database, session_id, claim_token).await
    }

    async fn create_upload_session(
        &self,
        actor: &FileAccessScope,
        command: crate::application::BeginUploadSessionCommand,
        provider_session: crate::application::UploadSession,
    ) -> FileResult<crate::application::UploadSessionData> {
        repository_sessions::create_upload_session(&self.database, actor, command, provider_session).await
    }

    async fn find_upload_intent(
        &self,
        actor: &FileAccessScope,
        owner_user_id: &str,
        space_id: SpaceId,
        idempotency_key: &str,
    ) -> FileResult<Option<(crate::application::UploadSessionData, Vec<crate::application::PartReceipt>)>> {
        repository_sessions::find_upload_intent(&self.database, actor, owner_user_id, space_id, idempotency_key).await
    }

    async fn get_upload_session(
        &self,
        actor: &FileAccessScope,
        session_id: UploadId,
    ) -> FileResult<Option<(crate::application::UploadSessionData, Vec<crate::application::PartReceipt>)>> {
        repository_sessions::get_upload_session(&self.database, actor, session_id).await
    }

    async fn claim_upload_part(
        &self,
        actor: &FileAccessScope,
        request: crate::application::UploadPartClaimRequest,
    ) -> FileResult<crate::application::UploadPartClaimResult> {
        repository_sessions::claim_upload_part(&self.database, actor, request).await
    }

    async fn complete_upload_part(&self, actor: &FileAccessScope, receipt: crate::application::PartReceipt, claim_token: &str) -> FileResult<()> {
        repository_sessions::complete_upload_part(&self.database, actor, receipt, claim_token).await
    }

    async fn release_upload_part_claim(&self, session_id: UploadId, part_number: crate::domain::PartNumber, claim_token: &str) -> FileResult<()> {
        repository_sessions::release_upload_part_claim(&self.database, session_id, part_number, claim_token).await
    }

    async fn begin_upload_completion(
        &self,
        actor: &FileAccessScope,
        session_id: UploadId,
    ) -> FileResult<(crate::application::UploadSessionData, Vec<crate::application::PartReceipt>)> {
        repository_sessions::begin_upload_completion(&self.database, actor, session_id).await
    }

    async fn reopen_upload_completion(&self, actor: &FileAccessScope, session_id: UploadId) -> FileResult<()> {
        repository_sessions::reopen_upload_completion(&self.database, actor, session_id).await
    }

    async fn abort_upload_completion(
        &self,
        owner_user_id: &str,
        session_id: UploadId,
        object: StoredObject,
    ) -> FileResult<crate::application::UploadCompletionTermination> {
        repository_session_lifecycle::abort_upload_completion(&self.database, owner_user_id, session_id, object).await
    }

    async fn abort_upload_completion_without_object(
        &self,
        owner_user_id: &str,
        session_id: UploadId,
    ) -> FileResult<crate::application::UploadCompletionTermination> {
        repository_session_lifecycle::abort_upload_completion_without_object(&self.database, owner_user_id, session_id).await
    }

    async fn abort_claimed_upload_completion(
        &self,
        session_id: UploadId,
        claim_token: &str,
        object: StoredObject,
    ) -> FileResult<crate::application::UploadCompletionTermination> {
        repository_session_lifecycle::abort_claimed_upload_completion(&self.database, session_id, claim_token, object).await
    }

    async fn finish_upload_session(&self, actor: &FileAccessScope, session_id: UploadId, object: StoredObject) -> FileResult<UploadCompletionResult> {
        repository_sessions::finish_upload_session(&self.database, actor, session_id, object).await
    }

    async fn finish_claimed_upload_session(&self, session_id: UploadId, claim_token: &str, object: StoredObject) -> FileResult<UploadCompletionResult> {
        repository_sessions::finish_claimed_upload_session(&self.database, session_id, claim_token, object).await
    }

    async fn create_reused_upload(
        &self,
        actor: &FileAccessScope,
        command: crate::application::BeginUploadSessionCommand,
        object: ExistingObject,
        part_size: crate::domain::ByteSize,
    ) -> FileResult<FileEntryView> {
        repository_sessions::create_reused_upload(&self.database, actor, command, object, part_size).await
    }
}

#[cfg(test)]
#[path = "repository_tests.rs"]
mod tests;
