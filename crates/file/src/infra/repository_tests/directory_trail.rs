use sqlx::query;

use super::{TestDatabase, insert_user, migrate};
use crate::application::{CreateFolderCommand, DirectoryTrailEntry, FileAccessScope, FileManagementRepository};
use crate::domain::{DirectoryId, EntryName, SpaceId};
use crate::infra::StorageFileRepository;

#[tokio::test]
async fn directory_trail_returns_root_ordered_active_ancestors_and_honors_data_scope() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    insert_user(database.pool(), "outside", None, "Outside").await;
    let repository = StorageFileRepository::new(storage::Database::new(database.pool().clone()));
    let actor = FileAccessScope::self_only("actor", None);
    let actor_space = repository.ensure_space("actor", None).await.unwrap();
    let first = create_folder(&repository, &actor, actor_space.clone(), DirectoryId::ROOT, "First").await;
    let second = create_folder(&repository, &actor, actor_space, directory_id(&first.id), "Second").await;
    let third = create_folder(&repository, &actor, SpaceId::new("actor").unwrap(), directory_id(&second.id), "Third").await;

    let trail = repository.directory_trail(&actor, directory_id(&third.id)).await.unwrap();

    assert_eq!(
        trail,
        vec![
            DirectoryTrailEntry {
                id: first.id.clone(),
                parent_id: None,
                name: "First".into(),
            },
            DirectoryTrailEntry {
                id: second.id.clone(),
                parent_id: Some(first.id.clone()),
                name: "Second".into(),
            },
            DirectoryTrailEntry {
                id: third.id.clone(),
                parent_id: Some(second.id.clone()),
                name: "Third".into(),
            },
        ]
    );

    let outside_space = repository.ensure_space("outside", None).await.unwrap();
    let outside = create_folder(
        &repository,
        &FileAccessScope::self_only("outside", None),
        outside_space,
        DirectoryId::ROOT,
        "Outside",
    )
    .await;
    assert!(repository.directory_trail(&actor, directory_id(&outside.id)).await.unwrap().is_empty());
    database.drop().await;
}

#[tokio::test]
async fn directory_trail_rejects_incomplete_and_cyclic_parent_chains() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = StorageFileRepository::new(storage::Database::new(database.pool().clone()));
    let actor = FileAccessScope::self_only("actor", None);
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    let inactive_parent = create_folder(&repository, &actor, space_id.clone(), DirectoryId::ROOT, "Inactive").await;
    let orphan = create_folder(&repository, &actor, space_id.clone(), directory_id(&inactive_parent.id), "Orphan").await;
    query("UPDATE file_entry SET status='trashed',trashed_at=CURRENT_TIMESTAMP WHERE entry_id=$1")
        .bind(&inactive_parent.id)
        .execute(database.pool())
        .await
        .unwrap();
    assert!(repository.directory_trail(&actor, directory_id(&orphan.id)).await.unwrap().is_empty());

    let cycle_root = create_folder(&repository, &actor, space_id.clone(), DirectoryId::ROOT, "Cycle root").await;
    let cycle_child = create_folder(&repository, &actor, space_id, directory_id(&cycle_root.id), "Cycle child").await;
    query("UPDATE file_entry SET parent_id=$1 WHERE entry_id=$2")
        .bind(&cycle_child.id)
        .bind(&cycle_root.id)
        .execute(database.pool())
        .await
        .unwrap();
    assert!(repository.directory_trail(&actor, directory_id(&cycle_child.id)).await.unwrap().is_empty());
    database.drop().await;
}

async fn create_folder(
    repository: &StorageFileRepository,
    actor: &FileAccessScope,
    space_id: SpaceId,
    parent_id: DirectoryId,
    name: &str,
) -> crate::application::FileEntryView {
    repository
        .create_folder(
            actor,
            CreateFolderCommand {
                space_id,
                parent_id,
                name: EntryName::new(name).unwrap(),
                actor_user_id: actor.user_id.clone(),
            },
        )
        .await
        .unwrap()
}

fn directory_id(id: &str) -> DirectoryId {
    DirectoryId::parse(id).unwrap()
}
