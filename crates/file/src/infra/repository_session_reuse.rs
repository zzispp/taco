use sqlx::{Postgres, query, query_as};
use storage::Database;
use time::OffsetDateTime;

use crate::application::{BeginUploadSessionCommand, ExistingObject, FileAccessScope, FileEntryView, ProviderUploadRef};
use crate::domain::{ByteSize, FileId, UploadId};
use crate::{FileError, FileResult};

use super::repository_commands::ensure_visible_space;
use super::repository_queries::find_entry;
use super::repository_session_support::{map_insert, parent_value, release_reservation_tx};
use super::repository_support::{ensure_active_parent_tx, storage_error};

// Deduplicated sessions never call a Provider; this marker only satisfies the
// session audit record and is never sent to Provider I/O methods.
const REUSED_UPLOAD_REF_PREFIX: &str = "reused:";

struct ReuseContext {
    command: BeginUploadSessionCommand,
    object: ExistingObject,
    part_size: ByteSize,
    entry_id: FileId,
    session_id: UploadId,
    provider_upload_ref: ProviderUploadRef,
    size: i64,
    now: OffsetDateTime,
}

impl ReuseContext {
    fn new(command: BeginUploadSessionCommand, object: ExistingObject, part_size: ByteSize) -> FileResult<Self> {
        let session_id = UploadId::new();
        Ok(Self {
            size: i64::try_from(command.size.bytes()).map_err(|_| FileError::SizeMismatch)?,
            command,
            object,
            part_size,
            entry_id: FileId::new(),
            session_id,
            provider_upload_ref: ProviderUploadRef::new(format!("{REUSED_UPLOAD_REF_PREFIX}{session_id}"))?,
            now: OffsetDateTime::now_utc(),
        })
    }
}

pub(super) async fn create_reused_upload(
    database: &Database,
    actor: &FileAccessScope,
    command: BeginUploadSessionCommand,
    object: ExistingObject,
    part_size: ByteSize,
) -> FileResult<FileEntryView> {
    ensure_visible_space(database, actor, &command.space_id).await?;
    let context = ReuseContext::new(command, object, part_size)?;
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    ensure_active_parent_tx(&mut transaction, context.command.space_id.as_str(), context.command.parent_id).await?;
    if let Some(existing_id) = find_existing_entry(&mut transaction, &context).await? {
        release_reservation_tx(&mut transaction, &context.command.space_id, context.size, context.now).await?;
        insert_completed_session(&mut transaction, &context, existing_id).await?;
        transaction.commit().await.map_err(storage_error)?;
        return find_entry(database, actor, existing_id).await?.ok_or(FileError::NotFound);
    }
    ensure_object_exists(&mut transaction, &context.object).await?;
    insert_file_reference(&mut transaction, &context, actor).await?;
    insert_completed_session(&mut transaction, &context, context.entry_id).await?;
    transaction.commit().await.map_err(storage_error)?;
    find_entry(database, actor, context.entry_id).await?.ok_or(FileError::NotFound)
}

async fn find_existing_entry(transaction: &mut sqlx::Transaction<'_, Postgres>, context: &ReuseContext) -> FileResult<Option<FileId>> {
    let existing: Option<(String, String, i64)> = query_as(
        "SELECT e.entry_id,o.sha256,o.size_bytes FROM file_entry e JOIN file_object o ON o.object_id=e.object_id WHERE e.space_id=$1 AND COALESCE(e.parent_id,'')=COALESCE($2,'') AND e.normalized_name=$3 AND e.status='active' FOR UPDATE OF e",
    )
    .bind(context.command.space_id.as_str())
    .bind(parent_value(context.command.parent_id))
    .bind(context.command.name.normalized())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(storage_error)?;
    let Some((entry_id, digest, size)) = existing else {
        return Ok(None);
    };
    if digest != context.command.digest.to_hex() || size != context.size {
        return Err(FileError::NameConflict);
    }
    Ok(Some(FileId::parse(&entry_id)?))
}

async fn ensure_object_exists(transaction: &mut sqlx::Transaction<'_, Postgres>, object: &ExistingObject) -> FileResult<()> {
    let exists: Option<(String,)> = query_as("SELECT object_id FROM file_object WHERE object_id=$1 AND status='active' FOR UPDATE")
        .bind(object.object_id.to_string())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?;
    exists.map(|_| ()).ok_or(FileError::NotFound)
}

async fn insert_file_reference(transaction: &mut sqlx::Transaction<'_, Postgres>, context: &ReuseContext, actor: &FileAccessScope) -> FileResult<()> {
    query("INSERT INTO file_entry(entry_id,space_id,parent_id,kind,name,normalized_name,object_id,status,created_by,created_at,updated_by,updated_at) VALUES($1,$2,$3,'file',$4,$5,$6,'active',$7,$8,$7,$8)")
        .bind(context.entry_id.to_string())
        .bind(context.command.space_id.as_str())
        .bind(parent_value(context.command.parent_id))
        .bind(context.command.name.as_str())
        .bind(context.command.name.normalized())
        .bind(context.object.object_id.to_string())
        .bind(&actor.user_id)
        .bind(context.now)
        .execute(&mut **transaction)
        .await
        .map_err(map_insert)?;
    query("UPDATE file_object SET ref_count=ref_count+1,updated_at=$2 WHERE object_id=$1")
        .bind(context.object.object_id.to_string())
        .bind(context.now)
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)?;
    query("UPDATE file_space SET active_bytes=active_bytes+$2,reserved_bytes=GREATEST(reserved_bytes-$2,0),updated_at=$3 WHERE space_id=$1")
        .bind(context.command.space_id.as_str())
        .bind(context.size)
        .bind(context.now)
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)?;
    Ok(())
}

async fn insert_completed_session(transaction: &mut sqlx::Transaction<'_, Postgres>, context: &ReuseContext, entry_id: FileId) -> FileResult<()> {
    let part_size = i64::try_from(context.part_size.bytes()).map_err(|_| FileError::SizeMismatch)?;
    query("INSERT INTO file_upload_session(session_id,owner_user_id,space_id,parent_id,idempotency_key,file_name,normalized_name,declared_size_bytes,declared_sha256,content_type,part_size_bytes,provider_key,provider_upload_ref,provider_object_key,state,reserved_bytes,result_entry_id,created_at,last_activity_at,completed_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,'completed',0,$15,$16,$16,$16)")
        .bind(context.session_id.to_string())
        .bind(&context.command.actor_user_id)
        .bind(context.command.space_id.as_str())
        .bind(parent_value(context.command.parent_id))
        .bind(&context.command.idempotency_key)
        .bind(context.command.name.as_str())
        .bind(context.command.name.normalized())
        .bind(context.size)
        .bind(context.command.digest.to_hex())
        .bind(&context.command.content_type)
        .bind(part_size)
        .bind(context.object.provider_key.as_str())
        .bind(context.provider_upload_ref.as_str())
        .bind(context.object.object_key.as_str())
        .bind(entry_id.to_string())
        .bind(context.now)
        .execute(&mut **transaction)
        .await
        .map_err(map_insert)?;
    Ok(())
}
