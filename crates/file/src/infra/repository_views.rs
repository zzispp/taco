use std::collections::HashMap;

use sqlx::query_as;
use storage::Database;

use crate::application::{FileEntryView, FileProperties, FileSpaceView, supports_inline_preview};
use crate::domain::ByteSize;
use crate::{FileError, FileResult};

use crate::infra::repository_support::{EntryRecord, SpaceRecord, format_time, storage_error};

pub(super) async fn build_views(database: &Database, records: Vec<EntryRecord>) -> FileResult<Vec<FileEntryView>> {
    let mut tags = entry_tags(database, &records).await?;
    let mut views = Vec::with_capacity(records.len());
    for record in records {
        let entry_tags = tags.remove(&record.entry_id).unwrap_or_default();
        views.push(entry_view(record, entry_tags)?);
    }
    Ok(views)
}

pub(in crate::infra) fn space_view(record: SpaceRecord, default_quota: ByteSize) -> FileSpaceView {
    FileSpaceView {
        id: record.space_id,
        owner_user_id: record.owner_user_id,
        owner_name: record.owner_name,
        department_name: record.department_name,
        status: record.status,
        logical_asset_size: record.active_bytes.max(0) as u64,
        managed_physical_usage: record.physical_bytes.max(0) as u64,
        reserved_bytes: record.reserved_bytes.max(0) as u64,
        quota_bytes: record.quota_override_bytes.unwrap_or(default_quota.bytes() as i64).max(0) as u64,
        updated_at: format_time(record.updated_at),
    }
}

async fn entry_tags(database: &Database, records: &[EntryRecord]) -> FileResult<HashMap<String, Vec<String>>> {
    if records.is_empty() {
        return Ok(HashMap::new());
    }
    let entry_ids = records.iter().map(|record| record.entry_id.clone()).collect::<Vec<_>>();
    query_as::<_, (String, String)>(
        "SELECT et.entry_id,t.name FROM file_entry_tag et JOIN file_tag t ON t.tag_id=et.tag_id WHERE et.entry_id=ANY($1) ORDER BY et.entry_id,t.normalized_name",
    )
        .bind(entry_ids)
        .fetch_all(database.pool())
        .await
        .map_err(storage_error)
        .map(group_tags)
}

fn group_tags(rows: Vec<(String, String)>) -> HashMap<String, Vec<String>> {
    let mut tags = HashMap::new();
    for (entry_id, tag) in rows {
        tags.entry(entry_id).or_insert_with(Vec::new).push(tag);
    }
    tags
}

fn entry_view(record: EntryRecord, tags: Vec<String>) -> FileResult<FileEntryView> {
    let size_bytes = u64::try_from(record.size_bytes).map_err(|_| FileError::Infrastructure("negative file size in database".into()))?;
    let is_file = record.kind == "file";
    let preview = is_file && supports_inline_preview(record.content_type.as_deref());
    Ok(FileEntryView {
        id: record.entry_id,
        space_id: record.space_id,
        owner_user_id: record.owner_user_id,
        owner_name: Some(record.owner_name),
        parent_id: record.parent_id,
        name: record.name.clone(),
        entry_type: record.kind,
        size_bytes,
        mime_type: record.content_type.clone(),
        object_url: None,
        thumbnail_url: None,
        created_at: format_time(record.created_at),
        updated_at: format_time(record.updated_at),
        trashed_at: record.trashed_at.map(format_time),
        tags,
        properties: FileProperties {
            checksum_sha256: record.sha256,
            extension: record.name.rsplit('.').next().filter(|extension| *extension != record.name).map(str::to_owned),
            mime_type: record.content_type,
            created_by: Some(record.created_by),
            provider_key: record.provider_key,
        },
        preview_supported: preview,
        download_only: is_file && !preview,
    })
}

#[cfg(test)]
mod tests {
    use super::group_tags;

    #[test]
    fn grouped_tags_keep_the_database_sort_order_per_entry() {
        let tags = group_tags(vec![
            ("entry-a".into(), "alpha".into()),
            ("entry-a".into(), "zulu".into()),
            ("entry-b".into(), "beta".into()),
        ]);

        assert_eq!(tags.get("entry-a"), Some(&vec!["alpha".into(), "zulu".into()]));
        assert_eq!(tags.get("entry-b"), Some(&vec!["beta".into()]));
    }
}
