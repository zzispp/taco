use kernel::pagination::{CursorDirection, CursorPage};
use rbac::domain::{DataScope, DataScopeFilter};
use sqlx::{Postgres, QueryBuilder};
use storage::{Database, StorageError, database::to_i64};

use crate::{
    application::{DeptListFilter, SystemBoundary, SystemCursorCodec, SystemResult, TimeIdPoint, point_time},
    domain::Dept,
    infra::{
        cursor_page::{CursorRecord, PageBuildContext, PageNavigation, build_page, navigation},
        mapping::storage_error,
        record::DeptRecord,
    },
};

use super::COLUMNS;

struct DeptPageWindow {
    snapshot_time: time::OffsetDateTime,
    snapshot_id: String,
    boundary_parent: Option<String>,
    boundary_order: Option<i64>,
    boundary_id: Option<String>,
    navigation: PageNavigation,
}

struct DeptSnapshotQuery<'a> {
    filter: &'a DeptListFilter,
    scope: Option<&'a DataScopeFilter>,
    decoded: Option<&'a TimeIdPoint>,
}

pub(super) async fn page(database: &Database, filter: DeptListFilter, scope: Option<DataScopeFilter>) -> SystemResult<CursorPage<Dept>> {
    let codec = SystemCursorCodec::dept(&filter, scope.as_ref())?;
    let decoded = codec.decode(&filter.page)?;
    let snapshot = resolve_snapshot(
        database,
        DeptSnapshotQuery {
            filter: &filter,
            scope: scope.as_ref(),
            decoded: decoded.as_ref().map(|cursor| &cursor.snapshot),
        },
    )
    .await?;
    let Some(snapshot) = snapshot else {
        return Ok(CursorPage::new(Vec::new(), None, None));
    };
    let window = dept_window(decoded.as_ref(), &snapshot, filter.page.limit)?;
    let mut query = filtered_query(&filter, scope.as_ref());
    push_window(&mut query, &window)?;
    let records = query
        .build_query_as::<DeptRecord>()
        .fetch_all(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)?;
    build_page(
        records,
        PageBuildContext {
            codec: &codec,
            snapshot: &snapshot,
            navigation: &window.navigation,
        },
    )
}

async fn resolve_snapshot(database: &Database, request: DeptSnapshotQuery<'_>) -> SystemResult<Option<TimeIdPoint>> {
    if let Some(snapshot) = request.decoded {
        return Ok(Some(snapshot.clone()));
    }
    let mut query = filtered_query(request.filter, request.scope);
    query.push(" ORDER BY create_time DESC,dept_id DESC LIMIT 1");
    let record = query
        .build_query_as::<DeptRecord>()
        .fetch_optional(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)?;
    record.map(|record| record.snapshot()).transpose()
}

fn filtered_query(filter: &DeptListFilter, scope: Option<&DataScopeFilter>) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new(format!("SELECT {COLUMNS} FROM sys_dept d WHERE d.del_flag='0'"));
    push_filters(&mut query, filter);
    if let Some(scope) = scope {
        push_scope(&mut query, scope);
    }
    query
}

