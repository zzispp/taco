use sqlx::{query, query_as, query_scalar};

use super::{TestDatabase, insert_user, migrate};
use crate::application::{FileAccessScope, FileManagementRepository, ObjectKey, StoredObject, UploadCommand, UploadCompletionTermination};
use crate::domain::{ByteSize, ContentDigest, DirectoryId, EntryName, FileId, ProviderKey, SpaceId, StoredObjectId, UploadId};
use crate::infra::StorageFileRepository;

#[tokio::test]
async fn completed_upload_adopts_a_concurrently_created_canonical_object() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = repository(&database);
    let actor = FileAccessScope::self_only("actor", None);
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    let digest = ContentDigest::from_bytes(b"data");

    upload(
        &repository,
        &actor,
        UploadCase {
            space_id: space_id.clone(),
            name: "first.txt",
            key: "first-object",
            digest,
        },
    )
    .await;
    upload(
        &repository,
        &actor,
        UploadCase {
            space_id,
            name: "second.txt",
            key: "second-object",
            digest,
        },
    )
    .await;

    assert_adopted_canonical_object(database.pool()).await;
    database.drop().await;
}

#[tokio::test]
async fn reused_physical_object_key_is_never_queued_for_deletion() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = repository(&database);
    let actor = FileAccessScope::self_only("actor", None);
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    let digest = ContentDigest::from_bytes(b"data");

    upload(
        &repository,
        &actor,
        UploadCase {
            space_id: space_id.clone(),
            name: "first.txt",
            key: "shared-object",
            digest,
        },
    )
    .await;
    upload(
        &repository,
        &actor,
        UploadCase {
            space_id,
            name: "second.txt",
            key: "shared-object",
            digest,
        },
    )
    .await;

    let state: (i64, i64) = query_as(
        "SELECT (SELECT ref_count FROM file_object WHERE object_key='shared-object'),(SELECT COUNT(*) FROM file_provider_cleanup WHERE cleanup_kind='object' AND object_key='shared-object' AND status<>'done')",
    )
    .fetch_one(database.pool())
    .await
    .unwrap();
    assert_eq!(state, (2, 0));
    database.drop().await;
}

#[tokio::test]
async fn aborting_terminal_completion_releases_reserved_quota() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = repository(&database);
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    repository
        .reserve_upload(space_id.clone(), ByteSize::from_bytes(4), ByteSize::from_bytes(100))
        .await
        .unwrap();
    let session_id = UploadId::new();
    insert_completing_session(database.pool(), session_id, &space_id).await;

    let object = stored_object("object-ref", ContentDigest::from_bytes(b"data"));
    let outcome = repository.abort_upload_completion("actor", session_id, object).await.unwrap();

    let session: (String, i64, bool) = query_as("SELECT state,reserved_bytes,completed_at IS NOT NULL FROM file_upload_session WHERE session_id=$1")
        .bind(session_id.to_string())
        .fetch_one(database.pool())
        .await
        .unwrap();
    assert_eq!(session, ("aborted".into(), 0, true));
    assert_eq!(outcome, UploadCompletionTermination::Terminated);
    assert_eq!(
        query_scalar::<_, i64>("SELECT reserved_bytes FROM file_space WHERE space_id=$1")
            .bind(space_id.as_str())
            .fetch_one(database.pool())
            .await
            .unwrap(),
        0
    );
    database.drop().await;
}

#[tokio::test]
async fn terminal_abort_returns_a_concurrent_completion_without_queuing_cleanup() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = repository(&database);
    let actor = FileAccessScope::self_only("actor", None);
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    repository
        .reserve_upload(space_id.clone(), ByteSize::from_bytes(4), ByteSize::from_bytes(100))
        .await
        .unwrap();
    let session_id = UploadId::new();
    insert_completing_session(database.pool(), session_id, &space_id).await;
    let object = stored_object("object-ref", ContentDigest::from_bytes(b"data"));
    let completed = repository.finish_upload_session(&actor, session_id, object.clone()).await.unwrap();

    let outcome = repository.abort_upload_completion("actor", session_id, object).await.unwrap();

    assert_eq!(outcome, UploadCompletionTermination::Completed(FileId::parse(&completed.entry.id).unwrap()));
    assert_eq!(
        query_scalar::<_, i64>("SELECT COUNT(*) FROM file_provider_cleanup WHERE cleanup_kind='object' AND status<>'done'")
            .fetch_one(database.pool())
            .await
            .unwrap(),
        0
    );
    database.drop().await;
}

