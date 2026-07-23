use sqlx::{Postgres, query, query_as};
use storage::Database;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::application::{ObjectKey, ProviderCleanupCandidate, ProviderCleanupKind, ProviderUploadRef};
use crate::domain::ProviderKey;
use crate::error::keys;
use crate::{FileError, FileResult};

use super::repository_support::storage_error;

const CLAIM_LEASE_MINUTES: i64 = 30;
const RETRY_DELAY_MINUTES: i64 = 5;
type ProviderCleanupRow = (String, String, String, Option<String>, Option<String>);

pub(super) async fn record(
    database: &Database,
    provider_key: &ProviderKey,
    kind: ProviderCleanupKind,
    object_key: Option<&ObjectKey>,
    upload_ref: Option<&ProviderUploadRef>,
) -> FileResult<()> {
    validate_payload(kind, object_key, upload_ref)?;
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    record_tx(&mut transaction, provider_key, kind, object_key, upload_ref).await?;
    transaction.commit().await.map_err(storage_error)?;
    Ok(())
}

pub(super) async fn record_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    provider_key: &ProviderKey,
    kind: ProviderCleanupKind,
    object_key: Option<&ObjectKey>,
    upload_ref: Option<&ProviderUploadRef>,
) -> FileResult<()> {
    validate_payload(kind, object_key, upload_ref)?;
    let now = OffsetDateTime::now_utc();
    query(
        "INSERT INTO file_provider_cleanup(cleanup_id,provider_key,cleanup_kind,object_key,upload_ref,status,attempt_count,next_attempt_at,created_at,updated_at) VALUES($1,$2,$3,$4,$5,'pending',0,$6,$6,$6) ON CONFLICT DO NOTHING",
    )
    .bind(Uuid::now_v7().to_string())
    .bind(provider_key.as_str())
    .bind(kind_name(kind))
    .bind(object_key.map(ObjectKey::as_str))
    .bind(upload_ref.map(ProviderUploadRef::as_str))
    .bind(now)
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)?;
    Ok(())
}

pub(super) async fn cancel_object_cleanup_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    provider_key: &ProviderKey,
    object_key: &ObjectKey,
) -> FileResult<()> {
    let cleanup: Option<(String, String)> = query_as(
        "SELECT cleanup_id,status FROM file_provider_cleanup WHERE provider_key=$1 AND cleanup_kind='object' AND object_key=$2 AND status<>'done' FOR UPDATE",
    )
    .bind(provider_key.as_str())
    .bind(object_key.as_str())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?;
    let Some((cleanup_id, status)) = cleanup else {
        return Ok(());
    };
    if status == "deleting" {
        return Err(FileError::ProviderUnavailable {
            operation: "adopt object during provider cleanup",
        });
    }
    query("UPDATE file_provider_cleanup SET status='done',last_error_code=NULL,updated_at=$2 WHERE cleanup_id=$1")
        .bind(cleanup_id)
        .bind(OffsetDateTime::now_utc())
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)?;
    Ok(())
}

pub(super) async fn claim(database: &Database, batch_size: u64) -> FileResult<Vec<ProviderCleanupCandidate>> {
    let limit = i64::try_from(batch_size).map_err(|_| FileError::InvalidInput(keys::CLEANUP_BATCH_SIZE_TOO_LARGE))?;
    if limit <= 0 {
        return Err(FileError::InvalidInput(keys::CLEANUP_BATCH_SIZE_INVALID));
    }
    let now = OffsetDateTime::now_utc();
    let lease_cutoff = now - Duration::minutes(CLAIM_LEASE_MINUTES);
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    let rows: Vec<ProviderCleanupRow> = query_as(
        "SELECT cleanup_id,provider_key,cleanup_kind,object_key,upload_ref FROM file_provider_cleanup WHERE (status IN ('pending','error') AND next_attempt_at<=$1) OR (status='deleting' AND claimed_at<=$2) ORDER BY next_attempt_at,cleanup_id LIMIT $3 FOR UPDATE SKIP LOCKED",
    )
    .bind(now)
    .bind(lease_cutoff)
    .bind(limit)
    .fetch_all(&mut *transaction)
    .await
    .map_err(storage_error)?;
    let mut candidates = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(candidate) = claim_row(&mut transaction, row, now).await? {
            candidates.push(candidate);
        }
    }
    transaction.commit().await.map_err(storage_error)?;
    Ok(candidates)
}