fn push_filters(query: &mut QueryBuilder<Postgres>, filter: &DeptListFilter) {
    for (column, value) in [
        ("d.dept_name", &filter.dept_name),
        ("d.leader", &filter.leader),
        ("d.phone", &filter.phone),
        ("d.email", &filter.email),
    ] {
        if let Some(value) = value {
            query.push(" AND ").push(column).push(" ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
        }
    }
    if let Some(value) = &filter.status {
        query.push(" AND d.status=").push_bind(value.clone());
    }
    if let Some(value) = filter.begin_time {
        query.push(" AND d.create_time>=").push_bind(value);
    }
    if let Some(value) = filter.end_time {
        query.push(" AND d.create_time<=").push_bind(value);
    }
}

fn push_scope(query: &mut QueryBuilder<Postgres>, scope: &DataScopeFilter) {
    match scope.data_scope {
        DataScope::All => {}
        DataScope::Custom => {
            query.push(" AND d.dept_id=ANY(").push_bind(scope.dept_ids.clone()).push(")");
        }
        DataScope::Department | DataScope::SelfOnly => push_department(query, scope.dept_id.clone()),
        DataScope::DepartmentAndChildren => push_department_tree(query, scope.dept_id.clone()),
    }
}

fn push_department(query: &mut QueryBuilder<Postgres>, dept_id: Option<String>) {
    match dept_id {
        Some(id) => {
            query.push(" AND d.dept_id=").push_bind(id);
        }
        None => {
            query.push(" AND FALSE");
        }
    }
}

fn push_department_tree(query: &mut QueryBuilder<Postgres>, dept_id: Option<String>) {
    let Some(id) = dept_id else {
        query.push(" AND FALSE");
        return;
    };
    query.push(" AND (d.dept_id=").push_bind(id.clone());
    query.push(" OR (',' || d.ancestors || ',') LIKE '%,' || ").push_bind(id).push(" || ',%')");
}

fn dept_window(decoded: Option<&crate::application::SystemDecodedCursor>, snapshot: &TimeIdPoint, limit: u64) -> SystemResult<DeptPageWindow> {
    let (parent, order, id) = match decoded.map(|cursor| &cursor.boundary) {
        Some(SystemBoundary::Dept { parent_id, order_num, dept_id }) => (Some(parent_id.clone()), Some(*order_num), Some(dept_id.clone())),
        _ => (None, None, None),
    };
    Ok(DeptPageWindow {
        snapshot_time: point_time(snapshot)?,
        snapshot_id: snapshot.id.clone(),
        boundary_parent: parent,
        boundary_order: order,
        boundary_id: id,
        navigation: navigation(decoded, limit),
    })
}

fn push_window(query: &mut QueryBuilder<Postgres>, window: &DeptPageWindow) -> SystemResult<()> {
    query.push(" AND (d.create_time,d.dept_id)<=(").push_bind(window.snapshot_time);
    query.push(",").push_bind(window.snapshot_id.clone()).push(")");
    if let (Some(parent), Some(order_num), Some(id)) = (window.boundary_parent.clone(), window.boundary_order, window.boundary_id.clone()) {
        let operator = if window.navigation.direction == CursorDirection::Next { ">" } else { "<" };
        query
            .push(" AND (d.parent_id,d.order_num,d.dept_id)")
            .push(operator)
            .push("(")
            .push_bind(parent);
        query.push(",").push_bind(order_num).push(",").push_bind(id).push(")");
    }
    let order = if window.navigation.direction == CursorDirection::Next {
        "ASC"
    } else {
        "DESC"
    };
    query
        .push(" ORDER BY d.parent_id ")
        .push(order)
        .push(",d.order_num ")
        .push(order)
        .push(",d.dept_id ")
        .push(order);
    let fetch_limit = window.navigation.limit.checked_add(1).ok_or_else(|| numeric_error("cursor limit overflow"))?;
    query.push(" LIMIT ").push_bind(to_i64(fetch_limit).map_err(numeric_error)?);
    Ok(())
}

fn numeric_error(error: impl std::fmt::Display) -> crate::application::SystemError {
    crate::application::SystemError::Infrastructure(format!("dept cursor numeric conversion failed: {error}"))
}

#[cfg(test)]
mod tests {
    use kernel::pagination::CursorPageRequest;

    use super::*;

    #[test]
    fn dept_page_uses_snapshot_and_business_sort_keyset() {
        let filter = DeptListFilter {
            page: CursorPageRequest::default(),
            dept_name: None,
            leader: None,
            phone: None,
            email: None,
            status: None,
            begin_time: None,
            end_time: None,
        };
        let snapshot = crate::application::point(time::OffsetDateTime::UNIX_EPOCH, "dept-z".into()).unwrap();
        let window = dept_window(None, &snapshot, filter.page.limit).unwrap();
        let mut query = filtered_query(&filter, None);
        push_window(&mut query, &window).unwrap();
        let sql = query.into_string();

        assert!(sql.contains("(d.create_time,d.dept_id)<="));
        assert!(sql.contains("ORDER BY d.parent_id ASC,d.order_num ASC,d.dept_id ASC"));
        assert!(sql.contains("LIMIT"));
        assert!(!sql.contains("OFFSET"));
    }
}
