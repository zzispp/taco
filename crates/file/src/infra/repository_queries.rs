use kernel::pagination::{CursorPage, CursorPageRequest};
use sqlx::{Postgres, QueryBuilder, query_as};
use storage::Database;

use crate::application::{ExistingObject, FileAccessScope, FileEntryView, FileListQuery, FileOverviewView, FilePage, FileReadRequest, TypeDistributionView};
use crate::domain::{ByteSize, ContentDigest, FileId, ProviderKey, SpaceId, StoredObjectId};
use crate::error::keys;
use crate::{FileError, FileResult};

use super::repository_support::{
    ContentRecord, EntryRecord, EntrySortSpec, ObjectRecord, PHYSICAL_USAGE_SQL, SpaceRecord, decode_cursor, encode_cursor, entry_columns, entry_query,
    normalize_list_filter, page_fingerprint, scope_query, storage_error,
};

#[path = "repository_views.rs"]
mod views;
use views::build_views;

#[path = "repository_space_queries.rs"]
mod spaces;
pub(super) use spaces::{ensure_visible_space, list_spaces, resolve_visible_space};

pub(super) async fn list_entries(database: &Database, actor: &FileAccessScope, filter: FileListQuery, page: CursorPageRequest) -> FileResult<FilePage> {
    page.validate().map_err(|_| FileError::InvalidInput(keys::CURSOR_LIMIT_INVALID))?;
    let filter = normalize_list_filter(filter);
    let sort = EntrySortSpec::from_filter(&filter)?;
    let fingerprint = page_fingerprint(actor, &filter);
    let cursor = decode_cursor(filter.cursor.as_deref(), &fingerprint, &page)?;
    let mut query = QueryBuilder::<Postgres>::new("SELECT ");
    query.push(entry_columns());
    entry_query(&mut query, actor, &filter)?;
    sort.push_cursor_bound(&mut query, cursor.as_ref())?;
    let limit = page.limit.checked_add(1).ok_or(FileError::InvalidInput(keys::CURSOR_LIMIT_TOO_LARGE))?;
    sort.push_order(&mut query);
    query
        .push(" LIMIT ")
        .push_bind(i64::try_from(limit).map_err(|_| FileError::InvalidInput(keys::CURSOR_LIMIT_TOO_LARGE))?);
    let records = query.build_query_as::<EntryRecord>().fetch_all(database.pool()).await.map_err(storage_error)?;
    let has_next = records.len() > page.limit as usize;
    let records = records.into_iter().take(page.limit as usize).collect::<Vec<_>>();
    let next = records
        .last()
        .filter(|_| has_next)
        .map(|record| encode_cursor(sort.cursor_value(record), &record.entry_id, &fingerprint, &page));
    let items = build_views(database, records).await?;
    Ok(CursorPage::new(items, next, None))
}

pub(super) async fn find_entry(database: &Database, actor: &FileAccessScope, id: FileId) -> FileResult<Option<FileEntryView>> {
    let filter = FileListQuery {
        trashed: None,
        ..FileListQuery::default()
    };
    let mut query = QueryBuilder::<Postgres>::new("SELECT ");
    query.push(entry_columns());
    entry_query(&mut query, actor, &filter)?;
    query
        .push(" AND e.entry_id=")
        .push_bind(id.to_string())
        .push(" AND e.status IN ('active','trashed')");
    let record = query
        .build_query_as::<EntryRecord>()
        .fetch_optional(database.pool())
        .await
        .map_err(storage_error)?;
    Ok(build_views(database, record.into_iter().collect()).await?.into_iter().next())
}

