use std::collections::{BTreeMap, BTreeSet};

use sqlx::{Postgres, QueryBuilder, query, query_as};
use storage::Database;
use time::OffsetDateTime;

use crate::application::{CleanupObject, FileAccessScope, PurgeBatch, normalize_batch_ids};
use crate::domain::{FileId, ProviderKey, StoredObjectId};
use crate::error::keys;
use crate::{FileError, FileResult};

use super::repository_support::{scope_query, storage_error};

pub(in crate::infra) async fn purge(database: &Database, actor: &FileAccessScope, ids: &[FileId]) -> FileResult<PurgeBatch> {
    let ids = normalize_batch_ids(ids.iter().copied())?;
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    let roots = selected_roots(&mut transaction, &ids).await?;
    let spaces = validate_purge_roots(&mut transaction, actor, &roots).await?;
    let mut object_refs = BTreeMap::<String, i64>::new();
    let purged_entries = purge_roots(&mut transaction, &roots, &mut object_refs).await?;
    mark_objects(&mut transaction, &object_refs).await?;
    recalc_spaces(&mut transaction, &spaces).await?;
    let objects = deleting_objects(&mut transaction, &object_refs).await?;
    transaction.commit().await.map_err(storage_error)?;
    Ok(PurgeBatch { purged_entries, objects })
}

async fn validate_purge_roots(transaction: &mut sqlx::Transaction<'_, Postgres>, actor: &FileAccessScope, roots: &[FileId]) -> FileResult<Vec<String>> {
    let mut spaces = BTreeSet::new();
    for id in roots {
        spaces.insert(validate_purge_root(transaction, actor, *id).await?);
    }
    Ok(spaces.into_iter().collect())
}

async fn validate_purge_root(transaction: &mut sqlx::Transaction<'_, Postgres>, actor: &FileAccessScope, id: FileId) -> FileResult<String> {
    let mut lookup = QueryBuilder::<Postgres>::new("SELECT e.space_id FROM file_entry e JOIN file_space s ON s.space_id=e.space_id WHERE e.entry_id=");
    lookup.push_bind(id.to_string()).push(" AND e.status='trashed' AND");
    scope_query(&mut lookup, actor, "s");
    lookup.push(" FOR UPDATE OF e");
    let space_id = lookup
        .build_query_as::<(String,)>()
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?
        .ok_or(FileError::NotFound)?
        .0;
    if !lock_trashed_tree(transaction, id).await? {
        return Err(FileError::InvalidInput(keys::PURGE_REQUIRES_TRASHED));
    }
    ensure_tree_mutable(transaction, id).await?;
    ensure_tree_has_no_references(transaction, id).await?;
    Ok(space_id)
}

async fn purge_roots(transaction: &mut sqlx::Transaction<'_, Postgres>, roots: &[FileId], object_refs: &mut BTreeMap<String, i64>) -> FileResult<u64> {
    let mut purged_entries = 0_u64;
    for id in roots {
        collect_object_refs(transaction, *id, object_refs).await?;
        purged_entries += delete_tree(transaction, *id).await?;
    }
    Ok(purged_entries)
}

async fn collect_object_refs(transaction: &mut sqlx::Transaction<'_, Postgres>, id: FileId, object_refs: &mut BTreeMap<String, i64>) -> FileResult<()> {
    let objects: Vec<(String, i64)> = query_as(
        "WITH RECURSIVE tree AS (SELECT entry_id,object_id FROM file_entry WHERE entry_id=$1 UNION ALL SELECT e.entry_id,e.object_id FROM file_entry e JOIN tree t ON e.parent_id=t.entry_id) SELECT object_id,COUNT(*) FROM tree WHERE object_id IS NOT NULL GROUP BY object_id",
    )
    .bind(id.to_string())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage_error)?;
    for (object_id, count) in objects {
        *object_refs.entry(object_id).or_default() += count;
    }
    Ok(())
}

async fn delete_tree(transaction: &mut sqlx::Transaction<'_, Postgres>, id: FileId) -> FileResult<u64> {
    query(
        "WITH RECURSIVE tree AS (SELECT entry_id FROM file_entry WHERE entry_id=$1 UNION ALL SELECT e.entry_id FROM file_entry e JOIN tree t ON e.parent_id=t.entry_id) DELETE FROM file_entry WHERE entry_id IN (SELECT entry_id FROM tree)",
    )
    .bind(id.to_string())
    .execute(&mut **transaction)
    .await
    .map_err(storage_error)
    .map(|result| result.rows_affected())
}

async fn mark_objects(transaction: &mut sqlx::Transaction<'_, Postgres>, object_refs: &BTreeMap<String, i64>) -> FileResult<()> {
    for (object_id, count) in object_refs {
        query("UPDATE file_object SET ref_count=GREATEST(ref_count-$2,0),status=CASE WHEN ref_count<=$2 THEN 'deleting' ELSE status END,updated_at=$3 WHERE object_id=$1")
            .bind(object_id)
            .bind(count)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut **transaction)
            .await
            .map_err(storage_error)?;
    }
    Ok(())
}

