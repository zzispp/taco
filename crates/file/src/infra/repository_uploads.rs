use sqlx::Postgres;
use sqlx::{query, query_as};
use storage::Database;
use time::OffsetDateTime;

use crate::application::StoredObject;
use crate::application::{ExistingObject, FileAccessScope, FileEntryView, UploadCommand};
use crate::domain::{FileId, SpaceId};
use crate::{FileError, FileResult};

use super::repository_commands::ensure_visible_space;
use super::repository_provider_cleanup::{cancel_object_cleanup_tx, record_tx};
use super::repository_queries::find_entry;
use super::repository_support::{ensure_active_parent_tx, same_physical_object};

pub(super) async fn reserve_upload(
    database: &Database,
    space_id: SpaceId,
    bytes: crate::domain::ByteSize,
    default_quota: crate::domain::ByteSize,
) -> FileResult<()> {
    let bytes = i64::try_from(bytes.bytes()).map_err(|_| FileError::SizeMismatch)?;
    let result = query(
        "UPDATE file_space SET reserved_bytes=reserved_bytes+$2,updated_at=$4 WHERE space_id=$1 AND active_bytes+trashed_bytes+reserved_bytes+$2<=COALESCE(quota_override_bytes,$3)",
    )
    .bind(space_id.as_str())
    .bind(bytes)
    .bind(i64::try_from(default_quota.bytes()).map_err(|_| FileError::SizeMismatch)?)
    .bind(OffsetDateTime::now_utc())
    .execute(database.pool())
    .await
    .map_err(super::repository_support::storage_error)?;
    if result.rows_affected() == 0 {
        let quota: Option<(i64, i64, i64, Option<i64>)> =
            query_as("SELECT active_bytes,trashed_bytes,reserved_bytes,quota_override_bytes FROM file_space WHERE space_id=$1")
                .bind(space_id.as_str())
                .fetch_optional(database.pool())
                .await
                .map_err(super::repository_support::storage_error)?;
        let Some((active, trashed, reserved, limit)) = quota else {
            return Err(FileError::NotFound);
        };
        let default_limit = i64::try_from(default_quota.bytes()).map_err(|_| FileError::SizeMismatch)?;
        let remaining = (limit.unwrap_or(default_limit) - active - trashed - reserved).max(0) as u64;
        return Err(FileError::QuotaExceeded {
            requested: crate::domain::ByteSize::from_bytes(bytes as u64),
            remaining: crate::domain::ByteSize::from_bytes(remaining),
        });
    }
    Ok(())
}

pub(super) async fn release_upload(database: &Database, space_id: SpaceId, bytes: crate::domain::ByteSize) -> FileResult<()> {
    let bytes = i64::try_from(bytes.bytes()).map_err(|_| FileError::SizeMismatch)?;
    query("UPDATE file_space SET reserved_bytes=GREATEST(reserved_bytes-$2,0),updated_at=$3 WHERE space_id=$1")
        .bind(space_id.as_str())
        .bind(bytes)
        .bind(OffsetDateTime::now_utc())
        .execute(database.pool())
        .await
        .map_err(super::repository_support::storage_error)?;
    Ok(())
}

pub(super) async fn create_uploaded_file(
    database: &Database,
    actor: &FileAccessScope,
    command: UploadCommand,
    object: StoredObject,
) -> FileResult<FileEntryView> {
    ensure_visible_space(database, actor, &command.space_id).await?;
    let digest = object.digest.ok_or(FileError::DigestMismatch)?;
    let size = i64::try_from(object.size.bytes()).map_err(|_| FileError::SizeMismatch)?;
    let content_type = command.content_type.clone();
    let id = FileId::new();
    let now = OffsetDateTime::now_utc();
    let mut transaction = database.pool().begin().await.map_err(super::repository_support::storage_error)?;
    ensure_active_parent_tx(&mut transaction, command.space_id.as_str(), command.parent_id).await?;
    let object_id = adopt_uploaded_object(
        &mut transaction,
        UploadedObjectContext {
            object: &object,
            digest,
            size,
            content_type: &content_type,
            now,
        },
    )
    .await?;
    let result = query("INSERT INTO file_entry(entry_id,space_id,parent_id,kind,name,normalized_name,object_id,status,created_by,created_at,updated_by,updated_at) VALUES($1,$2,$3,'file',$4,$5,$6,'active',$7,$8,$7,$8)")
        .bind(id.to_string()).bind(command.space_id.as_str()).bind(parent_value(command.parent_id)).bind(command.name.as_str()).bind(command.name.normalized()).bind(object_id).bind(&command.actor_user_id).bind(now).execute(&mut *transaction).await;
    map_write(result)?;
    query("UPDATE file_space SET active_bytes=active_bytes+$2,reserved_bytes=GREATEST(reserved_bytes-$2,0),updated_at=$3 WHERE space_id=$1")
        .bind(command.space_id.as_str())
        .bind(size)
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(super::repository_support::storage_error)?;
    transaction.commit().await.map_err(super::repository_support::storage_error)?;
    find_entry(database, actor, id).await?.ok_or(FileError::NotFound)
}