pub(super) async fn overview(
    database: &Database,
    actor: &FileAccessScope,
    requested_space: Option<SpaceId>,
    default_quota: ByteSize,
) -> FileResult<FileOverviewView> {
    let space_id = resolve_overview_space(database, actor, requested_space).await?;
    let space = overview_space_record(database, &space_id).await?;
    let (active, trashed, reserved, quota) = overview_usage(space.as_ref(), default_quota);
    let object_bytes = space.as_ref().map_or(0, |record| record.physical_bytes.max(0) as u64);
    let temporary = temporary_upload_bytes(database, &space_id).await?;
    let managed_physical = object_bytes.checked_add(temporary).ok_or(FileError::SizeMismatch)?;
    let logical_references = active.checked_add(trashed).ok_or(FileError::SizeMismatch)?;
    let distribution = type_distribution(database, &space_id).await?;
    let recent_files = recent_entries(database, actor, &space_id, false).await?;
    let recent_folders = recent_entries(database, actor, &space_id, true).await?;
    Ok(FileOverviewView {
        space_id: space_id.to_string(),
        logical_asset_size: active,
        managed_physical_usage: managed_physical,
        recycle_bin_size: trashed,
        temporary_upload_size: temporary,
        deduplication_savings: logical_references.saturating_sub(object_bytes),
        quota_bytes: quota,
        quota_reserved_bytes: reserved,
        type_distribution: distribution,
        recent_entries: recent_files,
        recent_folders,
    })
}

async fn resolve_overview_space(database: &Database, actor: &FileAccessScope, requested: Option<SpaceId>) -> FileResult<SpaceId> {
    let Some(space_id) = requested else {
        return SpaceId::new(actor.user_id.clone());
    };
    ensure_visible_space(database, actor, &space_id).await?;
    Ok(space_id)
}

async fn overview_space_record(database: &Database, space_id: &SpaceId) -> FileResult<Option<SpaceRecord>> {
    let mut query = QueryBuilder::<Postgres>::new(
        "SELECT s.space_id,s.owner_user_id,u.nick_name AS owner_name,d.dept_name AS department_name,s.status,s.active_bytes,s.trashed_bytes,s.reserved_bytes,s.quota_override_bytes,",
    );
    query.push(PHYSICAL_USAGE_SQL).push(
        " AS physical_bytes,s.updated_at FROM file_space s JOIN sys_user u ON u.user_id=s.owner_user_id LEFT JOIN sys_dept d ON d.dept_id=s.owner_dept_id WHERE s.space_id=",
    );
    query.push_bind(space_id.as_str().to_owned());
    query
        .build_query_as::<SpaceRecord>()
        .fetch_optional(database.pool())
        .await
        .map_err(storage_error)
}

fn overview_usage(space: Option<&SpaceRecord>, default_quota: ByteSize) -> (u64, u64, u64, u64) {
    space.map_or((0, 0, 0, default_quota.bytes()), |record| {
        (
            record.active_bytes.max(0) as u64,
            record.trashed_bytes.max(0) as u64,
            record.reserved_bytes.max(0) as u64,
            record.quota_override_bytes.unwrap_or(default_quota.bytes() as i64).max(0) as u64,
        )
    })
}

async fn temporary_upload_bytes(database: &Database, space_id: &SpaceId) -> FileResult<u64> {
    query_as::<_, (i64,)>("SELECT COALESCE(SUM(p.size_bytes),0)::BIGINT FROM file_upload_part p JOIN file_upload_session us ON us.session_id=p.session_id WHERE us.space_id=$1 AND us.state IN ('open','completing') AND p.state='completed'")
        .bind(space_id.as_str())
        .fetch_one(database.pool())
        .await
        .map(|row| row.0.max(0) as u64)
        .map_err(storage_error)
}

async fn type_distribution(database: &Database, space_id: &SpaceId) -> FileResult<Vec<TypeDistributionView>> {
    query_as::<_, (String, i64, i64)>(
        "SELECT CASE WHEN e.kind='folder' THEN 'folder' WHEN split_part(LOWER(COALESCE(o.content_type,'')),'/',1) IN ('image','video','audio','text','application') THEN split_part(LOWER(o.content_type),'/',1) ELSE 'other' END AS entry_type,COALESCE(SUM(COALESCE(o.size_bytes,0)),0)::BIGINT,COUNT(*) FROM file_entry e LEFT JOIN file_object o ON o.object_id=e.object_id WHERE e.space_id=$1 AND e.status='active' GROUP BY 1 ORDER BY 1",
    )
    .bind(space_id.as_str())
    .fetch_all(database.pool())
    .await
    .map_err(storage_error)
    .map(|rows| {
        rows.into_iter()
            .map(|(entry_type, bytes, count)| TypeDistributionView {
                entry_type,
                bytes: bytes.max(0) as u64,
                count: count.max(0) as u64,
            })
            .collect()
    })
}

