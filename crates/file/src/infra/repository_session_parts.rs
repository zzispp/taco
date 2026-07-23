use sqlx::{Postgres, QueryBuilder, query, query_as};
use storage::Database;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::application::{FileAccessScope, PartReceipt, ProviderPartRef, UploadPartClaimRequest, UploadPartClaimResult, UploadSessionData};
use crate::domain::{PartNumber, UploadId};
use crate::{FileError, FileResult};

use super::repository_session_support::{SessionRecord, parts_for_transaction, session_columns, session_data, validate_completion_parts};
use super::repository_support::{ensure_active_parent_tx, scope_query, storage_error};

const UPLOAD_PART_CLAIM_LEASE_MINUTES: i64 = 30;
type UploadPartRow = (i64, String, String, Option<String>, Option<OffsetDateTime>, Option<String>);

struct ValidatedPartClaim {
    request: UploadPartClaimRequest,
    expected_size: i64,
}

pub(super) async fn claim_upload_part(database: &Database, actor: &FileAccessScope, request: UploadPartClaimRequest) -> FileResult<UploadPartClaimResult> {
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    let claim = validate_part_claim(&mut transaction, actor, request).await?;
    let existing = load_part(&mut transaction, &claim).await?;
    let result = match existing {
        Some(row) => claim_existing_part(&mut transaction, &claim, row).await?,
        None => insert_new_part(&mut transaction, &claim).await?,
    };
    transaction.commit().await.map_err(storage_error)?;
    Ok(result)
}

async fn validate_part_claim(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    actor: &FileAccessScope,
    request: UploadPartClaimRequest,
) -> FileResult<ValidatedPartClaim> {
    let session: Option<(String, i64, i64)> = query_as(
        "SELECT session_id,declared_size_bytes,part_size_bytes FROM file_upload_session WHERE session_id=$1 AND owner_user_id=$2 AND state='open' AND cleanup_claim_token IS NULL FOR UPDATE",
    )
    .bind(request.session_id.to_string())
    .bind(&actor.user_id)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?;
    let Some((_, declared_size, part_size)) = session else {
        return Err(FileError::UploadNotFound);
    };
    let expected_size = i64::try_from(request.expected_size.bytes()).map_err(|_| FileError::SizeMismatch)?;
    if expected_size <= 0 || part_size <= 0 {
        return Err(FileError::InvalidPart);
    }
    let offset = i64::from(request.part_number.value() - 1)
        .checked_mul(part_size)
        .ok_or(FileError::SizeMismatch)?;
    if offset >= declared_size || expected_size != (declared_size - offset).min(part_size) {
        return Err(FileError::InvalidPart);
    }
    Ok(ValidatedPartClaim { request, expected_size })
}

async fn load_part(transaction: &mut sqlx::Transaction<'_, Postgres>, claim: &ValidatedPartClaim) -> FileResult<Option<UploadPartRow>> {
    query_as("SELECT size_bytes,sha256,state,claim_token,claimed_at,provider_part_ref FROM file_upload_part WHERE session_id=$1 AND part_number=$2 FOR UPDATE")
        .bind(claim.request.session_id.to_string())
        .bind(i64::from(claim.request.part_number.value()))
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)
}

async fn claim_existing_part(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    claim: &ValidatedPartClaim,
    row: UploadPartRow,
) -> FileResult<UploadPartClaimResult> {
    let (size, digest, state, _, claimed_at, provider_part_ref) = row;
    if size != claim.expected_size || digest != claim.request.digest.to_hex() {
        return Err(FileError::UploadPartConflict);
    }
    if state == "completed" {
        return Ok(UploadPartClaimResult::Completed(PartReceipt {
            session_id: claim.request.session_id,
            part_number: claim.request.part_number,
            provider_part_ref: ProviderPartRef::new(provider_part_ref.ok_or(FileError::UploadIncomplete)?)?,
            size: claim.request.expected_size,
            digest: claim.request.digest,
        }));
    }
    let lease_cutoff = OffsetDateTime::now_utc() - Duration::minutes(UPLOAD_PART_CLAIM_LEASE_MINUTES);
    if claimed_at.is_some_and(|value| value > lease_cutoff) {
        return Err(FileError::UploadPartConflict);
    }
    let token = Uuid::now_v7().to_string();
    let now = OffsetDateTime::now_utc();
    query("UPDATE file_upload_part SET claim_token=$3,claimed_at=$4,state='writing',created_at=$4 WHERE session_id=$1 AND part_number=$2")
        .bind(claim.request.session_id.to_string())
        .bind(i64::from(claim.request.part_number.value()))
        .bind(&token)
        .bind(now)
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)?;
    touch_session(transaction, claim.request.session_id, now).await?;
    Ok(UploadPartClaimResult::Claimed { token })
}

