use crate::application::{FileListQuery, FileManagementRepository};
use crate::domain::{ByteSize, DirectoryId, SpaceId};

use super::*;

const PAGE_LIMIT: u64 = 1;
const DEFAULT_QUOTA: ByteSize = ByteSize::from_bytes(20);

#[tokio::test]
async fn file_entry_pages_return_to_the_previous_batch() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_department(database.pool(), "dept-1", "0").await;
    insert_user(database.pool(), "actor", Some("dept-1"), "Actor").await;
    insert_space(database.pool(), "actor").await;
    for (id, name) in [("entry-a", "Alpha"), ("entry-b", "Beta"), ("entry-c", "Gamma")] {
        insert_folder(database.pool(), id, name).await;
    }
    let repository = repository(&database);
    let scope = FileAccessScope::all("actor");

    let first = repository.list_entries(&scope, entry_query(None), page_request(None)).await.unwrap();
    let second = repository
        .list_entries(&scope, entry_query(first.next_cursor.clone()), page_request(first.next_cursor.clone()))
        .await
        .unwrap();
    let first_ids = entry_ids(&first.items);
    let second_ids = entry_ids(&second.items);
    let previous_cursor = second.previous_cursor.clone();
    let returned = match previous_cursor.clone() {
        Some(cursor) => repository
            .list_entries(&scope, entry_query(Some(cursor.clone())), page_request(Some(cursor)))
            .await
            .unwrap(),
        None => kernel::pagination::CursorPage::new(Vec::new(), None, None),
    };
    let actual = entry_ids(&returned.items);
    let expected = entry_ids(&first.items);
    database.drop().await;

    assert_eq!(first_ids, vec!["entry-a"]);
    assert_eq!(second_ids, vec!["entry-b"]);
    assert!(previous_cursor.is_some(), "the second entry batch must expose a previous cursor");
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn file_space_pages_return_to_the_previous_batch() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_department(database.pool(), "dept-1", "0").await;
    for (id, name) in [("actor", "Alpha"), ("peer-b", "Beta"), ("peer-c", "Gamma")] {
        insert_user(database.pool(), id, Some("dept-1"), name).await;
    }
    let repository = repository(&database);
    let scope = FileAccessScope::all("actor");

    let first = repository
        .list_spaces(&scope, space_query(None), page_request(None), DEFAULT_QUOTA)
        .await
        .unwrap();
    let second = repository
        .list_spaces(
            &scope,
            space_query(first.next_cursor.clone()),
            page_request(first.next_cursor.clone()),
            DEFAULT_QUOTA,
        )
        .await
        .unwrap();
    let first_ids = space_ids(&first.items);
    let second_ids = space_ids(&second.items);
    let previous_cursor = second.previous_cursor.clone();
    let returned = match previous_cursor.clone() {
        Some(cursor) => repository
            .list_spaces(&scope, space_query(Some(cursor.clone())), page_request(Some(cursor)), DEFAULT_QUOTA)
            .await
            .unwrap(),
        None => kernel::pagination::CursorPage::new(Vec::new(), None, None),
    };
    let actual = space_ids(&returned.items);
    let expected = space_ids(&first.items);
    database.drop().await;

    assert_eq!(first_ids, vec!["actor"]);
    assert_eq!(second_ids, vec!["peer-b"]);
    assert!(previous_cursor.is_some(), "the second space batch must expose a previous cursor");
    assert_eq!(actual, expected);
}

#[tokio::test]
async fn drifted_entry_boundary_returns_an_opposite_recovery_cursor() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_department(database.pool(), "dept-1", "0").await;
    insert_user(database.pool(), "actor", Some("dept-1"), "Actor").await;
    insert_space(database.pool(), "actor").await;
    for (id, name) in [("entry-a", "Alpha"), ("entry-b", "Beta"), ("entry-c", "Gamma")] {
        insert_folder(database.pool(), id, name).await;
    }
    let repository = repository(&database);
    let scope = FileAccessScope::all("actor");
    let first = repository.list_entries(&scope, entry_query(None), page_request(None)).await.unwrap();
    query("UPDATE file_entry SET updated_at=TIMESTAMPTZ '2025-01-01 00:00:00Z'")
        .execute(database.pool())
        .await
        .unwrap();

    let stale = repository
        .list_entries(&scope, entry_query(first.next_cursor.clone()), page_request(first.next_cursor))
        .await
        .unwrap();
    let recovery = stale.previous_cursor.clone();
    let returned = match recovery.clone() {
        Some(cursor) => repository
            .list_entries(&scope, entry_query(Some(cursor.clone())), page_request(Some(cursor)))
            .await
            .unwrap(),
        None => kernel::pagination::CursorPage::new(Vec::new(), None, None),
    };
    let returned_ids = entry_ids(&returned.items);
    database.drop().await;

    assert_eq!(stale.items, Vec::new());
    assert_eq!(stale.next_cursor, None);
    assert!(recovery.is_some(), "an empty forward entry page must remain reversible");
    assert_eq!(returned_ids, vec!["entry-c"]);
}