pub(super) async fn read_content(
    database: &Database,
    actor: &FileAccessScope,
    request: FileReadRequest,
) -> FileResult<Option<(FileEntryView, ProviderKey, crate::application::ObjectKey)>> {
    let filter = FileListQuery {
        trashed: Some(false),
        ..FileListQuery::default()
    };
    let mut query = QueryBuilder::<Postgres>::new("SELECT ");
    query.push(entry_columns()).push(",o.object_key");
    entry_query(&mut query, actor, &filter)?;
    query.push(" AND e.entry_id=").push_bind(request.id.to_string()).push(" AND e.status='active'");
    let row = query
        .build_query_as::<ContentRecord>()
        .fetch_optional(database.pool())
        .await
        .map_err(storage_error)?;
    let Some(row) = row else { return Ok(None) };
    let record = row.record;
    let object_key = row.object_key;
    if record.kind != "file" {
        return Err(FileError::InvalidInput(keys::FOLDER_DOWNLOAD_FORBIDDEN));
    }
    let entry = build_views(database, vec![record]).await?.into_iter().next().ok_or(FileError::NotFound)?;
    let provider = ProviderKey::new(entry.properties.provider_key.clone().ok_or(FileError::NotFound)?)?;
    Ok(Some((entry, provider, crate::application::ObjectKey::new(object_key)?)))
}

pub(super) async fn find_reusable_object(
    database: &Database,
    actor: &FileAccessScope,
    _space_id: SpaceId,
    digest: ContentDigest,
    size: ByteSize,
) -> FileResult<Option<ExistingObject>> {
    let mut query = QueryBuilder::<Postgres>::new(
        "SELECT o.object_id,o.provider_key,o.object_key,o.size_bytes,o.sha256 FROM file_object o JOIN file_entry e ON e.object_id=o.object_id JOIN file_space s ON s.space_id=e.space_id WHERE e.status='active' AND o.status='active' AND o.sha256=",
    );
    query
        .push_bind(digest.to_hex())
        .push(" AND o.size_bytes=")
        .push_bind(i64::try_from(size.bytes()).map_err(|_| FileError::SizeMismatch)?)
        .push(" AND (");
    scope_query(&mut query, actor, "s");
    query.push(") ORDER BY o.created_at ASC LIMIT 1");
    let row = query
        .build_query_as::<ObjectRecord>()
        .fetch_optional(database.pool())
        .await
        .map_err(storage_error)?;
    let Some(row) = row else { return Ok(None) };
    let ObjectRecord {
        object_id,
        provider_key,
        object_key,
        size_bytes,
        sha256,
    } = row;
    Ok(Some(ExistingObject {
        object_id: StoredObjectId::parse(&object_id)?,
        provider_key: ProviderKey::new(provider_key)?,
        object_key,
        size: ByteSize::from_bytes(size_bytes.max(0) as u64),
        digest: ContentDigest::from_hex(&sha256)?,
    }))
}

async fn recent_entries(database: &Database, actor: &FileAccessScope, space_id: &SpaceId, folders: bool) -> FileResult<Vec<FileEntryView>> {
    let filter = FileListQuery {
        space_id: Some(space_id.clone()),
        parent_id: None,
        trashed: Some(false),
        ..FileListQuery::default()
    };
    let mut query = QueryBuilder::<Postgres>::new("SELECT ");
    query.push(entry_columns());
    entry_query(&mut query, actor, &filter)?;
    query.push(" AND e.kind=").push_bind(if folders { "folder" } else { "file" });
    query.push(" ORDER BY e.updated_at DESC,e.entry_id DESC LIMIT 10");
    let rows = query.build_query_as::<EntryRecord>().fetch_all(database.pool()).await.map_err(storage_error)?;
    build_views(database, rows).await
}

#[cfg(test)]
mod tests {
    use crate::infra::repository_support::PHYSICAL_USAGE_SQL;

    #[test]
    fn physical_usage_deduplicates_references_by_object_identity() {
        assert!(PHYSICAL_USAGE_SQL.contains("SELECT DISTINCT o.object_id,o.size_bytes"));
        assert!(!PHYSICAL_USAGE_SQL.contains("SUM(DISTINCT o.size_bytes)"));
    }
}
