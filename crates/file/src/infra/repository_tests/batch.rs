use sqlx::{PgPool, query, query_scalar};

use super::{TestDatabase, insert_user, migrate};
use crate::FileError;
use crate::application::{CreateFolderCommand, FileAccessScope, FileManagementRepository};
use crate::domain::{DirectoryId, EntryName, FileId, SpaceId, StoredObjectId, UploadId};
use crate::infra::StorageFileRepository;

#[tokio::test]
async fn batch_operations_deduplicate_parent_and_child_roots() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = StorageFileRepository::new(storage::Database::new(database.pool().clone()));
    let actor = FileAccessScope::self_only("actor", None);
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    let (parent, child) = create_folder_tree(&repository, &actor, space_id).await;
    let requested = vec![parent, child, parent];

    repository.trash(&actor, &requested).await.unwrap();
    assert_statuses(database.pool(), &[parent, child], "trashed").await;
    repository.restore(&actor, &requested).await.unwrap();
    assert_statuses(database.pool(), &[parent, child], "active").await;
    repository.trash(&actor, &requested).await.unwrap();

    let result = repository.purge(&actor, &requested).await.unwrap();

    assert_eq!(result.purged_entries, 2);
    assert_eq!(remaining_entries(database.pool()).await, 0);
    database.drop().await;
}

#[tokio::test]
async fn purge_preserves_referenced_tree_and_object_until_active_upload_is_removed() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = StorageFileRepository::new(storage::Database::new(database.pool().clone()));
    let actor = FileAccessScope::self_only("actor", None);
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    let (parent, child) = create_folder_tree(&repository, &actor, space_id.clone()).await;
    let object_id = insert_tree_file(database.pool(), &space_id, child).await;
    repository.trash(&actor, &[parent]).await.unwrap();
    insert_active_upload_target(database.pool(), &space_id, parent).await;

    let error = repository.purge(&actor, &[parent]).await.unwrap_err();
    assert_eq!(error, FileError::InvalidInput("errors.file.active_upload_target"));
    assert_statuses(database.pool(), &[parent, child], "trashed").await;
    assert_object_state(database.pool(), object_id, "active", 1).await;

    query("DELETE FROM file_upload_session WHERE parent_id=$1")
        .bind(parent.to_string())
        .execute(database.pool())
        .await
        .unwrap();
    let result = repository.purge(&actor, &[parent]).await.unwrap();

    assert_eq!(result.purged_entries, 3);
    assert_eq!(result.objects.len(), 1);
    assert_object_state(database.pool(), object_id, "deleting", 0).await;
    database.drop().await;
}

async fn create_folder_tree(repository: &StorageFileRepository, actor: &FileAccessScope, space_id: crate::domain::SpaceId) -> (FileId, FileId) {
    let parent = create_folder(repository, actor, space_id.clone(), DirectoryId::ROOT, "parent").await;
    let child = create_folder(repository, actor, space_id, DirectoryId::parse(&parent.to_string()).unwrap(), "child").await;
    (parent, child)
}

async fn create_folder(
    repository: &StorageFileRepository,
    actor: &FileAccessScope,
    space_id: crate::domain::SpaceId,
    parent_id: DirectoryId,
    name: &str,
) -> FileId {
    let entry = repository
        .create_folder(
            actor,
            CreateFolderCommand {
                space_id,
                parent_id,
                name: EntryName::new(name).unwrap(),
                actor_user_id: "actor".into(),
            },
        )
        .await
        .unwrap();
    FileId::parse(&entry.id).unwrap()
}

async fn assert_statuses(pool: &PgPool, ids: &[FileId], expected: &str) {
    for id in ids {
        let status = query_scalar::<_, String>("SELECT status FROM file_entry WHERE entry_id=$1")
            .bind(id.to_string())
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(status, expected);
    }
}

async fn remaining_entries(pool: &PgPool) -> i64 {
    query_scalar("SELECT COUNT(*) FROM file_entry").fetch_one(pool).await.unwrap()
}

async fn insert_tree_file(pool: &PgPool, space_id: &SpaceId, parent_id: FileId) -> StoredObjectId {
    let object_id = StoredObjectId::new();
    let entry_id = FileId::new();
    query("INSERT INTO file_object(object_id,provider_key,object_key,size_bytes,sha256,content_type,ref_count,status,created_at,updated_at) VALUES($1,'local','objects/batch-reference',4,repeat('f',64),'text/plain',1,'active',CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)")
        .bind(object_id.to_string())
        .execute(pool)
        .await
        .unwrap();
    query("INSERT INTO file_entry(entry_id,space_id,parent_id,kind,name,normalized_name,object_id,status,created_by,created_at,updated_at) VALUES($1,$2,$3,'file','Referenced.txt','referenced.txt',$4,'active','actor',CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)")
        .bind(entry_id.to_string())
        .bind(space_id.as_str())
        .bind(parent_id.to_string())
        .bind(object_id.to_string())
        .execute(pool)
        .await
        .unwrap();
    object_id
}

async fn insert_active_upload_target(pool: &PgPool, space_id: &SpaceId, parent_id: FileId) {
    let session_id = UploadId::new();
    query("INSERT INTO file_upload_session(session_id,owner_user_id,space_id,parent_id,idempotency_key,file_name,normalized_name,declared_size_bytes,declared_sha256,content_type,part_size_bytes,provider_key,provider_upload_ref,provider_object_key,state,reserved_bytes,created_at,last_activity_at) VALUES($1,'actor',$2,$3,$4,'pending.txt','pending.txt',4,repeat('a',64),'text/plain',4,'local','batch-upload-ref','objects/pending','open',4,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)")
        .bind(session_id.to_string())
        .bind(space_id.as_str())
        .bind(parent_id.to_string())
        .bind(format!("batch-upload-{session_id}"))
        .execute(pool)
        .await
        .unwrap();
}

async fn assert_object_state(pool: &PgPool, object_id: StoredObjectId, status: &str, ref_count: i64) {
    let actual: (String, i64) = sqlx::query_as("SELECT status,ref_count FROM file_object WHERE object_id=$1")
        .bind(object_id.to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(actual, (status.into(), ref_count));
}