async fn claim_row(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    row: ProviderCleanupRow,
    now: OffsetDateTime,
) -> FileResult<Option<ProviderCleanupCandidate>> {
    let (cleanup_id, provider, kind, object_key, upload_ref) = row;
    let provider_key = ProviderKey::new(provider)?;
    let kind = parse_kind(&kind)?;
    if kind == ProviderCleanupKind::Object && cleanup_references_active_object(transaction, &provider_key, object_key.as_deref()).await? {
        query("UPDATE file_provider_cleanup SET status='done',last_error_code=NULL,updated_at=$2 WHERE cleanup_id=$1")
            .bind(cleanup_id)
            .bind(now)
            .execute(&mut **transaction)
            .await
            .map_err(storage_error)?;
        return Ok(None);
    }
    let token = Uuid::now_v7().to_string();
    query("UPDATE file_provider_cleanup SET status='deleting',claim_token=$2,claimed_at=$3,attempt_count=attempt_count+1,updated_at=$3 WHERE cleanup_id=$1")
        .bind(&cleanup_id)
        .bind(&token)
        .bind(now)
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)?;
    Ok(Some(ProviderCleanupCandidate {
        cleanup_id,
        provider_key,
        kind,
        object_key: object_key.map(ObjectKey::new).transpose()?,
        upload_ref: upload_ref.map(ProviderUploadRef::new).transpose()?,
        claim_token: token,
    }))
}

async fn cleanup_references_active_object(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    provider_key: &ProviderKey,
    object_key: Option<&str>,
) -> FileResult<bool> {
    let object_key = object_key.ok_or(FileError::InvalidInput(keys::PROVIDER_CLEANUP_PAYLOAD_INVALID))?;
    query_as::<_, (bool,)>("SELECT EXISTS(SELECT 1 FROM file_object WHERE provider_key=$1 AND object_key=$2 AND status='active' AND ref_count>0)")
        .bind(provider_key.as_str())
        .bind(object_key)
        .fetch_one(&mut **transaction)
        .await
        .map(|row| row.0)
        .map_err(storage_error)
}

pub(super) async fn finalize(database: &Database, cleanup_id: &str, claim_token: &str) -> FileResult<()> {
    let now = OffsetDateTime::now_utc();
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    let row: Option<(String, String, Option<String>)> = query_as(
        "SELECT provider_key,cleanup_kind,object_key FROM file_provider_cleanup WHERE cleanup_id=$1 AND claim_token=$2 AND status='deleting' FOR UPDATE",
    )
    .bind(cleanup_id)
    .bind(claim_token)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(storage_error)?;
    let Some((provider_key, cleanup_kind, object_key)) = row else {
        return Err(FileError::NotFound);
    };
    if cleanup_kind == "object" {
        let Some(object_key) = object_key else {
            return Err(FileError::Infrastructure("object cleanup is missing its object key".into()));
        };
        query("DELETE FROM file_object WHERE provider_key=$1 AND object_key=$2 AND status='deleting' AND ref_count=0")
            .bind(provider_key)
            .bind(object_key)
            .execute(&mut *transaction)
            .await
            .map_err(storage_error)?;
    }
    let result = query("UPDATE file_provider_cleanup SET status='done',claim_token=NULL,claimed_at=NULL,updated_at=$3 WHERE cleanup_id=$1 AND claim_token=$2 AND status='deleting'")
        .bind(cleanup_id)
        .bind(claim_token)
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
    if result.rows_affected() == 0 {
        return Err(FileError::NotFound);
    }
    transaction.commit().await.map_err(storage_error)
}

pub(super) async fn release(database: &Database, cleanup_id: &str, claim_token: &str, error_code: &str) -> FileResult<()> {
    let now = OffsetDateTime::now_utc();
    let result = query("UPDATE file_provider_cleanup SET status='error',claim_token=NULL,claimed_at=NULL,last_error_code=$3,next_attempt_at=$4,updated_at=$4 WHERE cleanup_id=$1 AND claim_token=$2 AND status='deleting'")
        .bind(cleanup_id)
        .bind(claim_token)
        .bind(error_code)
        .bind(now + Duration::minutes(RETRY_DELAY_MINUTES))
        .execute(database.pool())
        .await
        .map_err(storage_error)?;
    if result.rows_affected() == 0 {
        return Err(FileError::NotFound);
    }
    Ok(())
}

fn validate_payload(kind: ProviderCleanupKind, object_key: Option<&ObjectKey>, upload_ref: Option<&ProviderUploadRef>) -> FileResult<()> {
    match kind {
        ProviderCleanupKind::Object if object_key.is_some() && upload_ref.is_none() => Ok(()),
        ProviderCleanupKind::Upload if upload_ref.is_some() => Ok(()),
        _ => Err(FileError::InvalidInput(keys::PROVIDER_CLEANUP_PAYLOAD_INVALID)),
    }
}

const fn kind_name(kind: ProviderCleanupKind) -> &'static str {
    match kind {
        ProviderCleanupKind::Object => "object",
        ProviderCleanupKind::Upload => "upload",
    }
}

fn parse_kind(value: &str) -> FileResult<ProviderCleanupKind> {
    match value {
        "object" => Ok(ProviderCleanupKind::Object),
        "upload" => Ok(ProviderCleanupKind::Upload),
        _ => Err(FileError::Infrastructure("invalid provider cleanup kind".into())),
    }
}
