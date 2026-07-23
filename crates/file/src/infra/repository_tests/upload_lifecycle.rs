use sqlx::query_as;

use super::upload::{insert_completing_session, insert_open_session, repository};
use super::{TestDatabase, insert_user, migrate};
use crate::application::{FileManagementRepository, UploadCompletionTermination};
use crate::domain::{ByteSize, UploadId};

#[tokio::test]
async fn terminal_completion_without_object_queues_staging_cleanup_with_quota_release() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = repository(&database);
    let space_id = reserve_upload(&repository).await;
    let session_id = UploadId::new();
    insert_completing_session(database.pool(), session_id, &space_id).await;

    let outcome = repository.abort_upload_completion_without_object("actor", session_id).await.unwrap();

    assert_eq!(outcome, UploadCompletionTermination::Terminated);
    assert_session_is_terminal(database.pool(), session_id).await;
    assert_upload_cleanup(database.pool()).await;
    database.drop().await;
}

#[tokio::test]
async fn canceling_upload_queues_staging_cleanup_with_quota_release() {
    let database = TestDatabase::create().await;
    migrate(database.pool()).await;
    insert_user(database.pool(), "actor", None, "Actor").await;
    let repository = repository(&database);
    let space_id = reserve_upload(&repository).await;
    let session_id = UploadId::new();
    insert_open_session(database.pool(), session_id, &space_id).await;
    let claim_token = repository.claim_upload_cancellation("actor", session_id).await.unwrap();

    repository.cancel_upload("actor", session_id, &claim_token).await.unwrap();

    assert_session_is_terminal(database.pool(), session_id).await;
    assert_upload_cleanup(database.pool()).await;
    database.drop().await;
}

async fn reserve_upload(repository: &crate::infra::StorageFileRepository) -> crate::domain::SpaceId {
    let space_id = repository.ensure_space("actor", None).await.unwrap();
    repository
        .reserve_upload(space_id.clone(), ByteSize::from_bytes(4), ByteSize::from_bytes(100))
        .await
        .unwrap();
    space_id
}

async fn assert_session_is_terminal(pool: &sqlx::PgPool, session_id: UploadId) {
    let session: (String, i64, bool) = query_as("SELECT state,reserved_bytes,completed_at IS NOT NULL FROM file_upload_session WHERE session_id=$1")
        .bind(session_id.to_string())
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(session, ("aborted".into(), 0, true));
}

async fn assert_upload_cleanup(pool: &sqlx::PgPool) {
    let cleanup: (String, String) = query_as("SELECT cleanup_kind,status FROM file_provider_cleanup WHERE upload_ref='upload-ref'")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(cleanup, ("upload".into(), "pending".into()));
}
