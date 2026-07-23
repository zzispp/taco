use std::sync::Arc;

use async_trait::async_trait;
use sqlx::query_scalar;
use tokio::sync::Barrier;

use super::{TestDatabase, insert_user, migrate};
use crate::FileResult;
use crate::application::{
    BeginUpload, BeginUploadResult, BeginUploadSessionCommand, ByteRange, CompleteUpload, FileAccessScope, FileManagementConfig, FileManagementConfigProvider,
    FileManagementRepository, FileProvider, FileService, FileServiceDependencies, FileUseCase, ObjectKey, ObjectRead, ProviderPartReceipt, ProviderUploadRef,
    StoredObject, UploadPart, UploadSession,
};
use crate::domain::{ByteSize, ContentDigest, DirectoryId, EntryName, ProviderCapacity, ProviderKey};
use crate::infra::{BoundedImagePreviewProcessor, LocalFileProvider, StorageFileRepository};

#[tokio::test]
async fn concurrent_idempotency_key_returns_the_winning_upload_session() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = StorageFileRepository::new(storage::Database::new(database.pool().clone()));
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    let provider = Arc::new(BarrierProvider::new(tempfile::tempdir().unwrap(), Arc::new(Barrier::new(2))));
    let config = Arc::new(StaticConfig::default());
    let first = service(repository.clone(), provider.clone(), config.clone());
    let second = service(repository, provider, config);
    let command = upload_command(space_id);
    let actor = FileAccessScope::self_only("actor", None);

    let (first, second) = tokio::join!(
        first.begin_upload_session(actor.clone(), command.clone()),
        second.begin_upload_session(actor, command)
    );

    assert_eq!(upload_session_id(first), upload_session_id(second));
    assert_eq!(
        query_scalar::<_, i64>("SELECT COUNT(*) FROM file_upload_session")
            .fetch_one(database.pool())
            .await
            .unwrap(),
        1
    );
    assert_eq!(
        query_scalar::<_, i64>("SELECT reserved_bytes FROM file_space WHERE space_id='actor'")
            .fetch_one(database.pool())
            .await
            .unwrap(),
        4
    );
    database.drop().await;
}

#[derive(Clone, Copy)]
struct StaticConfig(FileManagementConfig);

impl Default for StaticConfig {
    fn default() -> Self {
        Self(FileManagementConfig {
            max_file_bytes: ByteSize::from_bytes(100),
            default_space_quota_bytes: ByteSize::from_bytes(100),
            upload_part_bytes: ByteSize::from_bytes(4),
            upload_session_inactivity_days: 1,
        })
    }
}

#[async_trait]
impl FileManagementConfigProvider for StaticConfig {
    async fn file_management_config(&self) -> FileResult<FileManagementConfig> {
        Ok(self.0)
    }
}

struct BarrierProvider {
    _directory: tempfile::TempDir,
    inner: LocalFileProvider,
    barrier: Arc<Barrier>,
}

impl BarrierProvider {
    fn new(directory: tempfile::TempDir, barrier: Arc<Barrier>) -> Self {
        let inner = LocalFileProvider::new(directory.path()).unwrap();
        Self {
            _directory: directory,
            inner,
            barrier,
        }
    }
}

#[async_trait]
impl FileProvider for BarrierProvider {
    fn provider_key(&self) -> ProviderKey {
        self.inner.provider_key()
    }

    fn minimum_part_size(&self) -> ByteSize {
        self.inner.minimum_part_size()
    }

    async fn begin_upload(&self, request: BeginUpload) -> FileResult<UploadSession> {
        let session = self.inner.begin_upload(request).await?;
        self.barrier.wait().await;
        Ok(session)
    }

    async fn write_part(&self, request: UploadPart) -> FileResult<ProviderPartReceipt> {
        self.inner.write_part(request).await
    }

    async fn complete_upload(&self, request: CompleteUpload) -> FileResult<StoredObject> {
        self.inner.complete_upload(request).await
    }

    async fn abort_upload(&self, upload_ref: &ProviderUploadRef) -> FileResult<()> {
        self.inner.abort_upload(upload_ref).await
    }

    async fn read_range(&self, key: &ObjectKey, range: Option<ByteRange>) -> FileResult<ObjectRead> {
        self.inner.read_range(key, range).await
    }

    async fn delete(&self, key: &ObjectKey) -> FileResult<()> {
        self.inner.delete(key).await
    }

    async fn stat(&self, key: &ObjectKey) -> FileResult<StoredObject> {
        self.inner.stat(key).await
    }

    async fn capacity(&self) -> FileResult<ProviderCapacity> {
        self.inner.capacity().await
    }
}

fn service(repository: StorageFileRepository, provider: Arc<BarrierProvider>, config: Arc<StaticConfig>) -> Arc<dyn FileUseCase> {
    Arc::new(FileService::new(
        repository,
        FileServiceDependencies {
            provider,
            config,
            image_previews: Arc::new(BoundedImagePreviewProcessor),
        },
    ))
}

fn upload_command(space_id: crate::domain::SpaceId) -> BeginUploadSessionCommand {
    BeginUploadSessionCommand {
        space_id,
        parent_id: DirectoryId::ROOT,
        name: EntryName::new("race.txt").unwrap(),
        size: ByteSize::from_bytes(4),
        digest: ContentDigest::from_bytes(b"data"),
        content_type: "text/plain".into(),
        actor_user_id: "actor".into(),
        idempotency_key: "race-key".into(),
    }
}

fn upload_session_id(result: FileResult<BeginUploadResult>) -> String {
    match result.unwrap() {
        BeginUploadResult::UploadRequired { session } => session.id,
        BeginUploadResult::Completed { .. } => panic!("new upload must create a resumable session"),
    }
}
