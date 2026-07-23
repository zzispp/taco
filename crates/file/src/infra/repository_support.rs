use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use kernel::pagination::CursorPageRequest;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::{Postgres, QueryBuilder};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::application::{FileAccessScope, FileListQuery, FileScopeMode, FileSpaceQuery, StoredObject};
use crate::error::keys;
use crate::{FileError, FileResult};

#[path = "repository_cursor.rs"]
mod cursor;
pub(in crate::infra) use cursor::{EntrySortSpec, SpaceSortSpec};

pub(super) const PHYSICAL_USAGE_SQL: &str = "COALESCE((SELECT SUM(objects.size_bytes) FROM (SELECT DISTINCT o.object_id,o.size_bytes FROM file_object o JOIN file_entry pe ON pe.object_id=o.object_id WHERE pe.space_id=s.space_id) objects),0)::BIGINT";

pub(super) const VIRTUAL_SPACE_CTE: &str = "WITH visible_spaces AS (SELECT COALESCE(fs.space_id,u.user_id) AS space_id,u.user_id AS owner_user_id,u.nick_name AS owner_name,CASE WHEN u.del_flag='2' THEN fs.owner_dept_id ELSE u.dept_id END AS owner_dept_id,CASE WHEN u.del_flag='2' THEN 'archived' ELSE COALESCE(fs.status,'active') END AS status,COALESCE(fs.active_bytes,0) AS active_bytes,COALESCE(fs.trashed_bytes,0) AS trashed_bytes,COALESCE(fs.reserved_bytes,0) AS reserved_bytes,fs.quota_override_bytes,COALESCE(fs.updated_at,u.update_time,u.create_time) AS updated_at,fs.space_id IS NOT NULL AS materialized FROM sys_user u LEFT JOIN file_space fs ON fs.owner_user_id=u.user_id)";

#[derive(Clone, Debug, Eq, PartialEq, sqlx::FromRow)]
pub(super) struct EntryRecord {
    pub entry_id: String,
    pub space_id: String,
    pub owner_user_id: String,
    pub owner_name: String,
    pub parent_id: Option<String>,
    pub kind: String,
    pub name: String,
    pub normalized_name: String,
    pub status: String,
    pub trashed_at: Option<OffsetDateTime>,
    pub created_by: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
    pub size_bytes: i64,
    pub content_type: Option<String>,
    pub sha256: Option<String>,
    pub provider_key: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, sqlx::FromRow)]
pub(super) struct SpaceRecord {
    pub space_id: String,
    pub owner_user_id: String,
    pub owner_name: String,
    pub department_name: Option<String>,
    pub status: String,
    pub active_bytes: i64,
    pub trashed_bytes: i64,
    pub physical_bytes: i64,
    pub reserved_bytes: i64,
    pub quota_override_bytes: Option<i64>,
    pub updated_at: OffsetDateTime,
}

#[derive(Clone, Debug, Eq, PartialEq, sqlx::FromRow)]
pub(super) struct ContentRecord {
    #[sqlx(flatten)]
    pub record: EntryRecord,
    pub object_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq, sqlx::FromRow)]
