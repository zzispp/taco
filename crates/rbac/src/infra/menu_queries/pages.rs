use kernel::pagination::{CursorDirection, CursorPage};
use sqlx::{Postgres, QueryBuilder};
use storage::{Database, StorageError, database::to_i64};
use types::rbac::Menu;

use crate::{
    application::{
        MenuListFilter, RbacResult,
        cursor::{MenuBoundary, MenuCursor, MenuCursorCodec, TimeIdPoint, point, point_time},
    },
    infra::{
        cursor_page::{PageBuildContext, PageNavigation, build_page, navigation},
        mapping::storage_error,
        records::MenuRecord,
    },
};

use super::MENU_COLUMNS;

struct MenuPageWindow {
    snapshot_time: time::OffsetDateTime,
    snapshot_id: String,
    boundary_parent_id: Option<String>,
    boundary_order: Option<i64>,
    boundary_id: Option<String>,
    navigation: PageNavigation<MenuBoundary>,
}

pub(super) async fn page(database: &Database, filter: MenuListFilter) -> RbacResult<CursorPage<Menu>> {
    let codec = MenuCursorCodec::new(&filter)?;
    let decoded = codec.decode(&filter.page)?;
    let snapshot = resolve_snapshot(database, &filter, decoded.as_ref().map(|value| &value.snapshot)).await?;
    let Some(snapshot) = snapshot else {
        return Ok(CursorPage::new(Vec::new(), None, None));
    };
    let window = menu_window(decoded.as_ref(), &snapshot, filter.page.limit)?;
    let records = fetch_records(database, &filter, &window).await?;
    build_page(
        records,
        PageBuildContext {
            codec: &codec,
            snapshot: &snapshot,
            navigation: &window.navigation,
        },
    )
}

async fn resolve_snapshot(database: &Database, filter: &MenuListFilter, decoded: Option<&TimeIdPoint>) -> RbacResult<Option<TimeIdPoint>> {
    if let Some(snapshot) = decoded {
        return Ok(Some(snapshot.clone()));
    }
    let mut query = menu_query(filter);
    query.push(" ORDER BY create_time DESC,menu_id DESC LIMIT 1");
    let record = query
        .build_query_as::<MenuRecord>()
        .fetch_optional(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)?;
    record.map(|record| point(record.create_time, record.menu_id)).transpose()
}

async fn fetch_records(database: &Database, filter: &MenuListFilter, window: &MenuPageWindow) -> RbacResult<Vec<MenuRecord>> {
    let mut query = menu_query(filter);
    push_window(&mut query, window)?;
    query
        .build_query_as::<MenuRecord>()
        .fetch_all(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)
}

fn menu_query(filter: &MenuListFilter) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new("SELECT ");
    query.push(MENU_COLUMNS).push(" FROM sys_menu WHERE TRUE");
    if let Some(value) = &filter.menu_name {
        query.push(" AND menu_name ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
    }
    if let Some(value) = &filter.status {
        query.push(" AND status=").push_bind(value.clone());
    }
    if let Some(value) = filter.begin_time {
        query.push(" AND create_time>=").push_bind(value);
    }
    if let Some(value) = filter.end_time {
        query.push(" AND create_time<=").push_bind(value);
    }
    query
}

fn menu_window(decoded: Option<&MenuCursor>, snapshot: &TimeIdPoint, limit: u64) -> RbacResult<MenuPageWindow> {
    Ok(MenuPageWindow {
        snapshot_time: point_time(snapshot)?,
        snapshot_id: snapshot.id.clone(),
        boundary_parent_id: decoded.map(|cursor| cursor.boundary.parent_id.clone()),
        boundary_order: decoded.map(|cursor| cursor.boundary.order_num),
        boundary_id: decoded.map(|cursor| cursor.boundary.menu_id.clone()),
        navigation: navigation(decoded, limit),
    })
}

fn push_window(query: &mut QueryBuilder<Postgres>, window: &MenuPageWindow) -> RbacResult<()> {
    query.push(" AND (create_time,menu_id)<=(").push_bind(window.snapshot_time);
    query.push(",").push_bind(window.snapshot_id.clone()).push(")");
    if let (Some(parent), Some(order), Some(id)) = (window.boundary_parent_id.clone(), window.boundary_order, window.boundary_id.clone()) {
        let operator = match window.navigation.direction {
            CursorDirection::Next => ">",
            CursorDirection::Previous => "<",
        };
        query.push(" AND (parent_id,order_num,menu_id)").push(operator).push("(").push_bind(parent);
        query.push(",").push_bind(order).push(",").push_bind(id).push(")");
    }
    let order = match window.navigation.direction {
        CursorDirection::Next => "ASC",
        CursorDirection::Previous => "DESC",
    };
    query
        .push(" ORDER BY parent_id ")
        .push(order)
        .push(",order_num ")
        .push(order)
        .push(",menu_id ")
        .push(order);
    let fetch_limit = window.navigation.limit.checked_add(1).ok_or_else(|| numeric_error("cursor limit overflow"))?;
    query.push(" LIMIT ").push_bind(to_i64(fetch_limit).map_err(numeric_error)?);
    Ok(())
}

fn numeric_error(error: impl std::fmt::Display) -> crate::application::RbacError {
    crate::application::RbacError::Infrastructure(format!("menu cursor numeric conversion failed: {error}"))
}

#[cfg(test)]
mod tests {
    use kernel::pagination::{CursorDirection, CursorPageRequest, DecodedCursor};

    use super::*;

    #[test]
    fn menu_page_uses_create_snapshot_and_tree_order_boundary() {
        let snapshot = TimeIdPoint {
            time_micros: 0,
            id: "z".into(),
        };
        let decoded = DecodedCursor {
            direction: CursorDirection::Next,
            boundary: MenuBoundary {
                parent_id: "1".into(),
                order_num: 10,
                menu_id: "a".into(),
            },
            snapshot: snapshot.clone(),
        };
        let window = menu_window(Some(&decoded), &snapshot, 20).unwrap();
        let mut query = menu_query(&menu_filter());
        push_window(&mut query, &window).unwrap();
        let sql = query.sql();
        let sql = sql.as_str();

        assert!(sql.contains("(create_time,menu_id)<="));
        assert!(sql.contains("(parent_id,order_num,menu_id)>("));
        assert!(sql.contains("ORDER BY parent_id ASC,order_num ASC,menu_id ASC"));
        assert!(!sql.contains("create_time::text"));
        assert!(!sql.contains("OFFSET"));
    }

    fn menu_filter() -> MenuListFilter {
        MenuListFilter {
            page: CursorPageRequest::default(),
            menu_name: None,
            status: None,
            begin_time: None,
            end_time: None,
        }
    }
}
