use std::collections::BTreeMap;

use sqlx::{Postgres, query, query_as};
use storage::Database;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::application::{CleanupObject, ProviderCleanupKind, ProviderUploadRef, TrashCleanupBatch, UploadCleanupCandidate};
use crate::domain::{ProviderKey, StoredObjectId, UploadId};
use crate::error::keys;
use crate::{FileError, FileResult};

use super::repository_provider_cleanup::record_tx;
use super::repository_support::storage_error;

// Maximum ownership window for one cleanup worker before a later run may reclaim it.
const CLEANUP_CLAIM_LEASE_MINUTES: i64 = 30;
type ExpiredUploadRow = (String, String, String, String, String, String, i64, String);
type ClaimedUploadRow = (String, String, String, String, String, String, i64, String, String);

#[derive(Default)]
struct TrashCleanupState {
    purged_entries: u64,
    blocked_roots: u64,
    object_refs: BTreeMap<String, i64>,
    spaces: Vec<String>,
}

pub(super) async fn expired_trash(database: &Database, retention_days: u64, batch_size: u64) -> FileResult<TrashCleanupBatch> {
    let cutoff = cutoff(retention_days)?;
    let limit = limit(batch_size)?;
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    let roots: Vec<(String, String)> = query_as(
        "SELECT e.entry_id,e.space_id FROM file_entry e LEFT JOIN file_entry p ON p.entry_id=e.parent_id AND p.status='trashed' WHERE e.status='trashed' AND e.trashed_at<=$1 AND p.entry_id IS NULL ORDER BY e.trashed_at,e.entry_id LIMIT $2 FOR UPDATE OF e SKIP LOCKED",
    )
    .bind(cutoff)
    .bind(limit)
    .fetch_all(&mut *transaction)
    .await
    .map_err(storage_error)?;
    let state = purge_expired_trash_roots(&mut transaction, roots).await?;
    mark_objects(&mut transaction, state.object_refs).await?;
    recalc_spaces(&mut transaction, &state.spaces).await?;
    let objects = deleting_objects(&mut transaction, limit).await?;
    transaction.commit().await.map_err(storage_error)?;
    Ok(TrashCleanupBatch {
        purged_entries: state.purged_entries,
        blocked_roots: state.blocked_roots,
        objects,
    })
}

async fn purge_expired_trash_roots(transaction: &mut sqlx::Transaction<'_, Postgres>, roots: Vec<(String, String)>) -> FileResult<TrashCleanupState> {
    let mut state = TrashCleanupState::default();
    for (root_id, space_id) in roots {
        let root = crate::domain::FileId::parse(&root_id)?;
        if !super::repository_purge::lock_trashed_tree(transaction, root).await? || tree_has_blocking_references(transaction, &root_id).await? {
            state.blocked_roots += 1;
            continue;
        }
        let objects: Vec<(String, i64)> = query_as(
            "WITH RECURSIVE tree AS (SELECT entry_id,object_id FROM file_entry WHERE entry_id=$1 UNION ALL SELECT e.entry_id,e.object_id FROM file_entry e JOIN tree t ON e.parent_id=t.entry_id) SELECT object_id,COUNT(*) FROM tree WHERE object_id IS NOT NULL GROUP BY object_id",
        )
        .bind(&root_id)
        .fetch_all(&mut **transaction)
        .await
        .map_err(storage_error)?;
        for (object_id, count) in objects {
            *state.object_refs.entry(object_id).or_default() += count;
        }
        let result = query(
            "WITH RECURSIVE tree AS (SELECT entry_id FROM file_entry WHERE entry_id=$1 UNION ALL SELECT e.entry_id FROM file_entry e JOIN tree t ON e.parent_id=t.entry_id) DELETE FROM file_entry WHERE entry_id IN (SELECT entry_id FROM tree)",
        )
        .bind(&root_id)
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)?;
        state.purged_entries += result.rows_affected();
        state.spaces.push(space_id);
    }
    Ok(state)
}

async fn tree_has_blocking_references(transaction: &mut sqlx::Transaction<'_, Postgres>, root_id: &str) -> FileResult<bool> {
    let blocked: (bool,) = query_as(
        "WITH RECURSIVE tree AS (SELECT entry_id FROM file_entry WHERE entry_id=$1 UNION ALL SELECT e.entry_id FROM file_entry e JOIN tree t ON e.parent_id=t.entry_id) SELECT EXISTS (SELECT 1 FROM file_business_reference r WHERE r.entry_id IN (SELECT entry_id FROM tree)) OR EXISTS (SELECT 1 FROM sys_user u WHERE u.avatar_file_id IN (SELECT entry_id FROM tree)) OR EXISTS (SELECT 1 FROM file_upload_session us WHERE us.state IN ('open','completing') AND us.parent_id IN (SELECT entry_id FROM tree))",
    )
    .bind(root_id)
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_error)?;
    Ok(blocked.0)
}