#[tokio::test]
async fn retrying_a_completed_session_returns_the_same_entry() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = repository(&database);
    let actor = FileAccessScope::self_only("actor", None);
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    repository
        .reserve_upload(space_id.clone(), ByteSize::from_bytes(4), ByteSize::from_bytes(100))
        .await
        .unwrap();
    let session_id = UploadId::new();
    insert_completing_session(database.pool(), session_id, &space_id).await;
    let object = stored_object("object-ref", ContentDigest::from_bytes(b"data"));

    let first = repository.finish_upload_session(&actor, session_id, object.clone()).await.unwrap();
    let retry = repository.finish_upload_session(&actor, session_id, object).await.unwrap();

    assert_eq!(retry.entry.id, first.entry.id);
    assert_eq!(
        query_scalar::<_, i64>("SELECT COUNT(*) FROM file_entry WHERE name='terminal.txt'")
            .fetch_one(database.pool())
            .await
            .unwrap(),
        1
    );
    database.drop().await;
}

#[tokio::test]
async fn deleting_digest_does_not_block_a_new_physical_object() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = repository(&database);
    let actor = FileAccessScope::self_only("actor", None);
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    let digest = ContentDigest::from_bytes(b"data");
    query("INSERT INTO file_object(object_id,provider_key,object_key,size_bytes,sha256,content_type,ref_count,status,created_at,updated_at) VALUES($1,'local','deleting-object',4,$2,'text/plain',0,'deleting',CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)")
        .bind(StoredObjectId::new().to_string())
        .bind(digest.to_hex())
        .execute(database.pool())
        .await
        .unwrap();
    repository
        .reserve_upload(space_id.clone(), ByteSize::from_bytes(4), ByteSize::from_bytes(100))
        .await
        .unwrap();

    let entry = repository
        .create_uploaded_file(&actor, upload_command(space_id, "replacement.txt"), stored_object("replacement-object", digest))
        .await
        .unwrap();

    assert_eq!(entry.name, "replacement.txt");
    assert_eq!(
        query_scalar::<_, i64>("SELECT COUNT(*) FROM file_object WHERE status='active' AND sha256=$1")
            .bind(digest.to_hex())
            .fetch_one(database.pool())
            .await
            .unwrap(),
        1
    );
    database.drop().await;
}

struct UploadCase<'a> {
    space_id: SpaceId,
    name: &'a str,
    key: &'a str,
    digest: ContentDigest,
}

async fn upload(repository: &StorageFileRepository, actor: &FileAccessScope, case: UploadCase<'_>) {
    repository
        .reserve_upload(case.space_id.clone(), ByteSize::from_bytes(4), ByteSize::from_bytes(100))
        .await
        .unwrap();
    repository
        .create_uploaded_file(actor, upload_command(case.space_id, case.name), stored_object(case.key, case.digest))
        .await
        .unwrap();
}

pub(super) async fn insert_completing_session(pool: &sqlx::PgPool, session_id: UploadId, space_id: &SpaceId) {
    insert_upload_session(pool, session_id, space_id, "completing").await;
}

pub(super) async fn insert_open_session(pool: &sqlx::PgPool, session_id: UploadId, space_id: &SpaceId) {
    insert_upload_session(pool, session_id, space_id, "open").await;
}

async fn insert_upload_session(pool: &sqlx::PgPool, session_id: UploadId, space_id: &SpaceId, state: &str) {
    query("INSERT INTO file_upload_session(session_id,owner_user_id,space_id,idempotency_key,file_name,normalized_name,declared_size_bytes,declared_sha256,content_type,part_size_bytes,provider_key,provider_upload_ref,provider_object_key,state,reserved_bytes,created_at,last_activity_at) VALUES($1,'actor',$2,$3,'terminal.txt','terminal.txt',4,$4,'text/plain',4,'local','upload-ref','object-ref',$5,4,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)")
        .bind(session_id.to_string()).bind(space_id.as_str()).bind(format!("terminal-intent-{session_id}")).bind(ContentDigest::from_bytes(b"data").to_hex()).bind(state).execute(pool).await.unwrap();
}

async fn assert_adopted_canonical_object(pool: &sqlx::PgPool) {
    assert_eq!(query_scalar::<_, i64>("SELECT COUNT(*) FROM file_object").fetch_one(pool).await.unwrap(), 1);
    assert_eq!(query_scalar::<_, i64>("SELECT ref_count FROM file_object").fetch_one(pool).await.unwrap(), 2);
    let pending_cleanup =
        query_scalar::<_, i64>("SELECT COUNT(*) FROM file_provider_cleanup WHERE cleanup_kind='object' AND object_key='second-object' AND status='pending'")
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(pending_cleanup, 1);
}

pub(super) fn repository(database: &TestDatabase) -> StorageFileRepository {
    StorageFileRepository::new(storage::Database::new(database.pool().clone()))
}

fn upload_command(space_id: SpaceId, name: &str) -> UploadCommand {
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
