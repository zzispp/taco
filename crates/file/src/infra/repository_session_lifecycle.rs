use storage::Database;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::application::{ProviderCleanupKind, ProviderUploadRef, StoredObject, UploadCompletionTermination};
use crate::domain::{FileId, ProviderKey, UploadId};
use crate::{FileError, FileResult};

use super::repository_provider_cleanup::record_tx;
use super::repository_support::storage_error;

pub(super) async fn claim_upload_cancellation(database: &Database, owner_user_id: &str, session_id: UploadId) -> FileResult<String> {
    let claim_token = Uuid::now_v7().to_string();
    let result = sqlx::query(
        "UPDATE file_upload_session SET cleanup_claim_token=$3,cleanup_claimed_at=$4 WHERE session_id=$1 AND owner_user_id=$2 AND state='open' AND cleanup_claim_token IS NULL",
    )
    .bind(session_id.to_string())
    .bind(owner_user_id)
    .bind(&claim_token)
    .bind(OffsetDateTime::now_utc())
    .execute(database.pool())
    .await
    .map_err(storage_error)?;
    if result.rows_affected() == 0 {
        return Err(FileError::UploadNotFound);
    }
    Ok(claim_token)
}

pub(super) async fn cancel_upload(database: &Database, owner_user_id: &str, session_id: UploadId, claim_token: &str) -> FileResult<()> {
    let now = OffsetDateTime::now_utc();
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    let row = sqlx::query_as::<_, (String, i64, String, String)>(
        "SELECT space_id,reserved_bytes,provider_key,provider_upload_ref FROM file_upload_session WHERE session_id=$1 AND owner_user_id=$2 AND state='open' AND cleanup_claim_token=$3 FOR UPDATE",
    )
    .bind(session_id.to_string())
    .bind(owner_user_id)
    .bind(claim_token)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(storage_error)?;
    let Some((space_id, reserved, provider_key, upload_ref)) = row else {
        return Err(FileError::UploadNotFound);
    };
    let cleanup = upload_cleanup_reference(provider_key, upload_ref)?;
    sqlx::query(
        "UPDATE file_upload_session SET state='aborted',reserved_bytes=0,cleanup_claim_token=NULL,cleanup_claimed_at=NULL,completed_at=$2,last_activity_at=$2 WHERE session_id=$1 AND cleanup_claim_token=$3",
    )
    .bind(session_id.to_string())
    .bind(now)
    .bind(claim_token)
    .execute(&mut *transaction)
    .await
    .map_err(storage_error)?;
    sqlx::query("UPDATE file_space SET reserved_bytes=GREATEST(reserved_bytes-$2,0),updated_at=$3 WHERE space_id=$1")
        .bind(space_id)
        .bind(reserved)
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
    record_upload_cleanup(&mut transaction, &cleanup).await?;
    transaction.commit().await.map_err(storage_error)
}

pub(super) async fn abort_upload_completion(
    database: &Database,
    owner_user_id: &str,
    session_id: UploadId,
    object: StoredObject,
) -> FileResult<UploadCompletionTermination> {
    terminate_upload_completion(
        database,
        CompletionTermination {
            session_id,
            owner_user_id: Some(owner_user_id),
            claim_token: None,
            terminal_state: "aborted",
        },
        Some(object),
    )
    .await
}

pub(super) async fn abort_upload_completion_without_object(
    database: &Database,
    owner_user_id: &str,
    session_id: UploadId,
) -> FileResult<UploadCompletionTermination> {
    terminate_upload_completion(
        database,
        CompletionTermination {
            session_id,
            owner_user_id: Some(owner_user_id),
            claim_token: None,
            terminal_state: "aborted",
        },
        None,
    )
    .await
}

pub(super) async fn abort_claimed_upload_completion(
    database: &Database,
    session_id: UploadId,
    claim_token: &str,
    object: StoredObject,
) -> FileResult<UploadCompletionTermination> {
    terminate_upload_completion(
        database,
        CompletionTermination {
            session_id,
            owner_user_id: None,
            claim_token: Some(claim_token),
            terminal_state: "expired",
        },
        Some(object),
    )
    .await
}

struct CompletionTermination<'a> {
    session_id: UploadId,
    owner_user_id: Option<&'a str>,
    claim_token: Option<&'a str>,
    terminal_state: &'static str,
}