#[tokio::test]
async fn drifted_space_boundary_returns_an_opposite_recovery_cursor() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_department(database.pool(), "dept-1", "0").await;
    for (id, name) in [("actor", "Alpha"), ("peer-b", "Beta"), ("peer-c", "Gamma")] {
        insert_user(database.pool(), id, Some("dept-1"), name).await;
    }
    insert_space(database.pool(), "actor").await;
    let repository = repository(&database);
    let scope = FileAccessScope::all("actor");
    let first = repository
        .list_spaces(&scope, updated_space_query(None), page_request(None), DEFAULT_QUOTA)
        .await
        .unwrap();
    move_space_positions_forward(database.pool()).await;

    let stale = repository
        .list_spaces(
            &scope,
            updated_space_query(first.next_cursor.clone()),
            page_request(first.next_cursor),
            DEFAULT_QUOTA,
        )
        .await
        .unwrap();
    let recovery = stale.previous_cursor.clone();
    let returned = match recovery.clone() {
        Some(cursor) => repository
            .list_spaces(&scope, updated_space_query(Some(cursor.clone())), page_request(Some(cursor)), DEFAULT_QUOTA)
            .await
            .unwrap(),
        None => kernel::pagination::CursorPage::new(Vec::new(), None, None),
    };
    let returned_ids = space_ids(&returned.items);
    database.drop().await;

    assert_eq!(stale.items, Vec::new());
    assert_eq!(stale.next_cursor, None);
    assert!(recovery.is_some(), "an empty forward space page must remain reversible");
    assert_eq!(returned_ids, vec!["actor"]);
}

fn repository(database: &TestDatabase) -> StorageFileRepository {
    StorageFileRepository::new(storage::Database::new(database.pool().clone()))
}

fn page_request(cursor: Option<String>) -> CursorPageRequest {
    CursorPageRequest { limit: PAGE_LIMIT, cursor }
}

fn entry_query(cursor: Option<String>) -> FileListQuery {
    FileListQuery {
        cursor,
        space_id: Some(SpaceId::new("actor").unwrap()),
        parent_id: Some(DirectoryId::ROOT),
        sort_by: Some("updated_at".into()),
        sort_order: Some("asc".into()),
        ..FileListQuery::default()
    }
}

fn space_query(cursor: Option<String>) -> FileSpaceQuery {
    FileSpaceQuery {
        cursor,
        sort_by: Some("status".into()),
        sort_order: Some("asc".into()),
        ..FileSpaceQuery::default()
    }
}

fn updated_space_query(cursor: Option<String>) -> FileSpaceQuery {
    FileSpaceQuery {
        cursor,
        sort_by: Some("updated_at".into()),
        sort_order: Some("desc".into()),
        ..FileSpaceQuery::default()
    }
}

async fn insert_space(pool: &PgPool, owner: &str) {
    query("INSERT INTO file_space(space_id,owner_user_id,owner_dept_id,created_at,updated_at) VALUES($1,$1,'dept-1',CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)")
        .bind(owner)
        .execute(pool)
        .await
        .unwrap();
}

async fn insert_folder(pool: &PgPool, id: &str, name: &str) {
    query("INSERT INTO file_entry(entry_id,space_id,kind,name,normalized_name,status,created_by,created_at,updated_by,updated_at) VALUES($1,'actor','folder',$2,LOWER($2),'active','actor',TIMESTAMPTZ '2026-01-01 00:00:00Z','actor',TIMESTAMPTZ '2026-01-01 00:00:00Z')")
        .bind(id)
        .bind(name)
        .execute(pool)
        .await
        .unwrap();
}

async fn move_space_positions_forward(pool: &PgPool) {
    query("UPDATE sys_user SET update_time=TIMESTAMPTZ '2027-01-01 00:00:00Z'")
        .execute(pool)
        .await
        .unwrap();
    query("UPDATE file_space SET updated_at=TIMESTAMPTZ '2027-01-01 00:00:00Z'")
        .execute(pool)
        .await
        .unwrap();
}

fn entry_ids(items: &[crate::application::FileEntryView]) -> Vec<&str> {
    items.iter().map(|item| item.id.as_str()).collect()
}

fn space_ids(items: &[crate::application::FileSpaceView]) -> Vec<&str> {
    items.iter().map(|item| item.id.as_str()).collect()
}
