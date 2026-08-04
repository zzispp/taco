use std::collections::BTreeSet;

use sqlx::{Postgres, QueryBuilder, query, query_as};
use storage::Database;
use time::OffsetDateTime;

use crate::application::{FileAccessScope, normalize_batch_ids};
use crate::domain::FileId;
use crate::{FileError, FileResult};

use super::repository_purge::{ensure_tree_has_no_references, ensure_tree_mutable, selected_roots};
use super::repository_support::{scope_query, storage_error};

pub(super) struct FileStatusChangeRequest<'a> {
    pub(super) actor: &'a FileAccessScope,
    pub(super) ids: &'a [FileId],
    pub(super) now: OffsetDateTime,
}

impl<'a> FileStatusChangeRequest<'a> {
    pub(super) fn new(actor: &'a FileAccessScope, ids: &'a [FileId]) -> Self {
        Self {
            actor,
            ids,
            now: OffsetDateTime::now_utc(),
        }
    }
}

struct StatusMutation<'a> {
    actor: &'a FileAccessScope,
    ids: &'a [FileId],
    from: &'static str,
    to: &'static str,
    now: OffsetDateTime,
}

struct StatusMutationValidation<'a> {
    actor: &'a FileAccessScope,
    roots: &'a [FileId],
    from: &'static str,
    to: &'static str,
}

struct RootStatusMutation<'a> {
    roots: &'a [FileId],
    to: &'static str,
    now: OffsetDateTime,
}

struct EntryStatusMutation<'a> {
    actor: &'a FileAccessScope,
    id: FileId,
    from: &'static str,
    to: &'static str,
}

pub(in crate::infra) async fn trash(database: &Database, request: FileStatusChangeRequest<'_>) -> FileResult<()> {
    let FileStatusChangeRequest { actor, ids, now } = request;
    mutate_status(
        database,
        StatusMutation {
            actor,
            ids,
            from: "active",
            to: "trashed",
            now,
        },
    )
    .await
}

pub(in crate::infra) async fn restore(database: &Database, request: FileStatusChangeRequest<'_>) -> FileResult<()> {
    let FileStatusChangeRequest { actor, ids, now } = request;
    mutate_status(
        database,
        StatusMutation {
            actor,
            ids,
            from: "trashed",
            to: "active",
            now,
        },
    )
    .await
}

async fn mutate_status(database: &Database, request: StatusMutation<'_>) -> FileResult<()> {
    let StatusMutation { actor, ids, from, to, now } = request;
    let ids = normalize_batch_ids(ids.iter().copied())?;
    let mut transaction = database.pool().begin().await.map_err(storage_error)?;
    let roots = selected_roots(&mut transaction, &ids).await?;
    let spaces = validate_status_mutations(
        &mut transaction,
        StatusMutationValidation {
            actor,
            roots: &roots,
            from,
            to,
        },
    )
    .await?;
    mutate_roots(&mut transaction, RootStatusMutation { roots: &roots, to, now }).await?;
    recalc_space_usage(&mut transaction, &spaces, now).await?;
    transaction.commit().await.map_err(storage_error)
}

async fn validate_status_mutations(transaction: &mut sqlx::Transaction<'_, Postgres>, request: StatusMutationValidation<'_>) -> FileResult<Vec<String>> {
    let StatusMutationValidation { actor, roots, from, to } = request;
    let mut spaces = BTreeSet::new();
    for id in roots {
        spaces.insert(validate_status_mutation(transaction, EntryStatusMutation { actor, id: *id, from, to }).await?);
    }
    Ok(spaces.into_iter().collect())
}

async fn mutate_roots(transaction: &mut sqlx::Transaction<'_, Postgres>, request: RootStatusMutation<'_>) -> FileResult<()> {
    let RootStatusMutation { roots, to, now } = request;
    let sql = status_update_sql(to);
    for id in roots {
        query(sql)
            .bind(id.to_string())
            .bind(now)
            .execute(&mut **transaction)
            .await
            .map_err(map_sql_write)?;
    }
    Ok(())
}

