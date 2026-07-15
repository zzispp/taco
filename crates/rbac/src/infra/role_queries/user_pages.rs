use kernel::pagination::{CursorDirection, CursorPage};
use sqlx::{Postgres, QueryBuilder};
use storage::{Database, StorageError, database::to_i64};
use types::rbac::RoleUser;

use crate::{
    application::{
        RbacResult, RoleUserListFilter,
        cursor::{RoleUserCursor, RoleUserCursorCodec, TimeIdPoint, point, point_time},
    },
    domain::{DataScope, DataScopeFilter},
    infra::{
        cursor_page::{PageBuildContext, PageNavigation, build_page, navigation},
        mapping::storage_error,
        records::RoleUserRecord,
    },
};

const ROLE_USER_COLUMNS: &str = "u.user_id,u.user_name AS username,u.nick_name,u.dept_id,u.phonenumber,u.email,u.status,u.create_time";

struct RoleUserPageWindow {
    snapshot_time: time::OffsetDateTime,
    snapshot_id: String,
    boundary_time: Option<time::OffsetDateTime>,
    boundary_id: Option<String>,
    navigation: PageNavigation<TimeIdPoint>,
}

#[derive(Clone, Copy)]
struct RoleUserQuerySpec<'a> {
    filter: &'a RoleUserListFilter,
    scope: Option<&'a DataScopeFilter>,
}

pub(super) async fn page(database: &Database, filter: RoleUserListFilter, scope: Option<DataScopeFilter>) -> RbacResult<CursorPage<RoleUser>> {
    let codec = RoleUserCursorCodec::new(&filter, scope.as_ref())?;
    let decoded = codec.decode(&filter.page)?;
    let spec = RoleUserQuerySpec {
        filter: &filter,
        scope: scope.as_ref(),
    };
    let snapshot = resolve_snapshot(database, spec, decoded.as_ref().map(|value| &value.snapshot)).await?;
    let Some(snapshot) = snapshot else {
        return Ok(CursorPage::new(Vec::new(), None, None));
    };
    let window = role_user_window(decoded.as_ref(), &snapshot, filter.page.limit)?;
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

async fn resolve_snapshot(database: &Database, spec: RoleUserQuerySpec<'_>, decoded: Option<&TimeIdPoint>) -> RbacResult<Option<TimeIdPoint>> {
    if let Some(snapshot) = decoded {
        return Ok(Some(snapshot.clone()));
    }
    let mut query = role_user_query(spec.filter, spec.scope);
    query.push(" ORDER BY u.create_time DESC,u.user_id DESC LIMIT 1");
    let record = query
        .build_query_as::<RoleUserRecord>()
        .fetch_optional(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)?;
    record.map(|record| point(record.create_time, record.user_id)).transpose()
}

async fn fetch_records(database: &Database, spec: RoleUserQuerySpec<'_>, window: &RoleUserPageWindow) -> RbacResult<Vec<RoleUserRecord>> {
    let mut query = role_user_query(spec.filter, spec.scope);
    push_window(&mut query, window)?;
    query
        .build_query_as::<RoleUserRecord>()
        .fetch_all(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)
}

fn role_user_query(filter: &RoleUserListFilter, scope: Option<&DataScopeFilter>) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new("SELECT ");
    query.push(ROLE_USER_COLUMNS);
    query.push(" FROM sys_user u LEFT JOIN sys_dept d ON d.dept_id=u.dept_id WHERE u.del_flag='0'");
    if let Some(value) = &filter.username {
        query.push(" AND u.user_name ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
    }
    if let Some(value) = &filter.phonenumber {
        query.push(" AND u.phonenumber ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
    }
    push_allocation(&mut query, &filter.role_id, filter.allocated);
    if let Some(scope) = scope {
        push_scope(&mut query, scope);
    }
    query
}

fn push_allocation(query: &mut QueryBuilder<Postgres>, role_id: &str, allocated: bool) {
    if allocated {
        query.push(" AND EXISTS (SELECT 1 FROM sys_user_role ur WHERE ur.user_id=u.user_id AND ur.role_id=");
    } else {
        query.push(" AND NOT EXISTS (SELECT 1 FROM sys_user_role ur WHERE ur.user_id=u.user_id AND ur.role_id=");
    }
    query.push_bind(role_id.to_owned()).push(")");
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

fn role_user_window(decoded: Option<&RoleUserCursor>, snapshot: &TimeIdPoint, limit: u64) -> RbacResult<RoleUserPageWindow> {
    Ok(RoleUserPageWindow {
        snapshot_time: point_time(snapshot)?,
        snapshot_id: snapshot.id.clone(),
        boundary_time: decoded.map(|cursor| point_time(&cursor.boundary)).transpose()?,
        boundary_id: decoded.map(|cursor| cursor.boundary.id.clone()),
        navigation: navigation(decoded, limit),
    })
}

fn push_window(query: &mut QueryBuilder<Postgres>, window: &RoleUserPageWindow) -> RbacResult<()> {
    query.push(" AND (u.create_time,u.user_id)<=(").push_bind(window.snapshot_time);
    query.push(",").push_bind(window.snapshot_id.clone()).push(")");
    if let (Some(time), Some(id)) = (window.boundary_time, window.boundary_id.clone()) {
        let operator = match window.navigation.direction {
            CursorDirection::Next => ">",
            CursorDirection::Previous => "<",
        };
        query.push(" AND (u.create_time,u.user_id)").push(operator).push("(").push_bind(time);
        query.push(",").push_bind(id).push(")");
    }
    let order = match window.navigation.direction {
        CursorDirection::Next => "ASC",
        CursorDirection::Previous => "DESC",
    };
    query.push(" ORDER BY u.create_time ").push(order).push(",u.user_id ").push(order);
    let fetch_limit = window.navigation.limit.checked_add(1).ok_or_else(|| numeric_error("cursor limit overflow"))?;
    query.push(" LIMIT ").push_bind(to_i64(fetch_limit).map_err(numeric_error)?);
    Ok(())
}

fn numeric_error(error: impl std::fmt::Display) -> crate::application::RbacError {
    crate::application::RbacError::Infrastructure(format!("role-user cursor numeric conversion failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::pagination::CursorPageRequest;

    #[test]
    fn allocated_and_unallocated_queries_use_the_same_keyset_order() {
        for allocated in [true, false] {
            let filter = RoleUserListFilter {
                page: CursorPageRequest::default(),
                role_id: "role-1".into(),
                username: None,
                phonenumber: None,
                allocated,
            };
            let mut query = role_user_query(&filter, None);
            query.push(" ORDER BY u.create_time ASC,u.user_id ASC");
            let sql = query.sql();
            let sql = sql.as_str();
            assert!(sql.contains(if allocated { "AND EXISTS" } else { "AND NOT EXISTS" }));
            assert!(sql.contains("u.create_time"));
            assert!(!sql.contains("OFFSET"));
        }
    }
}