pub(super) async fn finalize_cleanup_object(database: &Database, object_id: StoredObjectId) -> FileResult<()> {
    let result = query("DELETE FROM file_object WHERE object_id=$1 AND status='deleting' AND ref_count=0")
        .bind(object_id.to_string())
        .execute(database.pool())
        .await
        .map_err(storage_error)?;
    if result.rows_affected() == 0 {
        return Err(FileError::NotFound);
    }
    Ok(())
}

pub(super) async fn expired_upload_sessions(database: &Database, inactivity_days: u64, batch_size: u64) -> FileResult<Vec<UploadCleanupCandidate>> {
    let cutoff = cutoff(inactivity_days)?;
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    let claimed = claim_expired_upload_sessions(&mut transaction, cutoff, batch_size).await?;
    transaction.commit().await.map_err(storage_error)?;
    upload_cleanup_candidates(claimed)
}

async fn claim_expired_upload_sessions(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    cutoff: OffsetDateTime,
    batch_size: u64,
) -> FileResult<Vec<ClaimedUploadRow>> {
    let claim_cutoff = OffsetDateTime::now_utc() - Duration::minutes(CLEANUP_CLAIM_LEASE_MINUTES);
    let rows: Vec<ExpiredUploadRow> = query_as(
        "SELECT session_id,owner_user_id,provider_key,provider_upload_ref,provider_object_key,state,declared_size_bytes,declared_sha256 FROM file_upload_session s WHERE state IN ('open','completing') AND (cleanup_claim_token IS NULL OR cleanup_claimed_at<=$3) AND last_activity_at<=$1 AND NOT EXISTS (SELECT 1 FROM file_upload_part p WHERE p.session_id=s.session_id AND p.state='writing') ORDER BY last_activity_at,session_id LIMIT $2 FOR UPDATE SKIP LOCKED",
    )
    .bind(cutoff)
    .bind(limit(batch_size)?)
    .bind(claim_cutoff)
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage_error)?;
    let now = OffsetDateTime::now_utc();
    let mut claimed = Vec::with_capacity(rows.len());
    for (session_id, owner_user_id, provider_key, provider_upload_ref, provider_object_key, state, expected_size, expected_digest) in rows {
        let claim_token = Uuid::now_v7().to_string();
        let result = query(
            "UPDATE file_upload_session s SET cleanup_claim_token=$2,cleanup_claimed_at=$3 WHERE session_id=$1 AND state IN ('open','completing') AND (cleanup_claim_token IS NULL OR cleanup_claimed_at<=$5) AND last_activity_at<=$4 AND NOT EXISTS (SELECT 1 FROM file_upload_part p WHERE p.session_id=s.session_id AND p.state='writing')",
        )
        .bind(&session_id)
        .bind(&claim_token)
        .bind(now)
        .bind(cutoff)
        .bind(claim_cutoff)
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)?;
        if result.rows_affected() == 1 {
            claimed.push((
                session_id,
                owner_user_id,
                provider_key,
                provider_upload_ref,
                provider_object_key,
                state,
                expected_size,
                expected_digest,
                claim_token,
            ));
        }
    }
    Ok(claimed)
}

fn upload_cleanup_candidates(claimed: Vec<ClaimedUploadRow>) -> FileResult<Vec<UploadCleanupCandidate>> {
    claimed
        .into_iter()
        .map(
            |(session_id, owner_user_id, provider_key, provider_upload_ref, provider_object_key, state, expected_size, expected_digest, claim_token)| {
                Ok(UploadCleanupCandidate {
                    session_id: UploadId::parse(&session_id)?,
                    owner_user_id,
                    provider_key: ProviderKey::new(provider_key)?,
                    provider_upload_ref: ProviderUploadRef::new(provider_upload_ref)?,
                    provider_object_key: crate::application::ObjectKey::new(provider_object_key)?,
                    expected_size: crate::domain::ByteSize::from_bytes(u64::try_from(expected_size).map_err(|_| FileError::SizeMismatch)?),
                    expected_digest: crate::domain::ContentDigest::from_hex(&expected_digest)?,
                    claim_token,
                    state,
                })
            },
        )
        .collect()
}