fn status_update_sql(to: &str) -> &'static str {
    match to {
        "trashed" => {
            "WITH RECURSIVE tree AS (SELECT entry_id FROM file_entry WHERE entry_id=$1 UNION ALL SELECT child.entry_id FROM file_entry child JOIN tree ON child.parent_id=tree.entry_id WHERE child.status='active') UPDATE file_entry SET status='trashed',trashed_at=$2,updated_at=$2 WHERE entry_id IN (SELECT entry_id FROM tree)"
        }
        "active" => {
            "WITH RECURSIVE tree AS (SELECT entry_id FROM file_entry WHERE entry_id=$1 UNION ALL SELECT child.entry_id FROM file_entry child JOIN tree ON child.parent_id=tree.entry_id WHERE child.status='trashed') UPDATE file_entry SET status='active',trashed_at=NULL,updated_at=$2 WHERE entry_id IN (SELECT entry_id FROM tree)"
        }
        _ => unreachable!("file status transitions are fixed by the repository"),
    }
}

async fn validate_status_mutation(transaction: &mut sqlx::Transaction<'_, Postgres>, request: EntryStatusMutation<'_>) -> FileResult<String> {
    let EntryStatusMutation { actor, id, from, to } = request;
    let mut lookup =
        QueryBuilder::<Postgres>::new("SELECT e.space_id,e.parent_id FROM file_entry e JOIN file_space s ON s.space_id=e.space_id WHERE e.entry_id=");
    lookup.push_bind(id.to_string()).push(" AND e.status=").push_bind(from).push(" AND");
    scope_query(&mut lookup, actor, "s");
    lookup.push(" FOR UPDATE OF e");
    let row = lookup
        .build_query_as::<(String, Option<String>)>()
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?;
    let Some((space_id, parent_id)) = row else { return Err(FileError::NotFound) };
    if to == "trashed" {
        ensure_tree_mutable(transaction, id).await?;
        ensure_tree_has_no_references(transaction, id).await?;
    } else {
        ensure_restore_parent(transaction, parent_id).await?;
        ensure_restore_namespace(transaction, id).await?;
    }
    Ok(space_id)
}

async fn ensure_restore_parent(transaction: &mut sqlx::Transaction<'_, Postgres>, parent_id: Option<String>) -> FileResult<()> {
    let Some(parent_id) = parent_id else { return Ok(()) };
    let active = query_as::<_, (String,)>("SELECT entry_id FROM file_entry WHERE entry_id=$1 AND kind='folder' AND status='active'")
        .bind(parent_id)
        .fetch_optional(&mut **transaction)
        .await
        .map_err(storage_error)?;
    active.map(|_| ()).ok_or(FileError::NameConflict)
}

async fn ensure_restore_namespace(transaction: &mut sqlx::Transaction<'_, Postgres>, id: FileId) -> FileResult<()> {
    let conflicts: (i64,) = query_as(
        "WITH RECURSIVE tree AS (SELECT entry_id,space_id,parent_id,normalized_name FROM file_entry WHERE entry_id=$1 UNION ALL SELECT e.entry_id,e.space_id,e.parent_id,e.normalized_name FROM file_entry e JOIN tree t ON e.parent_id=t.entry_id) SELECT COUNT(*) FROM tree t JOIN file_entry active ON active.space_id=t.space_id AND COALESCE(active.parent_id,'')=COALESCE(t.parent_id,'') AND active.normalized_name=t.normalized_name AND active.status='active' WHERE active.entry_id NOT IN (SELECT entry_id FROM tree)",
    )
    .bind(id.to_string())
    .fetch_one(&mut **transaction)
    .await
    .map_err(storage_error)?;
    if conflicts.0 > 0 {
        return Err(FileError::NameConflict);
    }
    Ok(())
}

async fn recalc_space_usage(transaction: &mut sqlx::Transaction<'_, Postgres>, spaces: &[String], now: OffsetDateTime) -> FileResult<()> {
    for space in spaces {
        query("UPDATE file_space SET active_bytes=COALESCE((SELECT SUM(o.size_bytes) FROM file_entry e JOIN file_object o ON o.object_id=e.object_id WHERE e.space_id=$1 AND e.status='active'),0),trashed_bytes=COALESCE((SELECT SUM(o.size_bytes) FROM file_entry e JOIN file_object o ON o.object_id=e.object_id WHERE e.space_id=$1 AND e.status='trashed'),0),updated_at=$2 WHERE space_id=$1")
            .bind(space)
            .bind(now)
            .execute(&mut **transaction)
            .await
            .map_err(storage_error)?;
    }
    Ok(())
}

fn map_sql_write(error: sqlx::Error) -> FileError {
    if error.to_string().contains("idx_file_entry_active_sibling_name") {
        FileError::NameConflict
    } else {
        storage_error(error)
    }
}
