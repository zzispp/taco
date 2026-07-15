use kernel::pagination::{CursorDirection, CursorPage};
use sqlx::{Postgres, QueryBuilder};
use storage::{Database, StorageError, database::to_i64};

use crate::{
    application::{DictDataListFilter, DictTypeListFilter, SystemBoundary, SystemCursorCodec, SystemResult, TimeIdPoint, point_time},
    domain::{DictData, DictType},
    infra::{
        cursor_page::{CursorRecord, PageBuildContext, PageNavigation, build_page, navigation},
        mapping::storage_error,
        record::{DictDataRecord, DictTypeRecord},
    },
};

use super::{DATA_COLUMNS, TYPE_COLUMNS};

struct DictPageWindow {
    snapshot_time: time::OffsetDateTime,
    snapshot_id: String,
    boundary_sort: Option<i64>,
    boundary_id: Option<String>,
    navigation: PageNavigation,
}

struct CommonFilters<'a> {
    status: &'a Option<String>,
    begin: Option<time::OffsetDateTime>,
    end: Option<time::OffsetDateTime>,
}

struct DictWindowInput<'a> {
    snapshot: &'a TimeIdPoint,
    boundary_id: Option<String>,
    boundary_sort: Option<i64>,
    decoded: Option<&'a crate::application::SystemDecodedCursor>,
    limit: u64,
}

pub(super) async fn page_types(database: &Database, filter: DictTypeListFilter) -> SystemResult<CursorPage<DictType>> {
    let codec = SystemCursorCodec::dict_type(&filter)?;
    let decoded = codec.decode(&filter.page)?;
    let snapshot = type_snapshot(database, &filter, decoded.as_ref().map(|cursor| &cursor.snapshot)).await?;
    let Some(snapshot) = snapshot else {
        return Ok(CursorPage::new(Vec::new(), None, None));
    };
    let window = type_window(decoded.as_ref(), &snapshot, filter.page.limit)?;
    let mut query = type_filtered_query(&filter);
    push_type_window(&mut query, &window)?;
    let records = fetch::<DictTypeRecord>(database, query).await?;
    build_page(records, page_context(&codec, &snapshot, &window.navigation))
}

pub(super) async fn page_data(database: &Database, filter: DictDataListFilter) -> SystemResult<CursorPage<DictData>> {
    let codec = SystemCursorCodec::dict_data(&filter)?;
    let decoded = codec.decode(&filter.page)?;
    let snapshot = data_snapshot(database, &filter, decoded.as_ref().map(|cursor| &cursor.snapshot)).await?;
    let Some(snapshot) = snapshot else {
        return Ok(CursorPage::new(Vec::new(), None, None));
    };
    let window = data_window(decoded.as_ref(), &snapshot, filter.page.limit)?;
    let mut query = data_filtered_query(&filter);
    push_data_window(&mut query, &window)?;
    let records = fetch::<DictDataRecord>(database, query).await?;
    build_page(records, page_context(&codec, &snapshot, &window.navigation))
}

fn page_context<'a>(codec: &'a SystemCursorCodec, snapshot: &'a TimeIdPoint, navigation: &'a PageNavigation) -> PageBuildContext<'a> {
    PageBuildContext { codec, snapshot, navigation }
}

async fn type_snapshot(database: &Database, filter: &DictTypeListFilter, decoded: Option<&TimeIdPoint>) -> SystemResult<Option<TimeIdPoint>> {
    let mut query = type_filtered_query(filter);
    query.push(" ORDER BY create_time DESC,dict_id DESC LIMIT 1");
    snapshot::<DictTypeRecord>(database, query, decoded).await
}

async fn data_snapshot(database: &Database, filter: &DictDataListFilter, decoded: Option<&TimeIdPoint>) -> SystemResult<Option<TimeIdPoint>> {
    let mut query = data_filtered_query(filter);
    query.push(" ORDER BY create_time DESC,dict_code DESC LIMIT 1");
    snapshot::<DictDataRecord>(database, query, decoded).await
}

async fn snapshot<R>(database: &Database, mut query: QueryBuilder<Postgres>, decoded: Option<&TimeIdPoint>) -> SystemResult<Option<TimeIdPoint>>
where
    R: CursorRecord + for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    if let Some(snapshot) = decoded {
        return Ok(Some(snapshot.clone()));
    }
    let record = query
        .build_query_as::<R>()
        .fetch_optional(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)?;
    record.map(|record| record.snapshot()).transpose()
}

async fn fetch<R>(database: &Database, mut query: QueryBuilder<Postgres>) -> SystemResult<Vec<R>>
where
    R: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
    query
        .build_query_as::<R>()
        .fetch_all(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)
}

