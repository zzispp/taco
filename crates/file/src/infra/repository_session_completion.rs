use sqlx::{Postgres, QueryBuilder, query, query_as};
use storage::Database;
use time::OffsetDateTime;

use crate::application::{FileAccessScope, ProviderCleanupKind, StoredObject, UploadCompletionResult};
use crate::domain::{FileId, UploadId};
use crate::error::keys;
use crate::{FileError, FileResult};

use super::repository_provider_cleanup::{cancel_object_cleanup_tx, record_tx};
use super::repository_queries::find_entry;
use super::repository_session_core::get_upload_session;
use super::repository_session_support::{map_insert, parent_value, release_reservation_tx};
use super::repository_support::{ensure_active_parent_tx, same_physical_object, storage_error};

pub(super) async fn finish_upload_session(
    database: &Database,
    actor: &FileAccessScope,
    session_id: UploadId,
    object: StoredObject,
) -> FileResult<UploadCompletionResult> {
    match finish_upload_session_inner(database, actor, session_id, None, object).await {
        Err(FileError::UploadNotFound) => completed_upload_result(database, actor, session_id).await,
        result => result,
    }
}

pub(super) async fn finish_claimed_upload_session(
    database: &Database,
    session_id: UploadId,
    claim_token: &str,
    object: StoredObject,
) -> FileResult<UploadCompletionResult> {
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
    finish_upload_session_inner(database, &actor, session_id, Some(claim_token), object).await
}

async fn finish_upload_session_inner(
    database: &Database,
    actor: &FileAccessScope,
    session_id: UploadId,
    claim_token: Option<&str>,
    object: StoredObject,
) -> FileResult<UploadCompletionResult> {
    let (session, _) = get_upload_session(database, actor, session_id).await?.ok_or(FileError::UploadNotFound)?;
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
    let entry_id = FileId::new();
    let now = OffsetDateTime::now_utc();
    let size = i64::try_from(object.size.bytes()).map_err(|_| FileError::SizeMismatch)?;
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    lock_completion_session(&mut transaction, &actor.user_id, session_id, claim_token).await?;
    ensure_active_parent_tx(&mut transaction, session.space_id.as_str(), session.parent_id)
        .await
        .map_err(completion_parent_error)?;
    if let Some((reused_entry_id, object_to_delete)) = reuse_named_entry(&mut transaction, &session, &object, session_id, claim_token, now).await? {
        transaction.commit().await.map_err(storage_error)?;
        let entry = find_entry(database, actor, reused_entry_id).await?.ok_or(FileError::NotFound)?;
        return Ok(UploadCompletionResult { entry, object_to_delete });
    }
    let (object_id, object_to_delete) = adopt_or_insert_object(&mut transaction, &session, &object, digest, size, now).await?;
    insert_file_entry(&mut transaction, &session, &object_id, entry_id, actor, now).await?;
    query("UPDATE file_space SET active_bytes=active_bytes+$2,reserved_bytes=GREATEST(reserved_bytes-$2,0),updated_at=$3 WHERE space_id=$1")
        .bind(session.space_id.as_str())
        .bind(size)
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
    mark_session_completed(&mut transaction, session_id, entry_id, claim_token, now).await?;
    transaction.commit().await.map_err(storage_error)?;
    let entry = find_entry(database, actor, entry_id).await?.ok_or(FileError::NotFound)?;
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

async fn lock_completion_session(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    owner: &str,
    session_id: UploadId,
    claim_token: Option<&str>,
) -> FileResult<()> {
    let mut lock = QueryBuilder::<Postgres>::new("SELECT session_id FROM file_upload_session WHERE session_id=");
    lock.push_bind(session_id.to_string())
        .push(" AND owner_user_id=")
        .push_bind(owner)
        .push(" AND state='completing' AND ");
    if let Some(token) = claim_token {
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
    session: &crate::application::UploadSessionData,
    object: &StoredObject,
    session_id: UploadId,
    claim_token: Option<&str>,
    now: OffsetDateTime,
) -> FileResult<Option<(FileId, Option<StoredObject>)>> {
    let digest = object.digest.ok_or(FileError::DigestMismatch)?;
    let size = i64::try_from(object.size.bytes()).map_err(|_| FileError::SizeMismatch)?;
    let sibling: Option<(String, String, String, String, i64)> = query_as("SELECT e.entry_id,o.provider_key,o.object_key,o.sha256,o.size_bytes FROM file_entry e JOIN file_object o ON o.object_id=e.object_id WHERE e.space_id=$1 AND COALESCE(e.parent_id,'')=COALESCE($2,'') AND e.normalized_name=$3 AND e.status='active' FOR UPDATE OF e,o")
        .bind(session.space_id.as_str()).bind(parent_value(session.parent_id)).bind(session.name.normalized()).fetch_optional(&mut **transaction).await.map_err(storage_error)?;
    let Some((entry_id, provider_key, object_key, existing_digest, existing_size)) = sibling else {
        return Ok(None);
    };
    if existing_digest != digest.to_hex() || existing_size != size {
        return Err(FileError::NameConflict);
    }
    let entry_id = FileId::parse(&entry_id)?;
    release_reservation_tx(transaction, &session.space_id, size, now).await?;
    mark_session_completed(transaction, session_id, entry_id, claim_token, now).await?;
    let adopted = same_physical_object(&provider_key, &object_key, object);
    if adopted {
        cancel_object_cleanup_tx(transaction, &object.provider_key, &object.key).await?;
    } else {
        record_tx(transaction, &object.provider_key, ProviderCleanupKind::Object, Some(&object.key), None).await?;
    }
    Ok(Some((entry_id, (!adopted).then_some(object.clone()))))
}

async fn adopt_or_insert_object(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    session: &crate::application::UploadSessionData,
    object: &StoredObject,
    digest: crate::domain::ContentDigest,
    size: i64,
    now: OffsetDateTime,
) -> FileResult<(String, Option<StoredObject>)> {
    cancel_object_cleanup_tx(transaction, &object.provider_key, &object.key).await?;
    if let Some(canonical) = canonical_object(transaction, digest, size).await? {
        return adopt_canonical_object(transaction, canonical, object, now).await;
    }
    if insert_object(transaction, session, object, digest, size, now).await? {
        return Ok((object.id.to_string(), None));
    }
    let canonical = canonical_object(transaction, digest, size)
        .await?
        .ok_or_else(|| FileError::Infrastructure("content deduplication object disappeared during completion".into()))?;
    adopt_canonical_object(transaction, canonical, object, now).await
}

struct CanonicalObject {
    id: String,
    provider_key: String,
    object_key: String,
}

async fn canonical_object(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    digest: crate::domain::ContentDigest,
    size: i64,
) -> FileResult<Option<CanonicalObject>> {
    query_as("SELECT object_id,provider_key,object_key FROM file_object WHERE sha256=$1 AND size_bytes=$2 AND status='active' FOR UPDATE")
        .bind(digest.to_hex())
        .bind(size)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)
        .map(|row: Option<(String, String, String)>| row.map(|(id, provider_key, object_key)| CanonicalObject { id, provider_key, object_key }))
}

async fn insert_object(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    session: &crate::application::UploadSessionData,
    object: &StoredObject,
    digest: crate::domain::ContentDigest,
    size: i64,
    now: OffsetDateTime,
) -> FileResult<bool> {
    query("INSERT INTO file_object(object_id,provider_key,object_key,size_bytes,sha256,content_type,ref_count,status,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,1,'active',$7,$7) ON CONFLICT DO NOTHING")
        .bind(object.id.to_string())
        .bind(object.provider_key.as_str())
        .bind(object.key.as_str())
        .bind(size)
        .bind(digest.to_hex())
        .bind(&session.content_type)
        .bind(now)
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)
        .map(|result| result.rows_affected() == 1)
}