async fn deleting_objects(transaction: &mut sqlx::Transaction<'_, Postgres>, object_refs: &BTreeMap<String, i64>) -> FileResult<Vec<CleanupObject>> {
    let mut objects = Vec::new();
    for object_id in object_refs.keys() {
        if let Some((id, provider, key)) = query_as::<_, (String, String, String)>(
            "SELECT object_id,provider_key,object_key FROM file_object WHERE object_id=$1 AND ref_count=0 AND status='deleting'",
        )
        .bind(object_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?
        {
            objects.push(CleanupObject {
                object_id: StoredObjectId::parse(&id)?,
                provider_key: ProviderKey::new(provider)?,
                object_key: crate::application::ObjectKey::new(key)?,
            });
        }
    }
    Ok(objects)
}

pub(in crate::infra) async fn selected_roots(transaction: &mut sqlx::Transaction<'_, Postgres>, ids: &[FileId]) -> FileResult<Vec<FileId>> {
    let ids = ids.iter().map(ToString::to_string).collect::<Vec<_>>();
    query_as::<_, (String,)>(
        "WITH RECURSIVE selected(entry_id) AS (SELECT DISTINCT entry_id FROM unnest($1::TEXT[]) AS request(entry_id)), tree(root_id,entry_id) AS (SELECT entry_id,entry_id FROM selected UNION ALL SELECT tree.root_id,child.entry_id FROM tree JOIN file_entry child ON child.parent_id=tree.entry_id) SELECT selected.entry_id FROM selected WHERE NOT EXISTS (SELECT 1 FROM tree WHERE tree.entry_id=selected.entry_id AND tree.root_id<>selected.entry_id) ORDER BY selected.entry_id",
    )
    .bind(ids)
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage_error)?
    .into_iter()
    .map(|(id,)| FileId::parse(&id))
    .collect()
}

pub(in crate::infra) async fn ensure_tree_has_no_references(transaction: &mut sqlx::Transaction<'_, Postgres>, id: FileId) -> FileResult<()> {
    let refs: (i64,) = query_as(
        "WITH RECURSIVE tree AS (SELECT entry_id FROM file_entry WHERE entry_id=$1 UNION ALL SELECT e.entry_id FROM file_entry e JOIN tree t ON e.parent_id=t.entry_id) SELECT COUNT(*) FROM file_business_reference r WHERE r.entry_id IN (SELECT entry_id FROM tree)",
    )
    .bind(id.to_string())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if refs.0 > 0 {
        return Err(FileError::InvalidInput(keys::ACTIVE_BUSINESS_REFERENCES));
    }
    let avatars: (i64,) = query_as(
        "WITH RECURSIVE tree AS (SELECT entry_id FROM file_entry WHERE entry_id=$1 UNION ALL SELECT e.entry_id FROM file_entry e JOIN tree t ON e.parent_id=t.entry_id) SELECT COUNT(*) FROM sys_user u WHERE u.avatar_file_id IN (SELECT entry_id FROM tree)",
    )
    .bind(id.to_string())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if avatars.0 > 0 {
        return Err(FileError::InvalidInput(keys::AVATAR_REFERENCE));
    }
    let active_uploads: (i64,) = query_as(
        "WITH RECURSIVE tree AS (SELECT entry_id FROM file_entry WHERE entry_id=$1 UNION ALL SELECT e.entry_id FROM file_entry e JOIN tree t ON e.parent_id=t.entry_id) SELECT COUNT(*) FROM file_upload_session us WHERE us.state IN ('open','completing') AND us.parent_id IN (SELECT entry_id FROM tree)",
    )
    .bind(id.to_string())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if active_uploads.0 > 0 {
        return Err(FileError::InvalidInput(keys::ACTIVE_UPLOAD_TARGET));
    }
    Ok(())
}

pub(in crate::infra) async fn lock_trashed_tree(transaction: &mut sqlx::Transaction<'_, Postgres>, id: FileId) -> FileResult<bool> {
    let rows: Vec<(String, String)> = query_as(
        "WITH RECURSIVE tree AS (SELECT entry_id,status FROM file_entry WHERE entry_id=$1 UNION ALL SELECT e.entry_id,e.status FROM file_entry e JOIN tree t ON e.parent_id=t.entry_id) SELECT entry_id,status FROM tree FOR UPDATE",
    )
    .bind(id.to_string())
    .fetch_all(&mut **transaction)
    .await
    .map_err(storage_error)?;
    Ok(!rows.is_empty() && rows.iter().all(|(_, status)| status == "trashed"))
}

pub(in crate::infra) async fn ensure_tree_mutable(transaction: &mut sqlx::Transaction<'_, Postgres>, id: FileId) -> FileResult<()> {
    let system_entries: (i64,) = query_as(
        "WITH RECURSIVE tree AS (SELECT entry_id,system_kind FROM file_entry WHERE entry_id=$1 UNION ALL SELECT e.entry_id,e.system_kind FROM file_entry e JOIN tree t ON e.parent_id=t.entry_id) SELECT COUNT(*) FROM tree WHERE system_kind IS NOT NULL",
    )
    .bind(id.to_string())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if system_entries.0 > 0 {
        return Err(FileError::InvalidInput(keys::SYSTEM_FOLDER_IMMUTABLE));
    }
    Ok(())
}

pub(in crate::infra) async fn recalc_spaces(transaction: &mut sqlx::Transaction<'_, Postgres>, spaces: &[String]) -> FileResult<()> {
    for space in spaces {
        query("UPDATE file_space SET active_bytes=COALESCE((SELECT SUM(o.size_bytes) FROM file_entry e JOIN file_object o ON o.object_id=e.object_id WHERE e.space_id=$1 AND e.status='active'),0),trashed_bytes=COALESCE((SELECT SUM(o.size_bytes) FROM file_entry e JOIN file_object o ON o.object_id=e.object_id WHERE e.space_id=$1 AND e.status='trashed'),0),updated_at=$2 WHERE space_id=$1")
            .bind(space)
            .bind(OffsetDateTime::now_utc())
            .execute(&mut **transaction)
            .await
            .map_err(storage_error)?;
    }
    Ok(())
}