struct UploadedObjectContext<'a> {
    object: &'a StoredObject,
    digest: crate::domain::ContentDigest,
    size: i64,
    content_type: &'a str,
    now: OffsetDateTime,
}

async fn adopt_uploaded_object(transaction: &mut sqlx::Transaction<'_, Postgres>, context: UploadedObjectContext<'_>) -> FileResult<String> {
    let UploadedObjectContext {
        object,
        digest,
        size,
        content_type,
        now,
    } = context;
    cancel_object_cleanup_tx(transaction, &object.provider_key, &object.key).await?;
    let inserted = query("INSERT INTO file_object(object_id,provider_key,object_key,size_bytes,sha256,content_type,ref_count,status,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,1,'active',$7,$7) ON CONFLICT DO NOTHING")
        .bind(object.id.to_string()).bind(object.provider_key.as_str()).bind(object.key.as_str()).bind(size).bind(digest.to_hex()).bind(content_type).bind(now).execute(&mut **transaction).await.map_err(super::repository_support::storage_error)?;
    if inserted.rows_affected() == 1 {
        return Ok(object.id.to_string());
    }
    let canonical: Option<(String, String, String)> =
        query_as("SELECT object_id,provider_key,object_key FROM file_object WHERE sha256=$1 AND size_bytes=$2 AND status='active' FOR UPDATE")
            .bind(digest.to_hex())
            .bind(size)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(super::repository_support::storage_error)?;
    let Some((object_id, provider_key, object_key)) = canonical else {
        return Err(FileError::Infrastructure("content deduplication object disappeared during upload".into()));
    };
    query("UPDATE file_object SET ref_count=ref_count+1,updated_at=$2 WHERE object_id=$1")
        .bind(&object_id)
        .bind(now)
        .execute(&mut **transaction)
        .await
        .map_err(super::repository_support::storage_error)?;
    let adopted = same_physical_object(&provider_key, &object_key, object);
    if !adopted {
        record_tx(
            transaction,
            &object.provider_key,
            crate::application::ProviderCleanupKind::Object,
            Some(&object.key),
            None,
        )
        .await?;
    }
    Ok(object_id)
}

pub(super) async fn create_reused_file(
    database: &Database,
    actor: &FileAccessScope,
    command: UploadCommand,
    object: ExistingObject,
) -> FileResult<FileEntryView> {
    ensure_visible_space(database, actor, &command.space_id).await?;
    let id = FileId::new();
    let now = OffsetDateTime::now_utc();
    let mut transaction = database.pool().begin().await.map_err(super::repository_support::storage_error)?;
    ensure_active_parent_tx(&mut transaction, command.space_id.as_str(), command.parent_id).await?;
    let object_exists = query_as::<_, (String,)>("SELECT object_id FROM file_object WHERE object_id=$1 AND status='active' FOR UPDATE")
        .bind(object.object_id.to_string())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(super::repository_support::storage_error)?;
    if object_exists.is_none() {
        return Err(FileError::NotFound);
    }
    let result = query("INSERT INTO file_entry(entry_id,space_id,parent_id,kind,name,normalized_name,object_id,status,created_by,created_at,updated_by,updated_at) VALUES($1,$2,$3,'file',$4,$5,$6,'active',$7,$8,$7,$8)")
        .bind(id.to_string()).bind(command.space_id.as_str()).bind(parent_value(command.parent_id)).bind(command.name.as_str()).bind(command.name.normalized()).bind(object.object_id.to_string()).bind(&command.actor_user_id).bind(now).execute(&mut *transaction).await;
    map_write(result)?;
    query("UPDATE file_object SET ref_count=ref_count+1,updated_at=$2 WHERE object_id=$1")
        .bind(object.object_id.to_string())
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(super::repository_support::storage_error)?;
    let size = i64::try_from(object.size.bytes()).map_err(|_| FileError::SizeMismatch)?;
    query("UPDATE file_space SET active_bytes=active_bytes+$2,reserved_bytes=GREATEST(reserved_bytes-$2,0),updated_at=$3 WHERE space_id=$1")
        .bind(command.space_id.as_str())
        .bind(size)
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(super::repository_support::storage_error)?;
    transaction.commit().await.map_err(super::repository_support::storage_error)?;
    find_entry(database, actor, id).await?.ok_or(FileError::NotFound)
}

fn parent_value(parent_id: crate::domain::DirectoryId) -> Option<String> {
    (parent_id != crate::domain::DirectoryId::ROOT).then(|| parent_id.to_string())
}

fn map_write(result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>) -> FileResult<()> {
    result.map(|_| ()).map_err(map_sql_error)
}

fn map_sql_error(error: sqlx::Error) -> FileError {
    if error.to_string().contains("idx_file_entry_active_sibling_name") {
        FileError::NameConflict
    } else {
        super::repository_support::storage_error(error)
    }
}