async fn insert_new_part(transaction: &mut sqlx::Transaction<'_, Postgres>, claim: &ValidatedPartClaim) -> FileResult<UploadPartClaimResult> {
    let token = Uuid::now_v7().to_string();
    let now = OffsetDateTime::now_utc();
    query("INSERT INTO file_upload_part(session_id,part_number,size_bytes,sha256,provider_part_ref,state,claim_token,claimed_at,created_at) VALUES($1,$2,$3,$4,NULL,'writing',$5,$6,$6)")
        .bind(claim.request.session_id.to_string())
        .bind(i64::from(claim.request.part_number.value()))
        .bind(claim.expected_size)
        .bind(claim.request.digest.to_hex())
        .bind(&token)
        .bind(now)
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)?;
    touch_session(transaction, claim.request.session_id, now).await?;
    Ok(UploadPartClaimResult::Claimed { token })
}

async fn touch_session(transaction: &mut sqlx::Transaction<'_, Postgres>, session_id: crate::domain::UploadId, now: OffsetDateTime) -> FileResult<()> {
    query("UPDATE file_upload_session SET last_activity_at=$2 WHERE session_id=$1")
        .bind(session_id.to_string())
        .bind(now)
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)?;
    Ok(())
}

pub(super) async fn complete_upload_part(database: &Database, actor: &FileAccessScope, receipt: PartReceipt, claim_token: &str) -> FileResult<()> {
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    let session: Option<(String,)> = query_as(
        "SELECT session_id FROM file_upload_session WHERE session_id=$1 AND owner_user_id=$2 AND state='open' AND cleanup_claim_token IS NULL FOR UPDATE",
    )
    .bind(receipt.session_id.to_string())
    .bind(&actor.user_id)
    .fetch_optional(&mut *transaction)
    .await
    .map_err(storage_error)?;
    if session.is_none() {
        return Err(FileError::UploadNotFound);
    }
    let result = query("UPDATE file_upload_part SET state='completed',claim_token=NULL,claimed_at=NULL,provider_part_ref=$4,created_at=$5 WHERE session_id=$1 AND part_number=$2 AND claim_token=$3 AND size_bytes=$6 AND sha256=$7")
        .bind(receipt.session_id.to_string()).bind(i64::from(receipt.part_number.value())).bind(claim_token).bind(receipt.provider_part_ref.as_str()).bind(OffsetDateTime::now_utc()).bind(i64::try_from(receipt.size.bytes()).map_err(|_| FileError::SizeMismatch)?).bind(receipt.digest.to_hex())
        .execute(&mut *transaction).await.map_err(storage_error)?;
    if result.rows_affected() == 0 {
        return Err(FileError::UploadPartConflict);
    }
    query("UPDATE file_upload_session SET last_activity_at=$2 WHERE session_id=$1 AND owner_user_id=$3 AND state='open' AND cleanup_claim_token IS NULL")
        .bind(receipt.session_id.to_string())
        .bind(OffsetDateTime::now_utc())
        .bind(&actor.user_id)
        .execute(&mut *transaction)
        .await
        .map_err(storage_error)?;
    transaction.commit().await.map_err(storage_error)
}

