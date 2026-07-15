use kernel::pagination::{CursorDirection, CursorPage};
use sqlx::{Postgres, QueryBuilder};
use storage::{Database, StorageError, database::to_i64};
use types::rbac::Role;

use crate::{
    application::{
        RbacResult, RoleListFilter,
        cursor::{RoleCursor, RoleCursorCodec, TimeIdPoint, point, point_time},
    },
    domain::{DataScope, DataScopeFilter},
    infra::{
        cursor_page::{PageBuildContext, PageNavigation, build_page, navigation},
        mapping::storage_error,
        records::RoleRecord,
    },
};

use super::sql::ROLE_COLUMNS;

struct RolePageWindow {
    snapshot_time: time::OffsetDateTime,
    snapshot_id: String,
    boundary_sort: Option<i64>,
    boundary_id: Option<String>,
    navigation: PageNavigation<crate::application::cursor::RoleBoundary>,
}

#[derive(Clone, Copy)]
struct RoleQuerySpec<'a> {
    filter: &'a RoleListFilter,
    scope: Option<&'a DataScopeFilter>,
}

pub(super) async fn page(database: &Database, filter: RoleListFilter) -> RbacResult<CursorPage<Role>> {
    page_with_scope(database, filter, None).await
}

pub(super) async fn page_scoped(database: &Database, filter: RoleListFilter, scope: DataScopeFilter) -> RbacResult<CursorPage<Role>> {
    page_with_scope(database, filter, Some(scope)).await
}

async fn page_with_scope(database: &Database, filter: RoleListFilter, scope: Option<DataScopeFilter>) -> RbacResult<CursorPage<Role>> {
    let codec = RoleCursorCodec::new(&filter, scope.as_ref())?;
    let decoded = codec.decode(&filter.page)?;
    let spec = RoleQuerySpec {
        filter: &filter,
        scope: scope.as_ref(),
    };
    let snapshot = resolve_snapshot(database, spec, decoded.as_ref().map(|value| &value.snapshot)).await?;
    let Some(snapshot) = snapshot else {
        return Ok(CursorPage::new(Vec::new(), None, None));
    };
    let window = role_window(decoded.as_ref(), &snapshot, filter.page.limit)?;
    let records = fetch_records(database, spec, &window).await?;
    build_page(
        records,
        PageBuildContext {
            codec: &codec,
            snapshot: &snapshot,
            navigation: &window.navigation,
        },
    )
}

async fn resolve_snapshot(database: &Database, spec: RoleQuerySpec<'_>, decoded: Option<&TimeIdPoint>) -> RbacResult<Option<TimeIdPoint>> {
    if let Some(snapshot) = decoded {
        return Ok(Some(snapshot.clone()));
    }
    let mut query = role_query(spec.filter, spec.scope);
    query.push(" ORDER BY r.create_time DESC,r.role_id DESC LIMIT 1");
    let record = query
        .build_query_as::<RoleRecord>()
        .fetch_optional(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)?;
    record.map(|record| point(record.create_time, record.role_id)).transpose()
}

async fn fetch_records(database: &Database, spec: RoleQuerySpec<'_>, window: &RolePageWindow) -> RbacResult<Vec<RoleRecord>> {
    let mut query = role_query(spec.filter, spec.scope);
    push_window(&mut query, window)?;
    query
        .build_query_as::<RoleRecord>()
        .fetch_all(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)
}

pub(super) fn role_query(filter: &RoleListFilter, scope: Option<&DataScopeFilter>) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new("SELECT ");
    if scope.is_some() {
        query.push("DISTINCT ");
    }
    query.push(ROLE_COLUMNS).push(" FROM sys_role r");
    if scope.is_some() {
        query.push(
            " LEFT JOIN sys_user_role ur ON ur.role_id=r.role_id LEFT JOIN sys_user u ON u.user_id=ur.user_id LEFT JOIN sys_dept d ON d.dept_id=u.dept_id",
        );
    }
    query.push(" WHERE r.del_flag='0'");
    push_filters(&mut query, filter);
    if let Some(scope) = scope {
        push_scope(&mut query, scope);
    }
    query
}

