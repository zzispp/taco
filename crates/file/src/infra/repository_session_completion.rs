use sqlx::{Postgres, QueryBuilder, query, query_as};
use storage::Database;
use time::OffsetDateTime;

use crate::application::{FileAccessScope, ProviderCleanupKind, ProviderCleanupRecordRequest, StoredObject, UploadCompletionResult, UploadSessionData};
use crate::domain::{ContentDigest, FileId, UploadId};
use crate::error::keys;
use crate::{FileError, FileResult};

use super::repository_provider_cleanup::{cancel_object_cleanup_tx, record_tx};
use super::repository_queries::find_entry;
use super::repository_session_core::get_upload_session;
use super::repository_session_support::{ReservationRelease, release_reservation_tx};
use super::repository_support::{ensure_active_parent_tx, same_physical_object, storage_error};

#[path = "repository_session_completion_persistence.rs"]
mod persistence;

pub(super) struct UploadSessionCompletion<'a> {
    pub(super) actor: &'a FileAccessScope,
    pub(super) session_id: UploadId,
    pub(super) claim_token: Option<&'a str>,
    pub(super) object: StoredObject,
}

pub(super) struct ClaimedUploadSessionCompletion<'a> {
    pub(super) session_id: UploadId,
    pub(super) claim_token: &'a str,
    pub(super) object: StoredObject,
}

impl<'a> UploadSessionCompletion<'a> {
    pub(super) fn new(actor: &'a FileAccessScope, session_id: UploadId, object: StoredObject) -> Self {
        Self {
            actor,
            session_id,
            claim_token: None,
            object,
        }
    }
}

impl<'a> ClaimedUploadSessionCompletion<'a> {
    pub(super) fn new(session_id: UploadId, claim_token: &'a str, object: StoredObject) -> Self {
        Self {
            session_id,
            claim_token,
            object,
        }
    }
}

pub(super) struct CompletionContext<'a> {
    pub(super) actor: &'a FileAccessScope,
    pub(super) session: UploadSessionData,
    pub(super) object: StoredObject,
    pub(super) session_id: UploadId,
    pub(super) claim_token: Option<&'a str>,
    pub(super) digest: ContentDigest,
    pub(super) size: i64,
    pub(super) entry_id: FileId,
    pub(super) now: OffsetDateTime,
}

impl<'a> CompletionContext<'a> {
    fn new(request: UploadSessionCompletion<'a>, session: UploadSessionData) -> FileResult<Self> {
        let UploadSessionCompletion {
            actor,
            session_id,
            claim_token,
            object,
        } = request;
        if session.owner_user_id != actor.user_id || session.state != "completing" {
            return Err(FileError::UploadNotFound);
        }
        let digest = object.digest.ok_or(FileError::DigestMismatch)?;
        if object.size != session.size || digest != session.digest {
            return Err(FileError::DigestMismatch);
        }
        if object.provider_key != session.provider_key || object.key != session.provider_object_key {
            return Err(FileError::InvalidInput(keys::PROVIDER_OBJECT_MISMATCH));
        }
        let size = i64::try_from(object.size.bytes()).map_err(|_| FileError::SizeMismatch)?;
        Ok(Self {
            actor,
            session,
            object,
            session_id,
            claim_token,
            digest,
            size,
            entry_id: FileId::new(),
            now: OffsetDateTime::now_utc(),
        })
    }
}

pub(super) async fn finish_upload_session(database: &Database, request: UploadSessionCompletion<'_>) -> FileResult<UploadCompletionResult> {
    let UploadSessionCompletion {
        actor,
        session_id,
        claim_token,
        object,
    } = request;
    match complete_upload_session(
        database,
        UploadSessionCompletion {
            actor,
            session_id,
            claim_token,
            object,
        },
    )
    .await
    {
        Err(FileError::UploadNotFound) => completed_upload_result(database, actor, session_id).await,
        result => result,
    }
}

pub(super) async fn finish_claimed_upload_session(database: &Database, request: ClaimedUploadSessionCompletion<'_>) -> FileResult<UploadCompletionResult> {
    let ClaimedUploadSessionCompletion {
        session_id,
        claim_token,
        object,
    } = request;
    let owner: Option<(String,)> = query_as("SELECT owner_user_id FROM file_upload_session WHERE session_id=$1 AND cleanup_claim_token=$2")
        .bind(session_id.to_string())
        .bind(claim_token)
        .fetch_optional(database.pool())
        .await
        .map_err(storage_error)?;
    let Some((owner_user_id,)) = owner else {
        return Err(FileError::UploadNotFound);
    };
    let actor = FileAccessScope::self_only(owner_user_id, None);
    complete_upload_session(
        database,
        UploadSessionCompletion {
            actor: &actor,
            session_id,
            claim_token: Some(claim_token),
            object,
        },
    )
    .await
}

