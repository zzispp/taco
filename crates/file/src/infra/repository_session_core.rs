use sqlx::{Postgres, QueryBuilder, query};
use storage::Database;
use time::OffsetDateTime;

use crate::FileResult;
use crate::application::{BeginUploadSessionCommand, FileAccessScope, UploadSession, UploadSessionData};
use crate::domain::{SpaceId, UploadId};

use super::repository_commands::ensure_visible_space;
use super::repository_session_support::{SessionRecord, parent_value, session_columns, session_with_parts};
use super::repository_support::{ensure_active_parent_tx, scope_query, storage_error};

pub(super) async fn find_upload_intent(
    database: &Database,
    actor: &FileAccessScope,
    owner_user_id: &str,
    space_id: SpaceId,
    idempotency_key: &str,
) -> FileResult<Option<(UploadSessionData, Vec<crate::application::PartReceipt>)>> {
    let mut query = QueryBuilder::<Postgres>::new(format!(
        "SELECT {} FROM file_upload_session us JOIN file_space s ON s.space_id=us.space_id WHERE us.owner_user_id=",
        session_columns()
    ));
    query
        .push_bind(owner_user_id.to_owned())
        .push(" AND us.space_id=")
        .push_bind(space_id.as_str().to_owned());
    query.push(" AND us.idempotency_key=").push_bind(idempotency_key.to_owned()).push(" AND");
    scope_query(&mut query, actor, "s");
    let record = query
        .build_query_as::<SessionRecord>()
        .fetch_optional(database.pool())
        .await
        .map_err(storage_error)?;
    session_with_parts(database, record).await
}

pub(super) async fn create_upload_session(
    database: &Database,
    actor: &FileAccessScope,
    command: BeginUploadSessionCommand,
    provider_session: UploadSession,
) -> FileResult<UploadSessionData> {
    ensure_visible_space(database, actor, &command.space_id).await?;
    let now = OffsetDateTime::now_utc();
    let size = i64::try_from(command.size.bytes()).map_err(|_| crate::FileError::SizeMismatch)?;
    let part_size = i64::try_from(provider_session.part_size.bytes()).map_err(|_| crate::FileError::SizeMismatch)?;
    let session_id = UploadId::new();
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    ensure_active_parent_tx(&mut transaction, command.space_id.as_str(), command.parent_id).await?;
    query("INSERT INTO file_upload_session(session_id,owner_user_id,space_id,parent_id,idempotency_key,file_name,normalized_name,declared_size_bytes,declared_sha256,content_type,part_size_bytes,provider_key,provider_upload_ref,provider_object_key,state,reserved_bytes,created_at,last_activity_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,'open',$8,$15,$15)")
        .bind(session_id.to_string()).bind(&command.actor_user_id).bind(command.space_id.as_str()).bind(parent_value(command.parent_id)).bind(&command.idempotency_key)
        .bind(command.name.as_str()).bind(command.name.normalized()).bind(size).bind(command.digest.to_hex()).bind(&command.content_type).bind(part_size)
        .bind(provider_session.provider_key.as_str()).bind(provider_session.provider_upload_ref.as_str()).bind(provider_session.key.as_str()).bind(now)
        .execute(&mut *transaction).await.map_err(super::repository_session_support::map_insert)?;
    transaction.commit().await.map_err(storage_error)?;
    Ok(UploadSessionData {
        id: session_id,
        owner_user_id: command.actor_user_id,
        space_id: command.space_id,
        parent_id: command.parent_id,
        name: command.name,
        size: command.size,
        digest: command.digest,
        content_type: command.content_type,
        part_size: provider_session.part_size,
        provider_key: provider_session.provider_key,
        provider_upload_ref: provider_session.provider_upload_ref,
        provider_object_key: provider_session.key,
        state: "open".into(),
        result_entry_id: None,
    })
}

pub(super) async fn get_upload_session(
    database: &Database,
    actor: &FileAccessScope,
    session_id: crate::domain::UploadId,
) -> FileResult<Option<(UploadSessionData, Vec<crate::application::PartReceipt>)>> {
    let mut query = QueryBuilder::<Postgres>::new(format!(
        "SELECT {} FROM file_upload_session us JOIN file_space s ON s.space_id=us.space_id WHERE us.session_id=",
        session_columns()
    ));
    query.push_bind(session_id.to_string()).push(" AND");
    scope_query(&mut query, actor, "s");
    let record = query
        .build_query_as::<SessionRecord>()
        .fetch_optional(database.pool())
        .await
        .map_err(storage_error)?;
    session_with_parts(database, record).await
}
