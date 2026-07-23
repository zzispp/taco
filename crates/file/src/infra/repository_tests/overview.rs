use sqlx::query;
use sqlx::query_scalar;

use super::{TestDatabase, insert_user, migrate};
use crate::application::{CreateFolderCommand, FileAccessScope, FileManagementRepository, ObjectKey, StoredObject, UploadCommand};
use crate::domain::{ByteSize, ContentDigest, DirectoryId, EntryName, ProviderKey, StoredObjectId, UploadId};
use crate::infra::StorageFileRepository;

#[tokio::test]
async fn overview_separates_reservations_temporary_bytes_and_deduplication() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = StorageFileRepository::new(storage::Database::new(database.pool().clone()));
    let actor = FileAccessScope::self_only("actor", None);
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    let digest = ContentDigest::from_bytes(b"data");
    for (name, key) in [("first.txt", "first-object"), ("second.txt", "second-object")] {
        repository
            .reserve_upload(space_id.clone(), ByteSize::from_bytes(4), ByteSize::from_bytes(100))
            .await
            .unwrap();
        repository
            .create_uploaded_file(&actor, upload_command(space_id.clone(), name), stored_object(key, digest))
            .await
            .unwrap();
    }
    insert_completed_temporary_part(database.pool(), space_id.as_str(), digest).await;

    let overview = repository.overview(&actor, Some(space_id), ByteSize::from_bytes(100)).await.unwrap();

    assert_eq!(overview.logical_asset_size, 8);
    assert_eq!(overview.quota_reserved_bytes, 4);
    assert_eq!(overview.temporary_upload_size, 4);
    assert_eq!(overview.managed_physical_usage, 8);
    assert_eq!(overview.deduplication_savings, 4);
    assert_eq!(overview.type_distribution.len(), 1);
    assert_eq!(overview.type_distribution[0].entry_type, "text");
    assert_eq!((overview.type_distribution[0].count, overview.type_distribution[0].bytes), (2, 8));
    database.drop().await;
}

#[tokio::test]
async fn overview_includes_a_recent_folder() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = StorageFileRepository::new(storage::Database::new(database.pool().clone()));
    let actor = FileAccessScope::self_only("actor", None);
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    let folder = repository
        .create_folder(
            &actor,
            CreateFolderCommand {
                space_id,
                parent_id: DirectoryId::ROOT,
                name: EntryName::new("Documents").unwrap(),
                actor_user_id: "actor".into(),
            },
        )
        .await
        .unwrap();

    let overview = repository.overview(&actor, None, ByteSize::from_bytes(100)).await.unwrap();
    assert_eq!(overview.recent_folders.len(), 1);
    assert_eq!(overview.recent_folders[0].id, folder.id);
    database.drop().await;
}

#[tokio::test]
async fn adopting_a_pending_cleanup_identity_cancels_that_cleanup_atomically() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = StorageFileRepository::new(storage::Database::new(database.pool().clone()));
    let actor = FileAccessScope::self_only("actor", None);
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    let key = ObjectKey::new("reused-key").unwrap();
    repository
        .record_provider_cleanup(&ProviderKey::local(), crate::application::ProviderCleanupKind::Object, Some(&key), None)
        .await
        .unwrap();
    repository
        .reserve_upload(space_id.clone(), ByteSize::from_bytes(4), ByteSize::from_bytes(100))
        .await
        .unwrap();

    repository
        .create_uploaded_file(
            &actor,
            upload_command(space_id, "reused.txt"),
            stored_object("reused-key", ContentDigest::from_bytes(b"data")),
        )
        .await
        .unwrap();

    let status: String = query_scalar("SELECT status FROM file_provider_cleanup WHERE object_key='reused-key'")
        .fetch_one(database.pool())
        .await
        .unwrap();
    assert_eq!(status, "done");
    database.drop().await;
}

#[tokio::test]
async fn provider_cleanup_does_not_claim_an_active_referenced_object() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = StorageFileRepository::new(storage::Database::new(database.pool().clone()));
    let actor = FileAccessScope::self_only("actor", None);
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    let key = ObjectKey::new("active-object").unwrap();
    repository
        .reserve_upload(space_id.clone(), ByteSize::from_bytes(4), ByteSize::from_bytes(100))
        .await
        .unwrap();
    repository
        .create_uploaded_file(
            &actor,
            upload_command(space_id, "active.txt"),
            stored_object("active-object", ContentDigest::from_bytes(b"data")),
        )
        .await
        .unwrap();
    repository
        .record_provider_cleanup(&ProviderKey::local(), crate::application::ProviderCleanupKind::Object, Some(&key), None)
        .await
        .unwrap();

    let claimed = repository.claim_provider_cleanups(10).await.unwrap();
    let status: String = query_scalar("SELECT status FROM file_provider_cleanup WHERE object_key='active-object'")
        .fetch_one(database.pool())
        .await
        .unwrap();

    assert!(claimed.is_empty());
    assert_eq!(status, "done");
    database.drop().await;
}

async fn insert_completed_temporary_part(pool: &sqlx::PgPool, space_id: &str, digest: ContentDigest) {
    let session_id = UploadId::new().to_string();
    query("UPDATE file_space SET reserved_bytes=reserved_bytes+4 WHERE space_id=$1")
        .bind(space_id)
        .execute(pool)
        .await
        .unwrap();
    query("INSERT INTO file_upload_session(session_id,owner_user_id,space_id,idempotency_key,file_name,normalized_name,declared_size_bytes,declared_sha256,content_type,part_size_bytes,provider_key,provider_upload_ref,provider_object_key,state,reserved_bytes,created_at,last_activity_at) VALUES($1,'actor',$2,'overview-intent','pending.txt','pending.txt',4,$3,'text/plain',4,'local',$1,$1,'open',4,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)")
        .bind(&session_id)
        .bind(space_id)
        .bind(digest.to_hex())
        .execute(pool)
        .await
        .unwrap();
    query("INSERT INTO file_upload_part(session_id,part_number,size_bytes,sha256,provider_part_ref,state,created_at) VALUES($1,1,4,$2,'part-1','completed',CURRENT_TIMESTAMP)")
        .bind(session_id)
        .bind(digest.to_hex())
        .execute(pool)
        .await
        .unwrap();
}

fn upload_command(space_id: crate::domain::SpaceId, name: &str) -> UploadCommand {
    UploadCommand {
        space_id,
        parent_id: DirectoryId::ROOT,
        name: EntryName::new(name).unwrap(),
        content_type: "text/plain".into(),
        bytes: bytes::Bytes::from_static(b"data"),
        actor_user_id: "actor".into(),
        idempotency_key: None,
    }
}

fn stored_object(key: &str, digest: ContentDigest) -> StoredObject {
    StoredObject {
        id: StoredObjectId::new(),
        provider_key: ProviderKey::local(),
        key: ObjectKey::new(key).unwrap(),
        size: ByteSize::from_bytes(4),
        digest: Some(digest),
    }
}