async fn complete_upload_session(database: &Database, request: UploadSessionCompletion<'_>) -> FileResult<UploadCompletionResult> {
    let actor = request.actor;
    let session_id = request.session_id;
    let (session, _) = get_upload_session(database, actor, session_id).await?.ok_or(FileError::UploadNotFound)?;
    let context = CompletionContext::new(request, session)?;
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    lock_completion_session(&mut transaction, &context).await?;
    ensure_active_parent_tx(&mut transaction, context.session.space_id.as_str(), context.session.parent_id)
        .await
        .map_err(completion_parent_error)?;
    if let Some((reused_entry_id, object_to_delete)) = reuse_named_entry(&mut transaction, &context).await? {
        transaction.commit().await.map_err(storage_error)?;
        let entry = find_entry(database, context.actor, reused_entry_id).await?.ok_or(FileError::NotFound)?;
        return Ok(UploadCompletionResult { entry, object_to_delete });
    }
    let (object_id, object_to_delete) = persistence::adopt_or_insert_object(&mut transaction, &context).await?;
    persistence::insert_file_entry(&mut transaction, &object_id, &context).await?;
    query("UPDATE file_space SET active_bytes=active_bytes+$2,reserved_bytes=GREATEST(reserved_bytes-$2,0),updated_at=$3 WHERE space_id=$1")
        .bind(context.session.space_id.as_str())
        .bind(context.size)
        .bind(context.now)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
    persistence::mark_session_completed(&mut transaction, &context, context.entry_id).await?;
    transaction.commit().await.map_err(storage_error)?;
    let entry = find_entry(database, context.actor, context.entry_id).await?.ok_or(FileError::NotFound)?;
    Ok(UploadCompletionResult { entry, object_to_delete })
}

async fn completed_upload_result(database: &Database, actor: &FileAccessScope, session_id: UploadId) -> FileResult<UploadCompletionResult> {
    let (session, _) = get_upload_session(database, actor, session_id).await?.ok_or(FileError::UploadNotFound)?;
    if session.owner_user_id != actor.user_id || session.state != "completed" {
        return Err(FileError::UploadNotFound);
    }
    let entry_id = session.result_entry_id.ok_or(FileError::UploadResultUnavailable)?;
    let entry = find_entry(database, actor, entry_id).await?.ok_or(FileError::UploadResultUnavailable)?;
    Ok(UploadCompletionResult { entry, object_to_delete: None })
}

fn completion_parent_error(error: FileError) -> FileError {
    match error {
        FileError::NotFound => FileError::InvalidInput(keys::PARENT_FOLDER_INVALID),
        error => error,
    }
}

async fn lock_completion_session(transaction: &mut sqlx::Transaction<'_, Postgres>, context: &CompletionContext<'_>) -> FileResult<()> {
    let mut lock = QueryBuilder::<Postgres>::new("SELECT session_id FROM file_upload_session WHERE session_id=");
    lock.push_bind(context.session_id.to_string())
        .push(" AND owner_user_id=")
        .push_bind(&context.actor.user_id)
        .push(" AND state='completing' AND ");
    if let Some(token) = context.claim_token {
        lock.push("cleanup_claim_token=").push_bind(token);
    } else {
        lock.push("cleanup_claim_token IS NULL");
    }
    lock.push(" FOR UPDATE");
    let row: Option<(String,)> = lock.build_query_as().fetch_optional(&mut **transaction).await.map_err(storage_error)?;
    row.map(|_| ()).ok_or(FileError::UploadNotFound)
}

async fn reuse_named_entry(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    context: &CompletionContext<'_>,
) -> FileResult<Option<(FileId, Option<StoredObject>)>> {
    let sibling: Option<(String, String, String, String, i64)> = query_as("SELECT e.entry_id,o.provider_key,o.object_key,o.sha256,o.size_bytes FROM file_entry e JOIN file_object o ON o.object_id=e.object_id WHERE e.space_id=$1 AND COALESCE(e.parent_id,'')=COALESCE($2,'') AND e.normalized_name=$3 AND e.status='active' FOR UPDATE OF e,o")
        .bind(context.session.space_id.as_str()).bind(super::repository_session_support::parent_value(context.session.parent_id)).bind(context.session.name.normalized()).fetch_optional(&mut **transaction).await.map_err(storage_error)?;
    let Some((entry_id, provider_key, object_key, existing_digest, existing_size)) = sibling else {
        return Ok(None);
    };
    if existing_digest != context.digest.to_hex() || existing_size != context.size {
        return Err(FileError::NameConflict);
    }
    let entry_id = FileId::parse(&entry_id)?;
    release_reservation_tx(
        transaction,
        ReservationRelease {
            space_id: &context.session.space_id,
            size: context.size,
            now: context.now,
        },
    )
    .await?;
    persistence::mark_session_completed(transaction, context, entry_id).await?;
    let adopted = same_physical_object(&provider_key, &object_key, &context.object);
    if adopted {
        cancel_object_cleanup_tx(transaction, &context.object.provider_key, &context.object.key).await?;
    } else {
        record_tx(
            transaction,
            ProviderCleanupRecordRequest {
                provider_key: &context.object.provider_key,
                kind: ProviderCleanupKind::Object,
                object_key: Some(&context.object.key),
                upload_ref: None,
            },
        )
        .await?;
    }
    Ok(Some((entry_id, (!adopted).then_some(context.object.clone()))))
}
