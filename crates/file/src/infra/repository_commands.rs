use sqlx::{Postgres, QueryBuilder, query, query_as};
use storage::Database;
use time::OffsetDateTime;

use crate::application::{CreateFolderCommand, FileAccessScope, FileEntryView, UpdateEntryCommand, UpdateSpaceCommand};
use crate::domain::{DirectoryId, EntryName, FileId, SpaceId, TagName};
use crate::error::keys;
use crate::{FileError, FileResult};

use super::repository_queries::{find_entry, list_spaces, resolve_visible_space};
use super::repository_support::{scope_query, storage_error};
use super::repository_system_folders::ensure_space;

pub(super) use super::repository_purge::purge;
pub(super) use super::repository_trash::{restore, trash};

pub(super) async fn create_folder(database: &Database, actor: &FileAccessScope, command: CreateFolderCommand) -> FileResult<FileEntryView> {
    ensure_target_space(database, actor, &command.space_id).await?;
    let id = FileId::new();
    let now = OffsetDateTime::now_utc();
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    super::repository_support::ensure_active_parent_tx(&mut transaction, command.space_id.as_str(), command.parent_id).await?;
    let result = query(
        "INSERT INTO file_entry(entry_id,space_id,parent_id,kind,name,normalized_name,object_id,status,created_by,created_at,updated_by,updated_at) VALUES($1,$2,$3,'folder',$4,$5,NULL,'active',$6,$7,$6,$7)",
    )
    .bind(id.to_string())
    .bind(command.space_id.as_str())
    .bind(parent_value(command.parent_id))
    .bind(command.name.as_str())
    .bind(command.name.normalized())
    .bind(&command.actor_user_id)
    .bind(now)
    .execute(&mut *transaction)
    .await;
    map_write(result)?;
    transaction.commit().await.map_err(storage_error)?;
    find_entry(database, actor, id).await?.ok_or(FileError::NotFound)
}

pub(super) async fn update_entry(database: &Database, actor: &FileAccessScope, command: UpdateEntryCommand) -> FileResult<FileEntryView> {
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    let mut lookup = QueryBuilder::<Postgres>::new(
        "SELECT e.entry_id,e.space_id,e.parent_id,e.kind,e.name,e.system_kind FROM file_entry e JOIN file_space s ON s.space_id=e.space_id WHERE e.entry_id=",
    );
    lookup.push_bind(command.id.to_string()).push(" AND e.status='active' AND");
    scope_query(&mut lookup, actor, "s");
    lookup.push(" FOR UPDATE OF e");
    let current = lookup
        .build_query_as::<(String, String, Option<String>, String, String, Option<String>)>()
        .fetch_optional(&mut *transaction)
        .await
        .map_err(storage_error)?
        .ok_or(FileError::NotFound)?;
    if current.5.is_some() && (command.name.is_some() || command.parent_id.is_some()) {
        return Err(FileError::InvalidInput(keys::SYSTEM_FOLDER_IMMUTABLE));
    }
    if let Some(parent_id) = command.parent_id {
        ensure_parent_tx(&mut transaction, &current.1, parent_id, command.id).await?;
    }
    let next_name = command.name.as_ref().map_or(current.4.as_str(), EntryName::as_str);
    let normalized = command
        .name
        .as_ref()
        .map_or_else(|| current.4.to_lowercase(), |name| name.normalized().to_owned());
    let parent = command.parent_id.map(parent_value).unwrap_or(current.2.clone());
    let now = OffsetDateTime::now_utc();
    let result = query("UPDATE file_entry SET name=$2,normalized_name=$3,parent_id=$4,updated_by=$5,updated_at=$6 WHERE entry_id=$1 AND status='active'")
        .bind(command.id.to_string())
        .bind(next_name)
        .bind(normalized)
        .bind(parent)
        .bind(&command.actor_user_id)
        .bind(now)
        .execute(&mut *transaction)
        .await;
    map_write(result)?;
    if let Some(tags) = command.tags {
        replace_tags(&mut transaction, &current.1, command.id, tags, &command.actor_user_id).await?;
    }
    transaction.commit().await.map_err(storage_error)?;
    find_entry(database, actor, command.id).await?.ok_or(FileError::NotFound)
}

