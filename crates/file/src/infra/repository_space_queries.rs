use kernel::pagination::{CursorPage, CursorPageRequest};
use sqlx::{Postgres, QueryBuilder};
use storage::Database;

use crate::application::{FileAccessScope, FileSpaceQuery, FileSpaceView};
use crate::domain::SpaceId;
use crate::error::keys;
use crate::{FileError, FileResult};

use super::views::space_view;
use crate::infra::repository_support::{
    PHYSICAL_USAGE_SQL, SpaceRecord, SpaceSortSpec, VIRTUAL_SPACE_CTE, decode_cursor, encode_cursor, scope_query, space_page_fingerprint, storage_error,
};

pub(in crate::infra) async fn list_spaces(
    database: &Database,
    actor: &FileAccessScope,
    filter: FileSpaceQuery,
    page: CursorPageRequest,
    default_quota: crate::domain::ByteSize,
) -> FileResult<CursorPage<FileSpaceView>> {
    page.validate().map_err(|_| FileError::InvalidInput(keys::CURSOR_LIMIT_INVALID))?;
    let sort = SpaceSortSpec::from_filter(&filter)?;
    let fingerprint = space_page_fingerprint(actor, &filter);
    let cursor = decode_cursor(filter.cursor.as_deref(), &fingerprint, &page)?;
    let mut query = QueryBuilder::<Postgres>::new(VIRTUAL_SPACE_CTE);
    query.push(" SELECT s.space_id,s.owner_user_id,s.owner_name,d.dept_name AS department_name,s.status,s.active_bytes,s.trashed_bytes,s.reserved_bytes,s.quota_override_bytes,");
    query.push(PHYSICAL_USAGE_SQL);
    query.push(" AS physical_bytes,s.updated_at FROM visible_spaces s LEFT JOIN sys_dept d ON d.dept_id=s.owner_dept_id WHERE");
    scope_query(&mut query, actor, "s");
    append_filters(&mut query, &filter);
    sort.push_cursor_bound(&mut query, cursor.as_ref(), default_quota)?;
    let limit = page.limit.checked_add(1).ok_or(FileError::InvalidInput(keys::CURSOR_LIMIT_TOO_LARGE))?;
    sort.push_order(&mut query, default_quota)?;
    query.push(" LIMIT ");
    query.push_bind(i64::try_from(limit).map_err(|_| FileError::InvalidInput(keys::CURSOR_LIMIT_TOO_LARGE))?);
    let rows = query.build_query_as::<SpaceRecord>().fetch_all(database.pool()).await.map_err(storage_error)?;
    let has_next = rows.len() > page.limit as usize;
    let rows = rows.into_iter().take(page.limit as usize).collect::<Vec<_>>();
    let next = next_cursor(rows.last().filter(|_| has_next), sort, default_quota, &fingerprint, &page)?;
    Ok(CursorPage::new(
        rows.into_iter().map(|row| space_view(row, default_quota)).collect(),
        next,
        None,
    ))
}

pub(in crate::infra) async fn resolve_visible_space(
    database: &Database,
    actor: &FileAccessScope,
    requested: &SpaceId,
) -> FileResult<Option<(SpaceId, String, Option<String>, bool)>> {
    let mut query = QueryBuilder::<Postgres>::new(VIRTUAL_SPACE_CTE);
    query.push(" SELECT s.space_id,s.owner_user_id,s.owner_dept_id,s.materialized FROM visible_spaces s WHERE s.space_id=");
    query.push_bind(requested.as_str().to_owned()).push(" AND");
    scope_query(&mut query, actor, "s");
    let row = query
        .build_query_as::<(String, String, Option<String>, bool)>()
        .fetch_optional(database.pool())
        .await
        .map_err(storage_error)?;
    row.map(|(space_id, owner, department, materialized)| Ok((SpaceId::new(space_id)?, owner, department, materialized)))
        .transpose()
}

pub(in crate::infra) async fn ensure_visible_space(database: &Database, actor: &FileAccessScope, space_id: &SpaceId) -> FileResult<()> {
    resolve_visible_space(database, actor, space_id).await?.map(|_| ()).ok_or(FileError::NotFound)
}

fn append_filters(query: &mut QueryBuilder<Postgres>, filter: &FileSpaceQuery) {
    if let Some(owner) = &filter.owner_user_id {
        query.push(" AND s.owner_user_id=").push_bind(owner.clone());
    }
    if let Some(status) = &filter.status {
        query.push(" AND s.status=").push_bind(status.clone());
    }
    if let Some(search) = &filter.search {
        query
            .push(" AND (s.owner_name ILIKE '%' || ")
            .push_bind(search.clone())
            .push(" || '%' OR s.owner_user_id ILIKE '%' || ")
            .push_bind(search.clone())
            .push(" || '%' OR COALESCE(d.dept_name,'') ILIKE '%' || ")
            .push_bind(search.clone())
            .push(" || '%')");
    }
}

fn next_cursor(
    row: Option<&SpaceRecord>,
    sort: SpaceSortSpec,
    default_quota: crate::domain::ByteSize,
    fingerprint: &str,
    page: &CursorPageRequest,
) -> FileResult<Option<String>> {
    row.map(|row| Ok(encode_cursor(sort.cursor_value(row, default_quota)?, &row.space_id, fingerprint, page)))
        .transpose()
}