pub(super) async fn finalize_expired_upload(database: &Database, session_id: UploadId, claim_token: &str) -> FileResult<()> {
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    let row: Option<(String, i64, String, String, String)> = query_as(
        "SELECT space_id,reserved_bytes,state,provider_key,provider_upload_ref FROM file_upload_session WHERE session_id=$1 AND cleanup_claim_token=$2 AND state IN ('open','completing') FOR UPDATE",
    )
    .bind(session_id.to_string())
    .bind(claim_token)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(storage_error)?;
    let Some((space_id, reserved, state, provider_key, upload_ref)) = row else {
        return Err(FileError::UploadNotFound);
    };
    let terminal = if state == "completing" { "aborted" } else { "expired" };
    let now = OffsetDateTime::now_utc();
    query("DELETE FROM file_upload_part WHERE session_id=$1")
        .bind(session_id.to_string())
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
    let result = query("UPDATE file_upload_session SET state=$2,reserved_bytes=0,cleanup_claim_token=NULL,cleanup_claimed_at=NULL,completed_at=$3,last_activity_at=$3 WHERE session_id=$1 AND cleanup_claim_token=$4")
        .bind(session_id.to_string()).bind(terminal).bind(now).bind(claim_token).execute(&mut *transaction).await.map_err(storage_error)?;
    if result.rows_affected() == 0 {
        return Err(FileError::UploadNotFound);
    }
    query("UPDATE file_space SET reserved_bytes=GREATEST(reserved_bytes-$2,0),updated_at=$3 WHERE space_id=$1")
        .bind(space_id)
        .bind(reserved)
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
    let provider_key = ProviderKey::new(provider_key)?;
    let upload_ref = ProviderUploadRef::new(upload_ref)?;
    record_tx(&mut transaction, &provider_key, ProviderCleanupKind::Upload, None, Some(&upload_ref)).await?;
    transaction.commit().await.map_err(storage_error)
}

pub(super) async fn release_upload_cleanup_claim(database: &Database, session_id: UploadId, claim_token: &str) -> FileResult<()> {
    let result = query("UPDATE file_upload_session SET cleanup_claim_token=NULL,cleanup_claimed_at=NULL WHERE session_id=$1 AND cleanup_claim_token=$2")
        .bind(session_id.to_string())
        .bind(claim_token)
        .execute(database.pool())
        .await
        .map_err(storage_error)?;
    if result.rows_affected() == 0 {
        return Err(FileError::UploadNotFound);
    }
    Ok(())
}

async fn mark_objects(transaction: &mut sqlx::Transaction<'_, Postgres>, refs: BTreeMap<String, i64>) -> FileResult<()> {
    for (object_id, count) in refs {
        query("UPDATE file_object SET ref_count=GREATEST(ref_count-$2,0),status=CASE WHEN ref_count<=$2 THEN 'deleting' ELSE status END,updated_at=$3 WHERE object_id=$1")
            .bind(object_id).bind(count).bind(OffsetDateTime::now_utc()).execute(&mut **transaction).await.map_err(storage_error)?;
    }
    Ok(())
}

async fn deleting_objects(transaction: &mut sqlx::Transaction<'_, Postgres>, limit: i64) -> FileResult<Vec<CleanupObject>> {
    let rows: Vec<(String, String, String)> = query_as("SELECT o.object_id,o.provider_key,o.object_key FROM file_object o WHERE o.status='deleting' AND o.ref_count=0 AND NOT EXISTS (SELECT 1 FROM file_provider_cleanup c WHERE c.provider_key=o.provider_key AND c.cleanup_kind='object' AND c.object_key=o.object_key AND c.status<>'done') ORDER BY o.updated_at,o.object_id LIMIT $1 FOR UPDATE OF o SKIP LOCKED")
        .bind(limit).fetch_all(&mut **transaction).await.map_err(storage_error)?;
    rows.into_iter()
        .map(|(object_id, provider_key, object_key)| {
            Ok(CleanupObject {
                object_id: StoredObjectId::parse(&object_id)?,
                provider_key: ProviderKey::new(provider_key)?,
                object_key: crate::application::ObjectKey::new(object_key)?,
            })
        })
        .collect()
}

async fn recalc_spaces(transaction: &mut sqlx::Transaction<'_, Postgres>, spaces: &[String]) -> FileResult<()> {
    for space in spaces {
        query("UPDATE file_space SET active_bytes=COALESCE((SELECT SUM(o.size_bytes) FROM file_entry e JOIN file_object o ON o.object_id=e.object_id WHERE e.space_id=$1 AND e.status='active'),0),trashed_bytes=COALESCE((SELECT SUM(o.size_bytes) FROM file_entry e JOIN file_object o ON o.object_id=e.object_id WHERE e.space_id=$1 AND e.status='trashed'),0),updated_at=$2 WHERE space_id=$1")
            .bind(space).bind(OffsetDateTime::now_utc()).execute(&mut **transaction).await.map_err(storage_error)?;
    }
    Ok(())
}

fn cutoff(days: u64) -> FileResult<OffsetDateTime> {
    let days = i64::try_from(days).map_err(|_| FileError::InvalidInput(keys::RETENTION_DAYS_TOO_LARGE))?;
    OffsetDateTime::now_utc()
        .checked_sub(Duration::days(days))
        .ok_or(FileError::InvalidInput(keys::RETENTION_DAYS_TOO_LARGE))
}

fn limit(batch_size: u64) -> FileResult<i64> {
    if batch_size == 0 {
        return Err(FileError::InvalidInput(keys::CLEANUP_BATCH_SIZE_INVALID));
    }
    i64::try_from(batch_size).map_err(|_| FileError::InvalidInput(keys::CLEANUP_BATCH_SIZE_TOO_LARGE))
}
