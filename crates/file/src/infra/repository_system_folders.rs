use sqlx::{query, query_as};
use storage::Database;
use time::OffsetDateTime;

use crate::domain::{DirectoryId, SpaceId};
use crate::{FileError, FileResult};

use super::repository_support::storage_error;

const AVATAR_FOLDER_NAME: &str = "Avatars";
const AVATAR_FOLDER_NORMALIZED_NAME: &str = "avatars";
const AVATAR_SYSTEM_KIND: &str = "avatar";

pub(super) async fn ensure_space(database: &Database, owner_user_id: &str, owner_dept_id: Option<&str>) -> FileResult<SpaceId> {
    let now = OffsetDateTime::now_utc();
    query(
        "INSERT INTO file_space(space_id,owner_user_id,owner_dept_id,created_at,updated_at) VALUES($1,$2,$3,$4,$4) ON CONFLICT(owner_user_id) DO UPDATE SET owner_dept_id=EXCLUDED.owner_dept_id,updated_at=EXCLUDED.updated_at",
    )
    .bind(owner_user_id)
    .bind(owner_user_id)
    .bind(owner_dept_id)
    .bind(now)
    .execute(database.pool())
    .await
    .map_err(storage_error)?;
    SpaceId::new(owner_user_id.to_owned())
}

pub(super) async fn ensure_avatar_folder(database: &Database, owner_user_id: &str, owner_dept_id: Option<&str>) -> FileResult<DirectoryId> {
    let space_id = ensure_space(database, owner_user_id, owner_dept_id).await?;
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    lock_space(&mut transaction, &space_id).await?;
    if let Some(id) = find_avatar_folder(&mut transaction, &space_id).await? {
        transaction.commit().await.map_err(storage_error)?;
        return DirectoryId::parse(&id);
    }
    let id = DirectoryId::new();
    insert_avatar_folder(&mut transaction, &space_id, id, owner_user_id).await?;
    transaction.commit().await.map_err(storage_error)?;
    Ok(id)
}

async fn lock_space(transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>, space_id: &SpaceId) -> FileResult<()> {
    query_as::<_, (String,)>("SELECT space_id FROM file_space WHERE space_id=$1 FOR UPDATE")
        .bind(space_id.as_str())
        .fetch_one(&mut **transaction)
        .await
        .map_err(storage_error)?;
    Ok(())
}

async fn find_avatar_folder(transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>, space_id: &SpaceId) -> FileResult<Option<String>> {
    query_as::<_, (String,)>("SELECT entry_id FROM file_entry WHERE space_id=$1 AND kind='folder' AND system_kind=$2 AND status='active'")
        .bind(space_id.as_str())
        .bind(AVATAR_SYSTEM_KIND)
        .fetch_optional(&mut **transaction)
        .await
        .map(|row| row.map(|value| value.0))
        .map_err(storage_error)
}

async fn insert_avatar_folder(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    space_id: &SpaceId,
    id: DirectoryId,
    owner_user_id: &str,
) -> FileResult<()> {
    let now = OffsetDateTime::now_utc();
    query(
        "INSERT INTO file_entry(entry_id,space_id,parent_id,kind,name,normalized_name,object_id,status,system_kind,created_by,created_at,updated_by,updated_at) VALUES($1,$2,NULL,'folder',$3,$4,NULL,'active',$5,$6,$7,$6,$7)",
    )
    .bind(id.to_string())
    .bind(space_id.as_str())
    .bind(AVATAR_FOLDER_NAME)
    .bind(AVATAR_FOLDER_NORMALIZED_NAME)
    .bind(AVATAR_SYSTEM_KIND)
    .bind(owner_user_id)
    .bind(now)
    .execute(&mut **transaction)
    .await
    .map_err(map_insert_error)?;
    Ok(())
}

fn map_insert_error(error: sqlx::Error) -> FileError {
    if error.to_string().contains("idx_file_entry_active_sibling_name") {
        FileError::NameConflict
    } else {
        storage_error(error)
    }
}