fn push_filters(query: &mut QueryBuilder<Postgres>, filter: &RoleListFilter) {
    if let Some(value) = &filter.role_name {
        query.push(" AND r.role_name ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
    }
    if let Some(value) = &filter.role_key {
        query.push(" AND r.role_key ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
    }
    if let Some(value) = &filter.status {
        query.push(" AND r.status=").push_bind(value.clone());
    }
    if let Some(value) = filter.system {
        query.push(" AND r.system=").push_bind(value);
    }
    if let Some(value) = filter.begin_time {
        query.push(" AND r.create_time>=").push_bind(value);
    }
    if let Some(value) = filter.end_time {
        query.push(" AND r.create_time<=").push_bind(value);
    }
}

fn push_scope(query: &mut QueryBuilder<Postgres>, scope: &DataScopeFilter) {
    match scope.data_scope {
        DataScope::All => {}
        DataScope::Custom => {
            query.push(" AND u.dept_id=ANY(").push_bind(scope.dept_ids.clone()).push(")");
        }
        DataScope::Department => push_department_scope(query, scope.dept_id.clone()),
        DataScope::DepartmentAndChildren => push_department_tree_scope(query, scope.dept_id.clone()),
        DataScope::SelfOnly => {
            query.push(" AND u.user_id=").push_bind(scope.user_id.clone());
        }
    }
}

fn push_department_scope(query: &mut QueryBuilder<Postgres>, dept_id: Option<String>) {
    match dept_id {
        Some(dept_id) => {
            query.push(" AND u.dept_id=").push_bind(dept_id);
        }
        None => {
            query.push(" AND FALSE");
        }
    }
}

fn push_department_tree_scope(query: &mut QueryBuilder<Postgres>, dept_id: Option<String>) {
    let Some(dept_id) = dept_id else {
        query.push(" AND FALSE");
        return;
    };
    query.push(" AND (u.dept_id=").push_bind(dept_id.clone());
    query.push(" OR (',' || d.ancestors || ',') LIKE '%,' || ").push_bind(dept_id).push(" || ',%')");
}

fn role_window(decoded: Option<&RoleCursor>, snapshot: &TimeIdPoint, limit: u64) -> RbacResult<RolePageWindow> {
    Ok(RolePageWindow {
        snapshot_time: point_time(snapshot)?,
        snapshot_id: snapshot.id.clone(),
        boundary_sort: decoded.map(|cursor| cursor.boundary.role_sort),
        boundary_id: decoded.map(|cursor| cursor.boundary.role_id.clone()),
        navigation: navigation(decoded, limit),
    })
}

fn push_window(query: &mut QueryBuilder<Postgres>, window: &RolePageWindow) -> RbacResult<()> {
    query.push(" AND (r.create_time,r.role_id)<=(").push_bind(window.snapshot_time);
    query.push(",").push_bind(window.snapshot_id.clone()).push(")");
    if let (Some(role_sort), Some(role_id)) = (window.boundary_sort, window.boundary_id.clone()) {
        let operator = match window.navigation.direction {
            CursorDirection::Next => ">",
            CursorDirection::Previous => "<",
        };
        query.push(" AND (r.role_sort,r.role_id)").push(operator).push("(").push_bind(role_sort);
        query.push(",").push_bind(role_id).push(")");
    }
    let order = match window.navigation.direction {
        CursorDirection::Next => "ASC",
        CursorDirection::Previous => "DESC",
    };
    query.push(" ORDER BY r.role_sort ").push(order).push(",r.role_id ").push(order);
    let fetch_limit = window.navigation.limit.checked_add(1).ok_or_else(|| numeric_error("cursor limit overflow"))?;
    query.push(" LIMIT ").push_bind(to_i64(fetch_limit).map_err(numeric_error)?);
    Ok(())
}

fn numeric_error(error: impl std::fmt::Display) -> crate::application::RbacError {
    crate::application::RbacError::Infrastructure(format!("role cursor numeric conversion failed: {error}"))
}

#[cfg(test)]
mod tests {
    use kernel::pagination::{CursorDirection, CursorPageRequest, DecodedCursor};

    use super::*;
    use crate::application::cursor::RoleBoundary;

    #[test]
    fn role_page_uses_create_snapshot_and_business_order_boundary() {
        let snapshot = TimeIdPoint {
            time_micros: 0,
            id: "z".into(),
        };
        let decoded = DecodedCursor {
            direction: CursorDirection::Next,
            boundary: RoleBoundary {
                role_sort: 10,
                role_id: "a".into(),
            },
            snapshot: snapshot.clone(),
        };
        let window = role_window(Some(&decoded), &snapshot, 20).unwrap();
        let mut query = role_query(&role_filter(), None);
        push_window(&mut query, &window).unwrap();
        let sql = query.sql();
        let sql = sql.as_str();

        assert!(sql.contains("(r.create_time,r.role_id)<="));
        assert!(sql.contains("(r.role_sort,r.role_id)>("));
        assert!(sql.contains("ORDER BY r.role_sort ASC,r.role_id ASC"));
        assert!(!sql.contains("OFFSET"));
        assert!(!sql.contains("COUNT("));
    }

    fn role_filter() -> RoleListFilter {
        RoleListFilter {
            page: CursorPageRequest::default(),
            role_name: None,
            role_key: None,
            status: None,
            system: None,
            begin_time: None,
            end_time: None,
        }
    }
}
