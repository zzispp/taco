use kernel::pagination::CursorPage;
use sqlx::{Postgres, QueryBuilder};
use storage::Database;

use crate::application::{FileAccessScope, FileSpaceListRequest, FileSpaceQuery, FileSpaceView};
use crate::domain::SpaceId;
use crate::error::keys;
use crate::{FileError, FileResult};

use super::views::space_view;
use crate::infra::repository_support::{
    CursorBoundary, FilePageContext, PHYSICAL_USAGE_SQL, SpaceRecord, SpaceSortSpec, VIRTUAL_SPACE_CTE, scope_query, space_page_fingerprint, storage_error,
};

pub(in crate::infra) async fn list_spaces(database: &Database, request: FileSpaceListRequest<'_>) -> FileResult<CursorPage<FileSpaceView>> {
    let FileSpaceListRequest {
        actor,
        query: filter,
        page,
        default_quota,
    } = request;
    page.validate().map_err(|_| FileError::InvalidInput(keys::CURSOR_LIMIT_INVALID))?;
    let sort = SpaceSortSpec::from_filter(&filter)?;
    let fingerprint = space_page_fingerprint(actor, &filter);
    let page_context = FilePageContext::new(filter.cursor.as_deref(), &fingerprint, &page)?;
    let mut query = QueryBuilder::<Postgres>::new(VIRTUAL_SPACE_CTE);
    query.push(" SELECT s.space_id,s.owner_user_id,s.owner_name,d.dept_name AS department_name,s.status,s.active_bytes,s.trashed_bytes,s.reserved_bytes,s.quota_override_bytes,");
    query.push(PHYSICAL_USAGE_SQL);
    query.push(" AS physical_bytes,s.updated_at FROM visible_spaces s LEFT JOIN sys_dept d ON d.dept_id=s.owner_dept_id WHERE");
    scope_query(&mut query, actor, "s");
    append_filters(&mut query, &filter);
    sort.push_cursor_bound(&mut query, page_context.cursor(), default_quota)?;
    sort.push_order(&mut query, default_quota, page_context.direction())?;
    query.push(" LIMIT ").push_bind(page_context.query_limit()?);
    let rows = query.build_query_as::<SpaceRecord>().fetch_all(database.pool()).await.map_err(storage_error)?;
    let slice = page_context.build_page(rows, |row| {
        Ok(CursorBoundary::new(sort.cursor_value(row, default_quota)?, row.space_id.clone()))
    })?;
    Ok(CursorPage::new(
        slice.records.into_iter().map(|row| space_view(row, default_quota)).collect(),
        slice.next_cursor,
        slice.previous_cursor,
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
