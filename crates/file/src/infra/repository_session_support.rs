use sqlx::{Postgres, query, query_as};
use storage::Database;
use time::OffsetDateTime;

use crate::application::{PartReceipt, ProviderPartRef, ProviderUploadRef, UploadSessionData};
use crate::domain::{ByteSize, ContentDigest, DirectoryId, EntryName, FileId, PartNumber, ProviderKey, SpaceId, UploadId};
use crate::{FileError, FileResult};

type CompletedPartRow = (i64, i64, String, Option<String>);

#[derive(sqlx::FromRow)]
pub(super) struct SessionRecord {
    pub(super) session_id: String,
    pub(super) owner_user_id: String,
    pub(super) space_id: String,
    pub(super) parent_id: Option<String>,
    pub(super) file_name: String,
    pub(super) declared_size_bytes: i64,
    pub(super) declared_sha256: Option<String>,
    pub(super) content_type: String,
    pub(super) part_size_bytes: i64,
    pub(super) provider_key: String,
    pub(super) provider_upload_ref: String,
    pub(super) provider_object_key: String,
    pub(super) state: String,
    pub(super) result_entry_id: Option<String>,
}

pub(super) fn session_columns() -> &'static str {
    "us.session_id,us.owner_user_id,us.space_id,us.parent_id,us.file_name,us.declared_size_bytes,us.declared_sha256,us.content_type,us.part_size_bytes,us.provider_key,us.provider_upload_ref,us.provider_object_key,us.state,us.result_entry_id"
}

pub(super) fn session_data(record: SessionRecord) -> FileResult<UploadSessionData> {
    Ok(UploadSessionData {
        id: UploadId::parse(&record.session_id)?,
        owner_user_id: record.owner_user_id,
        space_id: SpaceId::new(record.space_id)?,
        parent_id: record
            .parent_id
            .map(|value| DirectoryId::parse(&value))
            .transpose()?
            .unwrap_or(DirectoryId::ROOT),
        name: EntryName::new(record.file_name)?,
        size: ByteSize::from_bytes(u64::try_from(record.declared_size_bytes).map_err(|_| FileError::SizeMismatch)?),
        digest: ContentDigest::from_hex(record.declared_sha256.as_deref().ok_or(FileError::DigestMismatch)?)?,
        content_type: record.content_type,
        part_size: ByteSize::from_bytes(u64::try_from(record.part_size_bytes).map_err(|_| FileError::SizeMismatch)?),
        provider_key: ProviderKey::new(record.provider_key)?,
        provider_upload_ref: ProviderUploadRef::new(record.provider_upload_ref)?,
        provider_object_key: crate::application::ObjectKey::new(record.provider_object_key)?,
        state: record.state,
        result_entry_id: record.result_entry_id.map(|value| FileId::parse(&value)).transpose()?,
    })
}

pub(super) async fn session_with_parts(database: &Database, record: Option<SessionRecord>) -> FileResult<Option<(UploadSessionData, Vec<PartReceipt>)>> {
    let Some(record) = record else { return Ok(None) };
    let data = session_data(record)?;
    let rows: Vec<CompletedPartRow> =
        query_as("SELECT part_number,size_bytes,sha256,provider_part_ref FROM file_upload_part WHERE session_id=$1 AND state='completed' ORDER BY part_number")
            .bind(data.id.to_string())
            .fetch_all(database.pool())
            .await
            .map_err(super::repository_support::storage_error)?;
    let parts = part_receipts(data.id, rows)?;
    Ok(Some((data, parts)))
}

pub(super) async fn parts_for_transaction(transaction: &mut sqlx::Transaction<'_, Postgres>, session_id: UploadId) -> FileResult<Vec<PartReceipt>> {
    let rows: Vec<CompletedPartRow> =
        query_as("SELECT part_number,size_bytes,sha256,provider_part_ref FROM file_upload_part WHERE session_id=$1 AND state='completed' ORDER BY part_number")
            .bind(session_id.to_string())
            .fetch_all(&mut **transaction)
            .await
            .map_err(super::repository_support::storage_error)?;
    part_receipts(session_id, rows)
}

pub(super) fn validate_completion_parts(session: &UploadSessionData, parts: &[PartReceipt]) -> FileResult<()> {
    if parts.is_empty() {
        return Err(FileError::UploadIncomplete);
    }
    let mut total = 0_u64;
    for (index, part) in parts.iter().enumerate() {
        let number = u32::try_from(index + 1).map_err(|_| FileError::InvalidPart)?;
        if part.part_number.value() != number {
            return Err(FileError::UploadIncomplete);
        }
        let offset = u64::from(number - 1).checked_mul(session.part_size.bytes()).ok_or(FileError::SizeMismatch)?;
        let expected = session
            .size
            .bytes()
            .checked_sub(offset)
            .ok_or(FileError::SizeMismatch)?
            .min(session.part_size.bytes());
        if part.size.bytes() != expected {
            return Err(FileError::SizeMismatch);
        }
        total = total.checked_add(part.size.bytes()).ok_or(FileError::SizeMismatch)?;
    }
    if total != session.size.bytes() {
        return Err(FileError::UploadIncomplete);
    }
    Ok(())
}

fn part_receipts(session_id: UploadId, rows: Vec<CompletedPartRow>) -> FileResult<Vec<PartReceipt>> {
    rows.into_iter()
        .map(|(number, size, digest, provider_part_ref)| {
            Ok(PartReceipt {
                session_id,
                part_number: PartNumber::new(u32::try_from(number).map_err(|_| FileError::InvalidPart)?)?,
                provider_part_ref: ProviderPartRef::new(provider_part_ref.ok_or(FileError::UploadIncomplete)?)?,
                size: ByteSize::from_bytes(u64::try_from(size).map_err(|_| FileError::SizeMismatch)?),
                digest: ContentDigest::from_hex(&digest)?,
            })
        })
        .collect()
}

pub(super) async fn release_reservation_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    space_id: &SpaceId,
    size: i64,
    now: OffsetDateTime,
) -> FileResult<()> {
    query("UPDATE file_space SET reserved_bytes=GREATEST(reserved_bytes-$2,0),updated_at=$3 WHERE space_id=$1")
        .bind(space_id.as_str())
        .bind(size)
        .bind(now)
        .execute(&mut **transaction)
        .await
        .map_err(super::repository_support::storage_error)?;
    Ok(())
}

pub(super) fn parent_value(parent_id: DirectoryId) -> Option<String> {
    (parent_id != DirectoryId::ROOT).then(|| parent_id.to_string())
}

pub(super) fn map_insert(error: sqlx::Error) -> FileError {
    let message = error.to_string();
    if message.contains("idx_file_upload_intent") || message.contains("idx_file_entry_active_sibling_name") {
        FileError::NameConflict
    } else {
        super::repository_support::storage_error(error)
    }
}