pub(super) async fn release_upload_part_claim(database: &Database, session_id: UploadId, part_number: PartNumber, claim_token: &str) -> FileResult<()> {
    let result = query("DELETE FROM file_upload_part WHERE session_id=$1 AND part_number=$2 AND state='writing' AND claim_token=$3")
        .bind(session_id.to_string())
        .bind(i64::from(part_number.value()))
        .bind(claim_token)
        .execute(database.pool())
        .await
        .map_err(storage_error)?;
    if result.rows_affected() == 0 {
        return Err(FileError::UploadPartConflict);
    }
    Ok(())
}

pub(super) async fn begin_upload_completion(
    database: &Database,
    actor: &FileAccessScope,
    session_id: UploadId,
) -> FileResult<(UploadSessionData, Vec<PartReceipt>)> {
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    let session = lock_open_completion_session(&mut transaction, actor, session_id).await?;
    ensure_active_parent_tx(&mut transaction, session.space_id.as_str(), session.parent_id).await?;
    let parts = validated_completion_parts(&mut transaction, &session).await?;
    mark_completion_started(&mut transaction, session_id).await?;
    transaction.commit().await.map_err(storage_error)?;
    Ok((
        UploadSessionData {
            state: "completing".into(),
            ..session
        },
        parts,
    ))
}

async fn lock_open_completion_session(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    actor: &FileAccessScope,
    session_id: UploadId,
) -> FileResult<UploadSessionData> {
    let mut select = QueryBuilder::<Postgres>::new(format!(
        "SELECT {} FROM file_upload_session us JOIN file_space s ON s.space_id=us.space_id WHERE us.session_id=",
        session_columns()
    ));
    select.push_bind(session_id.to_string()).push(" AND");
    scope_query(&mut select, actor, "s");
    select.push(" AND us.cleanup_claim_token IS NULL FOR UPDATE OF us");
    let record = select
        .build_query_as::<SessionRecord>()
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?
        .ok_or(FileError::UploadNotFound)?;
    let session = session_data(record)?;
    if session.owner_user_id != actor.user_id {
        return Err(FileError::UploadNotFound);
    }
    if session.state == "completing" || session.state == "completed" {
        return Err(FileError::UploadCompletionInProgress);
    }
    if session.state != "open" {
        return Err(FileError::UploadNotFound);
    }
    Ok(session)
}

async fn validated_completion_parts(transaction: &mut sqlx::Transaction<'_, Postgres>, session: &UploadSessionData) -> FileResult<Vec<PartReceipt>> {
    let writing: (i64,) = query_as("SELECT COUNT(*) FROM file_upload_part WHERE session_id=$1 AND state='writing'")
        .bind(session.id.to_string())
        .fetch_one(&mut **transaction)
        .await
        .map_err(storage_error)?;
    if writing.0 > 0 {
        return Err(FileError::UploadPartConflict);
    }
    let parts = parts_for_transaction(transaction, session.id).await?;
    validate_completion_parts(session, &parts)?;
    Ok(parts)
}

async fn mark_completion_started(transaction: &mut sqlx::Transaction<'_, Postgres>, session_id: UploadId) -> FileResult<()> {
    let result =
        query("UPDATE file_upload_session SET state='completing',last_activity_at=$2 WHERE session_id=$1 AND state='open' AND cleanup_claim_token IS NULL")
            .bind(session_id.to_string())
            .bind(OffsetDateTime::now_utc())
            .execute(&mut **transaction)
            .await
            .map_err(storage_error)?;
    if result.rows_affected() == 0 {
        return Err(FileError::UploadCompletionInProgress);
    }
    Ok(())
}

pub(super) async fn reopen_upload_completion(database: &Database, actor: &FileAccessScope, session_id: UploadId) -> FileResult<()> {
    let result = query("UPDATE file_upload_session SET state='open',last_activity_at=$3 WHERE session_id=$1 AND owner_user_id=$2 AND state='completing' AND cleanup_claim_token IS NULL")
        .bind(session_id.to_string()).bind(&actor.user_id).bind(OffsetDateTime::now_utc()).execute(database.pool()).await.map_err(storage_error)?;
    if result.rows_affected() == 0 {
        return Err(FileError::UploadNotFound);
    }
    Ok(())
}
