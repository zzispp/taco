use sqlx::{PgPool, query_scalar};

use super::{TestDatabase, insert_user, migrate};
use crate::application::{CreateFolderCommand, FileAccessScope, FileManagementRepository};
use crate::domain::{DirectoryId, EntryName, FileId};
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