async fn terminate_upload_completion(
    database: &Database,
    request: CompletionTermination<'_>,
    object: Option<StoredObject>,
) -> FileResult<UploadCompletionTermination> {
    let now = OffsetDateTime::now_utc();
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    let session = load_completion_session(&mut transaction, request.session_id).await?;
    if session.state == "completed" {
        return completed_termination(&session);
    }
    validate_termination(&request, &session)?;
    apply_termination(
        &mut transaction,
        CompletionTerminationChange {
            request: &request,
            session,
            object,
            now,
        },
    )
    .await?;
    transaction.commit().await.map_err(storage_error)?;
    Ok(UploadCompletionTermination::Terminated)
}

struct CompletionSession {
    state: String,
    result_entry_id: Option<String>,
    space_id: String,
    reserved: i64,
    owner: String,
    claim_token: Option<String>,
    provider_key: String,
    upload_ref: String,
}

async fn load_completion_session(transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>, session_id: UploadId) -> FileResult<CompletionSession> {
    let row = sqlx::query_as::<_, (String, Option<String>, String, i64, String, Option<String>, String, String)>(
        "SELECT state,result_entry_id,space_id,reserved_bytes,owner_user_id,cleanup_claim_token,provider_key,provider_upload_ref FROM file_upload_session WHERE session_id=$1 FOR UPDATE",
    )
    .bind(session_id.to_string())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?;
    let Some((state, result_entry_id, space_id, reserved, owner, claim_token, provider_key, upload_ref)) = row else {
        return Err(FileError::UploadNotFound);
    };
    Ok(CompletionSession {
        state,
        result_entry_id,
        space_id,
        reserved,
        owner,
        claim_token,
        provider_key,
        upload_ref,
    })
}

fn completed_termination(session: &CompletionSession) -> FileResult<UploadCompletionTermination> {
    session
        .result_entry_id
        .as_deref()
        .ok_or(FileError::UploadResultUnavailable)
        .and_then(FileId::parse)
        .map(UploadCompletionTermination::Completed)
}

fn validate_termination(request: &CompletionTermination<'_>, session: &CompletionSession) -> FileResult<()> {
    if request.owner_user_id.is_some_and(|expected| expected != session.owner) {
        return Err(FileError::UploadNotFound);
    }
    if session.state != "completing" || session.claim_token.as_deref() != request.claim_token {
        return Err(FileError::UploadNotFound);
    }
    Ok(())
}

struct CompletionTerminationChange<'a> {
    request: &'a CompletionTermination<'a>,
    session: CompletionSession,
    object: Option<StoredObject>,
    now: OffsetDateTime,
}

async fn apply_termination(transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>, change: CompletionTerminationChange<'_>) -> FileResult<()> {
    let CompletionTerminationChange { request, session, object, now } = change;
    sqlx::query("UPDATE file_upload_session SET state=$2,reserved_bytes=0,cleanup_claim_token=NULL,cleanup_claimed_at=NULL,completed_at=$3,last_activity_at=$3 WHERE session_id=$1 AND state='completing'")
        .bind(request.session_id.to_string())
        .bind(request.terminal_state)
        .bind(now)
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)?;
    sqlx::query("UPDATE file_space SET reserved_bytes=GREATEST(reserved_bytes-$2,0),updated_at=$3 WHERE space_id=$1")
        .bind(session.space_id)
        .bind(session.reserved)
        .bind(now)
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)?;
    if let Some(object) = object {
        record_tx(transaction, &object.provider_key, ProviderCleanupKind::Object, Some(&object.key), None).await?;
    } else {
        let cleanup = upload_cleanup_reference(session.provider_key, session.upload_ref)?;
        record_upload_cleanup(transaction, &cleanup).await?;
    }
    Ok(())
}

struct UploadCleanupReference {
    provider_key: ProviderKey,
    upload_ref: ProviderUploadRef,
}

fn upload_cleanup_reference(provider_key: String, upload_ref: String) -> FileResult<UploadCleanupReference> {
    Ok(UploadCleanupReference {
        provider_key: ProviderKey::new(provider_key)?,
        upload_ref: ProviderUploadRef::new(upload_ref)?,
    })
}

async fn record_upload_cleanup(transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>, cleanup: &UploadCleanupReference) -> FileResult<()> {
    record_tx(transaction, &cleanup.provider_key, ProviderCleanupKind::Upload, None, Some(&cleanup.upload_ref)).await
}