async fn adopt_canonical_object(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    canonical: CanonicalObject,
    object: &StoredObject,
    now: OffsetDateTime,
) -> FileResult<(String, Option<StoredObject>)> {
    query("UPDATE file_object SET ref_count=ref_count+1,updated_at=$2 WHERE object_id=$1")
        .bind(&canonical.id)
        .bind(now)
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)?;
    let adopted = same_physical_object(&canonical.provider_key, &canonical.object_key, object);
    if !adopted {
        record_tx(transaction, &object.provider_key, ProviderCleanupKind::Object, Some(&object.key), None).await?;
    }
    Ok((canonical.id, (!adopted).then_some(object.clone())))
}

async fn insert_file_entry(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    session: &crate::application::UploadSessionData,
    object_id: &str,
    entry_id: FileId,
    actor: &FileAccessScope,
    now: OffsetDateTime,
) -> FileResult<()> {
    query("INSERT INTO file_entry(entry_id,space_id,parent_id,kind,name,normalized_name,object_id,status,created_by,created_at,updated_by,updated_at) VALUES($1,$2,$3,'file',$4,$5,$6,'active',$7,$8,$7,$8)")
        .bind(entry_id.to_string()).bind(session.space_id.as_str()).bind(parent_value(session.parent_id)).bind(session.name.as_str()).bind(session.name.normalized()).bind(object_id).bind(&actor.user_id).bind(now).execute(&mut **transaction).await.map_err(map_insert)?;
    Ok(())
}

async fn mark_session_completed(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    session_id: UploadId,
    entry_id: FileId,
    claim_token: Option<&str>,
    now: OffsetDateTime,
) -> FileResult<()> {
    let mut update = QueryBuilder::<Postgres>::new(
        "UPDATE file_upload_session SET state='completed',reserved_bytes=0,cleanup_claim_token=NULL,cleanup_claimed_at=NULL,result_entry_id=",
    );
    update
        .push_bind(entry_id.to_string())
        .push(",completed_at=")
        .push_bind(now)
        .push(",last_activity_at=")
        .push_bind(now)
        .push(" WHERE session_id=")
        .push_bind(session_id.to_string())
        .push(" AND state='completing' AND ");
    if let Some(token) = claim_token {
        update.push("cleanup_claim_token=").push_bind(token);
    } else {
        update.push("cleanup_claim_token IS NULL");
    }
    let result = update.build().execute(&mut **transaction).await.map_err(storage_error)?;
    if result.rows_affected() == 0 {
        return Err(FileError::UploadNotFound);
    }
    Ok(())
}