pub(super) struct ObjectRecord {
    pub object_id: String,
    pub provider_key: String,
    pub object_key: String,
    pub size_bytes: i64,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(super) struct PageCursor {
    pub sort_value: String,
    pub id: String,
    pub fingerprint: String,
    pub limit: u64,
}

pub(super) fn storage_error(error: impl std::fmt::Display) -> FileError {
    FileError::Infrastructure(format!("file repository operation failed: {error}"))
}

pub(super) fn same_physical_object(provider_key: &str, object_key: &str, object: &StoredObject) -> bool {
    provider_key == object.provider_key.as_str() && object_key == object.key.as_str()
}

pub(super) fn format_time(value: OffsetDateTime) -> String {
    value.format(&Rfc3339).unwrap_or_else(|_| value.unix_timestamp().to_string())
}

pub(super) fn parse_time(value: &str) -> FileResult<OffsetDateTime> {
    OffsetDateTime::parse(value, &Rfc3339).map_err(|_| FileError::InvalidInput(keys::TIME_FILTER_INVALID))
}

pub(super) fn scope_query(query: &mut QueryBuilder<Postgres>, scope: &FileAccessScope, alias: &str) {
    match scope.mode {
        FileScopeMode::All => {
            query.push(" TRUE");
        }
        FileScopeMode::SelfOnly => push_owner_scope(query, scope, alias),
        FileScopeMode::Department => push_department_scope(query, scope, alias, false),
        FileScopeMode::DepartmentAndChildren => push_department_scope(query, scope, alias, true),
        FileScopeMode::Custom => push_custom_scope(query, scope, alias),
    };
}

fn push_owner_scope(query: &mut QueryBuilder<Postgres>, scope: &FileAccessScope, alias: &str) {
    query.push(" ").push(alias).push(".owner_user_id=").push_bind(scope.user_id.clone());
}

fn push_department_scope(query: &mut QueryBuilder<Postgres>, scope: &FileAccessScope, alias: &str, include_children: bool) {
    query.push(" (");
    push_owner_scope(query, scope, alias);
    query.push(" OR ");
    push_effective_department(query, alias);
    query.push("=").push_bind(scope.department_id.clone());
    if include_children {
        query.push(" OR EXISTS (SELECT 1 FROM sys_dept child WHERE child.dept_id=");
        push_effective_department(query, alias);
        query
            .push(" AND child.del_flag='0' AND (',' || child.ancestors || ',') LIKE '%,' || ")
            .push_bind(scope.department_id.clone())
            .push(" || ',%')");
    }
    query.push(")");
}

fn push_custom_scope(query: &mut QueryBuilder<Postgres>, scope: &FileAccessScope, alias: &str) {
    query.push(" (");
    push_owner_scope(query, scope, alias);
    query.push(" OR ");
    if scope.department_ids.is_empty() {
        query.push("FALSE");
    } else {
        push_effective_department(query, alias);
        query.push(" = ANY(").push_bind(scope.department_ids.clone()).push(")");
    }
    query.push(")");
}

fn push_effective_department(query: &mut QueryBuilder<Postgres>, alias: &str) {
    query
        .push("(SELECT CASE WHEN scoped_owner.del_flag='2' THEN ")
        .push(alias)
        .push(".owner_dept_id ELSE scoped_owner.dept_id END FROM sys_user scoped_owner WHERE scoped_owner.user_id=")
        .push(alias)
        .push(".owner_user_id)");
}

pub(super) async fn ensure_active_parent_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    space_id: &str,
    parent_id: crate::domain::DirectoryId,
) -> FileResult<()> {
    if parent_id == crate::domain::DirectoryId::ROOT {
        return Ok(());
    }
    let parent: Option<(String,)> =
        sqlx::query_as("SELECT entry_id FROM file_entry WHERE entry_id=$1 AND space_id=$2 AND kind='folder' AND status='active' FOR SHARE")
            .bind(parent_id.to_string())
            .bind(space_id)
            .fetch_optional(&mut **transaction)
            .await
            .map_err(storage_error)?;
    parent.map(|_| ()).ok_or(FileError::NotFound)
}

pub(super) fn normalize_list_filter(filter: FileListQuery) -> FileListQuery {
    FileListQuery {
        trashed: Some(filter.trashed.unwrap_or(false)),
        ..filter
    }
}