pub(super) async fn update_space(
    database: &Database,
    actor: &FileAccessScope,
    space_id: SpaceId,
    command: UpdateSpaceCommand,
    default_quota: crate::domain::ByteSize,
) -> FileResult<crate::application::FileSpaceView> {
    let Some((resolved_id, owner_user_id, owner_dept_id, materialized)) = resolve_visible_space(database, actor, &space_id).await? else {
        return Err(FileError::NotFound);
    };
    let target_id = if materialized {
        resolved_id
    } else {
        ensure_space(database, &owner_user_id, owner_dept_id.as_deref()).await?
    };
    query("UPDATE file_space SET quota_override_bytes=$2,updated_at=$3 WHERE space_id=$1")
        .bind(target_id.as_str())
        .bind(
            command
                .quota_bytes
                .map(|value| i64::try_from(value).map_err(|_| FileError::InvalidInput(keys::QUOTA_TOO_LARGE)))
                .transpose()?,
        )
        .bind(OffsetDateTime::now_utc())
        .execute(database.pool())
        .await
        .map_err(storage_error)?;
    list_spaces(
        database,
        actor,
        crate::application::FileSpaceQuery {
            cursor: None,
            owner_user_id: Some(owner_user_id),
            ..crate::application::FileSpaceQuery::default()
        },
        kernel::pagination::CursorPageRequest { limit: 1, cursor: None },
        default_quota,
    )
    .await?
    .items
    .into_iter()
    .find(|item| item.id == target_id.as_str())
    .ok_or(FileError::NotFound)
}

pub(super) async fn ensure_target_space(database: &Database, actor: &FileAccessScope, space_id: &SpaceId) -> FileResult<SpaceId> {
    let Some((resolved_id, owner_user_id, owner_dept_id, materialized)) = resolve_visible_space(database, actor, space_id).await? else {
        return Err(FileError::NotFound);
    };
    if materialized {
        return Ok(resolved_id);
    }
    ensure_space(database, &owner_user_id, owner_dept_id.as_deref()).await
}

pub(super) async fn ensure_visible_space(database: &Database, actor: &FileAccessScope, space_id: &SpaceId) -> FileResult<()> {
    let mut query = QueryBuilder::<Postgres>::new("SELECT s.owner_user_id,s.owner_dept_id FROM file_space s WHERE s.space_id=");
    query.push_bind(space_id.as_str().to_owned()).push(" AND");
    scope_query(&mut query, actor, "s");
    query
        .build()
        .fetch_optional(database.pool())
        .await
        .map_err(storage_error)?
        .map(|_| ())
        .ok_or(FileError::NotFound)
}

async fn ensure_parent_tx(transaction: &mut sqlx::Transaction<'_, Postgres>, space_id: &str, parent_id: DirectoryId, moving_id: FileId) -> FileResult<()> {
    if parent_id == DirectoryId::ROOT {
        return Ok(());
    }
    let row = query_as::<_, (String,)>("WITH RECURSIVE descendants AS (SELECT entry_id FROM file_entry WHERE entry_id=$1 UNION ALL SELECT e.entry_id FROM file_entry e JOIN descendants d ON e.parent_id=d.entry_id) SELECT entry_id FROM file_entry WHERE entry_id=$2 AND space_id=$3 AND kind='folder' AND status='active' AND entry_id NOT IN (SELECT entry_id FROM descendants) FOR UPDATE")
        .bind(moving_id.to_string()).bind(parent_id.to_string()).bind(space_id).fetch_optional(&mut **transaction).await.map_err(storage_error)?;
    row.map(|_| ()).ok_or(FileError::InvalidInput(keys::PARENT_FOLDER_INVALID))
}

fn parent_value(parent_id: DirectoryId) -> Option<String> {
    (parent_id != DirectoryId::ROOT).then(|| parent_id.to_string())
}

async fn replace_tags(transaction: &mut sqlx::Transaction<'_, Postgres>, space_id: &str, entry_id: FileId, tags: Vec<TagName>, actor: &str) -> FileResult<()> {
    query("DELETE FROM file_entry_tag WHERE entry_id=$1")
        .bind(entry_id.to_string())
        .execute(&mut **transaction)
        .await
        .map_err(storage_error)?;
    for tag in tags {
        let trimmed = tag.as_str();
        let normalized = tag.normalized();
        let tag_id = query_as::<_, (String,)>("INSERT INTO file_tag(tag_id,space_id,name,normalized_name,created_by,created_at) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(space_id,normalized_name) DO UPDATE SET name=file_tag.name RETURNING tag_id")
            .bind(FileId::new().to_string()).bind(space_id).bind(trimmed).bind(&normalized).bind(actor).bind(OffsetDateTime::now_utc()).fetch_one(&mut **transaction).await.map_err(storage_error)?.0;
        query("INSERT INTO file_entry_tag(entry_id,tag_id,created_at) VALUES($1,$2,$3) ON CONFLICT DO NOTHING")
            .bind(entry_id.to_string())
            .bind(tag_id)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut **transaction)
            .await
            .map_err(storage_error)?;
    }
    Ok(())
}

fn map_write(result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>) -> FileResult<()> {
    result.map(|_| ()).map_err(|error| {
        if error.to_string().contains("idx_file_entry_active_sibling_name") {
            FileError::NameConflict
        } else {
            storage_error(error)
        }
    })
}