pub(in crate::infra) fn type_filtered_query(filter: &DictTypeListFilter) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new(format!("SELECT {TYPE_COLUMNS} FROM sys_dict_type WHERE TRUE"));
    if let Some(value) = &filter.dict_name {
        query.push(" AND dict_name ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
    }
    if let Some(value) = &filter.dict_type {
        query.push(" AND dict_type ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
    }
    push_common_filters(
        &mut query,
        CommonFilters {
            status: &filter.status,
            begin: filter.begin_time,
            end: filter.end_time,
        },
    );
    query
}

pub(in crate::infra) fn data_filtered_query(filter: &DictDataListFilter) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new(format!("SELECT {DATA_COLUMNS} FROM sys_dict_data WHERE TRUE"));
    if let Some(value) = &filter.dict_type {
        query.push(" AND dict_type=").push_bind(value.clone());
    }
    if let Some(value) = &filter.dict_label {
        query.push(" AND dict_label ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
    }
    push_common_filters(
        &mut query,
        CommonFilters {
            status: &filter.status,
            begin: filter.begin_time,
            end: filter.end_time,
        },
    );
    query
}

fn push_common_filters(query: &mut QueryBuilder<Postgres>, filters: CommonFilters<'_>) {
    if let Some(value) = filters.status {
        query.push(" AND status=").push_bind(value.clone());
    }
    if let Some(value) = filters.begin {
        query.push(" AND create_time>=").push_bind(value);
    }
    if let Some(value) = filters.end {
        query.push(" AND create_time<=").push_bind(value);
    }
}

fn type_window(decoded: Option<&crate::application::SystemDecodedCursor>, snapshot: &TimeIdPoint, limit: u64) -> SystemResult<DictPageWindow> {
    let boundary_id = match decoded.map(|cursor| &cursor.boundary) {
        Some(SystemBoundary::DictType { dict_id }) => Some(dict_id.clone()),
        _ => None,
    };
    window(DictWindowInput {
        snapshot,
        boundary_id,
        boundary_sort: None,
        decoded,
        limit,
    })
}

fn data_window(decoded: Option<&crate::application::SystemDecodedCursor>, snapshot: &TimeIdPoint, limit: u64) -> SystemResult<DictPageWindow> {
    let (sort, id) = match decoded.map(|cursor| &cursor.boundary) {
        Some(SystemBoundary::DictData { dict_sort, dict_code }) => (Some(*dict_sort), Some(dict_code.clone())),
        _ => (None, None),
    };
    window(DictWindowInput {
        snapshot,
        boundary_id: id,
        boundary_sort: sort,
        decoded,
        limit,
    })
}

fn window(input: DictWindowInput<'_>) -> SystemResult<DictPageWindow> {
    Ok(DictPageWindow {
        snapshot_time: point_time(input.snapshot)?,
        snapshot_id: input.snapshot.id.clone(),
        boundary_sort: input.boundary_sort,
        boundary_id: input.boundary_id,
        navigation: navigation(input.decoded, input.limit),
    })
}

fn push_type_window(query: &mut QueryBuilder<Postgres>, window: &DictPageWindow) -> SystemResult<()> {
    push_snapshot(query, "dict_id", window);
    if let Some(id) = &window.boundary_id {
        let operator = cursor_operator(window.navigation.direction);
        query.push(" AND dict_id").push(operator).push_bind(id.clone());
    }
    push_order_limit(query, "dict_id", window)
}

fn push_data_window(query: &mut QueryBuilder<Postgres>, window: &DictPageWindow) -> SystemResult<()> {
    push_snapshot(query, "dict_code", window);
    if let (Some(sort), Some(id)) = (window.boundary_sort, window.boundary_id.clone()) {
        let operator = cursor_operator(window.navigation.direction);
        query.push(" AND (dict_sort,dict_code)").push(operator).push("(").push_bind(sort);
        query.push(",").push_bind(id).push(")");
    }
    push_order_limit(query, "dict_sort,dict_code", window)
}

fn push_snapshot(query: &mut QueryBuilder<Postgres>, id_column: &str, window: &DictPageWindow) {
    query.push(" AND (create_time,").push(id_column).push(")<=(").push_bind(window.snapshot_time);
    query.push(",").push_bind(window.snapshot_id.clone()).push(")");
}

fn push_order_limit(query: &mut QueryBuilder<Postgres>, columns: &str, window: &DictPageWindow) -> SystemResult<()> {
    let order = if window.navigation.direction == CursorDirection::Next {
        "ASC"
    } else {
        "DESC"
    };
    query.push(" ORDER BY ").push(columns.replace(',', &format!(" {order},"))).push(" ").push(order);
    let fetch_limit = window.navigation.limit.checked_add(1).ok_or_else(|| numeric_error("cursor limit overflow"))?;
    query.push(" LIMIT ").push_bind(to_i64(fetch_limit).map_err(numeric_error)?);
    Ok(())
}

fn cursor_operator(direction: CursorDirection) -> &'static str {
    if direction == CursorDirection::Next { ">" } else { "<" }
}

fn numeric_error(error: impl std::fmt::Display) -> crate::application::SystemError {
    crate::application::SystemError::Infrastructure(format!("dict cursor numeric conversion failed: {error}"))
}

#[cfg(test)]
mod tests {
    use kernel::pagination::CursorPageRequest;

    use super::*;

    #[test]
    fn dictionary_pages_use_snapshot_and_business_sort_keysets() {
        let type_filter = DictTypeListFilter {
            page: CursorPageRequest::default(),
            dict_name: None,
            dict_type: None,
            status: None,
            begin_time: None,
            end_time: None,
        };
        let data_filter = DictDataListFilter {
            page: CursorPageRequest::default(),
            dict_type: None,
            dict_label: None,
            status: None,
            begin_time: None,
            end_time: None,
        };
        let snapshot = crate::application::point(time::OffsetDateTime::UNIX_EPOCH, "dict-z".into()).unwrap();

        let type_window = type_window(None, &snapshot, type_filter.page.limit).unwrap();
        let mut types = type_filtered_query(&type_filter);
        push_type_window(&mut types, &type_window).unwrap();
        let type_sql = types.into_string();

        let data_window = data_window(None, &snapshot, data_filter.page.limit).unwrap();
        let mut data = data_filtered_query(&data_filter);
        push_data_window(&mut data, &data_window).unwrap();
        let data_sql = data.into_string();

        assert!(type_sql.contains("(create_time,dict_id)<="));
        assert!(type_sql.contains("ORDER BY dict_id ASC"));
        assert!(data_sql.contains("(create_time,dict_code)<="));
        assert!(data_sql.contains("ORDER BY dict_sort ASC,dict_code ASC"));
        assert!(!type_sql.contains("OFFSET"));
        assert!(!data_sql.contains("OFFSET"));
    }
}