pub(super) fn entry_query(query: &mut QueryBuilder<Postgres>, scope: &FileAccessScope, filter: &FileListQuery) -> FileResult<()> {
    query.push(
        " FROM file_entry e JOIN file_space s ON s.space_id=e.space_id JOIN sys_user owner ON owner.user_id=s.owner_user_id LEFT JOIN file_object o ON o.object_id=e.object_id WHERE",
    );
    scope_query(query, scope, "s");
    if let Some(trashed) = filter.trashed {
        query.push(" AND e.status=");
        query.push_bind(if trashed { "trashed" } else { "active" });
    }
    if let Some(space_id) = &filter.space_id {
        query.push(" AND e.space_id=").push_bind(space_id.as_str().to_owned());
    }
    if let Some(parent_id) = filter.parent_id {
        if parent_id == crate::domain::DirectoryId::ROOT {
            query.push(" AND e.parent_id IS NULL");
        } else {
            query.push(" AND e.parent_id=").push_bind(parent_id.to_string());
        }
    }
    if let Some(kind) = &filter.kind {
        let kind = match kind.as_str() {
            "file" | "folder" => kind,
            _ => return Err(FileError::InvalidInput(keys::ENTRY_TYPE_INVALID)),
        };
        query.push(" AND e.kind=").push_bind(kind.to_owned());
    }
    if let Some(search) = &filter.search {
        query.push(" AND e.name ILIKE '%' || ").push_bind(search.clone()).push(" || '%'");
    }
    if let Some(mime_type) = &filter.mime_type {
        query.push(" AND o.content_type=").push_bind(mime_type.clone());
    }
    if let Some(extension) = &filter.extension {
        query.push(" AND e.name ILIKE '%.' || ").push_bind(extension.clone()).push(" ");
    }
    if let Some(tag) = &filter.tag {
        query
            .push(" AND EXISTS (SELECT 1 FROM file_entry_tag et JOIN file_tag t ON t.tag_id=et.tag_id WHERE et.entry_id=e.entry_id AND t.normalized_name=")
            .push_bind(tag.to_lowercase())
            .push(")");
    }
    if let Some(start) = &filter.start_time {
        query.push(" AND e.created_at>=").push_bind(parse_time(start)?);
    }
    if let Some(end) = &filter.end_time {
        query.push(" AND e.created_at<=").push_bind(parse_time(end)?);
    }
    Ok(())
}

pub(super) fn entry_columns() -> &'static str {
    "e.entry_id,e.space_id,s.owner_user_id,owner.nick_name AS owner_name,e.parent_id,e.kind,e.name,e.normalized_name,e.status,e.trashed_at,e.created_by,e.created_at,e.updated_at,COALESCE(o.size_bytes,0) AS size_bytes,o.content_type,o.sha256,o.provider_key"
}

pub(super) fn page_fingerprint(scope: &FileAccessScope, filter: &FileListQuery) -> String {
    let mut department_ids = scope.department_ids.clone();
    department_ids.sort();
    let filter = FileListQuery {
        cursor: None,
        ..filter.clone()
    };
    let value = serde_json::json!({
        "scope": {"mode": format!("{:?}", scope.mode), "user": scope.user_id, "dept": scope.department_id, "depts": department_ids},
        "filter": filter,
    });
    let mut digest = Sha256::new();
    digest.update(serde_json::to_vec(&value).unwrap_or_default());
    format!("{:x}", digest.finalize())
}

pub(super) fn space_page_fingerprint(scope: &FileAccessScope, filter: &FileSpaceQuery) -> String {
    let mut department_ids = scope.department_ids.clone();
    department_ids.sort();
    let filter = FileSpaceQuery {
        cursor: None,
        ..filter.clone()
    };
    let value = serde_json::json!({
        "scope": {"mode": format!("{:?}", scope.mode), "user": scope.user_id, "dept": scope.department_id, "depts": department_ids},
        "filter": filter,
    });
    let mut digest = Sha256::new();
    digest.update(serde_json::to_vec(&value).unwrap_or_default());
    format!("{:x}", digest.finalize())
}

pub(super) fn encode_cursor(sort_value: impl Into<String>, id: &str, fingerprint: &str, page: &CursorPageRequest) -> String {
    let cursor = PageCursor {
        sort_value: sort_value.into(),
        id: id.to_owned(),
        fingerprint: fingerprint.to_owned(),
        limit: page.limit,
    };
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(&cursor).expect("file cursor serialization is infallible"))
}

pub(super) fn decode_cursor(cursor: Option<&str>, fingerprint: &str, page: &CursorPageRequest) -> FileResult<Option<PageCursor>> {
    let Some(cursor) = cursor else { return Ok(None) };
    let bytes = URL_SAFE_NO_PAD.decode(cursor).map_err(|_| FileError::InvalidInput(keys::CURSOR_MALFORMED))?;
    let value: PageCursor = serde_json::from_slice(&bytes).map_err(|_| FileError::InvalidInput(keys::CURSOR_MALFORMED))?;
    if value.fingerprint != fingerprint || value.limit != page.limit {
        return Err(FileError::InvalidInput(keys::CURSOR_QUERY_MISMATCH));
    }
    Ok(Some(value))
}

#[cfg(test)]
#[path = "repository_support_tests.rs"]
mod tests;
