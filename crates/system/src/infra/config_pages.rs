use kernel::pagination::{CursorDirection, CursorPage};
use sqlx::{Postgres, QueryBuilder};
use storage::{Database, StorageError, database::to_i64};

use crate::{
    application::{ConfigListFilter, SystemBoundary, SystemCursorCodec, SystemResult, TimeIdPoint, point_time},
    domain::ConfigItem,
    infra::{
        cursor_page::{CursorRecord, PageBuildContext, PageNavigation, build_page, navigation},
        mapping::storage_error,
        record::ConfigRecord,
    },
};

use super::COLUMNS;

struct ConfigPageWindow {
    snapshot_time: time::OffsetDateTime,
    snapshot_id: String,
    boundary_id: Option<String>,
    navigation: PageNavigation,
}

pub(super) async fn page(database: &Database, filter: ConfigListFilter) -> SystemResult<CursorPage<ConfigItem>> {
    let codec = SystemCursorCodec::config(&filter)?;
    let decoded = codec.decode(&filter.page)?;
    let snapshot = resolve_snapshot(database, &filter, decoded.as_ref().map(|cursor| &cursor.snapshot)).await?;
    let Some(snapshot) = snapshot else {
        return Ok(CursorPage::new(Vec::new(), None, None));
    };
    let window = config_window(decoded.as_ref(), &snapshot, filter.page.limit)?;
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

async fn resolve_snapshot(database: &Database, filter: &ConfigListFilter, decoded: Option<&TimeIdPoint>) -> SystemResult<Option<TimeIdPoint>> {
    if let Some(snapshot) = decoded {
        return Ok(Some(snapshot.clone()));
    }
    let mut query = filtered_query(filter);
    query.push(" ORDER BY create_time DESC,config_id DESC LIMIT 1");
    let record = query
        .build_query_as::<ConfigRecord>()
        .fetch_optional(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)?;
    record.map(|record| record.snapshot()).transpose()
}

async fn fetch_records(database: &Database, filter: &ConfigListFilter, window: &ConfigPageWindow) -> SystemResult<Vec<ConfigRecord>> {
    let mut query = filtered_query(filter);
    push_window(&mut query, window)?;
    query
        .build_query_as::<ConfigRecord>()
        .fetch_all(database.pool())
        .await
        .map_err(StorageError::from)
        .map_err(storage_error)
}

pub(in crate::infra) fn filtered_query(filter: &ConfigListFilter) -> QueryBuilder<Postgres> {
    let mut query = QueryBuilder::new(format!("SELECT {COLUMNS} FROM sys_config WHERE TRUE"));
    if let Some(value) = &filter.config_name {
        query.push(" AND config_name ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
    }
    if let Some(value) = &filter.config_key {
        query.push(" AND config_key ILIKE '%' || ").push_bind(value.clone()).push(" || '%'");
    }
    if let Some(value) = &filter.config_type {
        query.push(" AND config_type=").push_bind(value.clone());
    }
    if let Some(value) = filter.public_read {
        query.push(" AND public_read=").push_bind(value);
    }
    if let Some(value) = filter.begin_time {
        query.push(" AND create_time>=").push_bind(value);
    }
    if let Some(value) = filter.end_time {
        query.push(" AND create_time<=").push_bind(value);
    }
    query
}

fn config_window(decoded: Option<&crate::application::SystemDecodedCursor>, snapshot: &TimeIdPoint, limit: u64) -> SystemResult<ConfigPageWindow> {
    let boundary_id = match decoded.map(|cursor| &cursor.boundary) {
        Some(SystemBoundary::Config { config_id }) => Some(config_id.clone()),
        _ => None,
    };
    Ok(ConfigPageWindow {
        snapshot_time: point_time(snapshot)?,
        snapshot_id: snapshot.id.clone(),
        boundary_id,
        navigation: navigation(decoded, limit),
    })
}

fn push_window(query: &mut QueryBuilder<Postgres>, window: &ConfigPageWindow) -> SystemResult<()> {
    query.push(" AND (create_time,config_id)<=(").push_bind(window.snapshot_time);
    query.push(",").push_bind(window.snapshot_id.clone()).push(")");
    if let Some(id) = &window.boundary_id {
        let operator = if window.navigation.direction == CursorDirection::Next { ">" } else { "<" };
        query.push(" AND config_id").push(operator).push_bind(id.clone());
    }
    let order = if window.navigation.direction == CursorDirection::Next {
        "ASC"
    } else {
        "DESC"
    };
    query.push(" ORDER BY config_id ").push(order);
    let fetch_limit = window.navigation.limit.checked_add(1).ok_or_else(|| numeric_error("cursor limit overflow"))?;
    query.push(" LIMIT ").push_bind(to_i64(fetch_limit).map_err(numeric_error)?);
    Ok(())
}

fn numeric_error(error: impl std::fmt::Display) -> crate::application::SystemError {
    crate::application::SystemError::Infrastructure(format!("config cursor numeric conversion failed: {error}"))
}

#[cfg(test)]
mod tests {
    use kernel::pagination::CursorPageRequest;

    use super::*;

    #[test]
    fn config_query_uses_native_time_and_no_offset() {
        let sql = filtered_query(&ConfigListFilter {
            page: CursorPageRequest::default(),
            config_name: Some("name".into()),
            config_key: Some("key".into()),
            config_type: None,
            public_read: None,
            begin_time: Some(time::OffsetDateTime::UNIX_EPOCH),
            end_time: None,
        })
        .into_string();
        assert!(sql.contains("config_name ILIKE"));
        assert!(sql.contains("config_key ILIKE"));
        assert!(sql.contains("create_time>="));
        assert!(!sql.contains("OFFSET"));
    }

    #[test]
    fn config_page_uses_snapshot_and_business_sort_keyset() {
        let filter = ConfigListFilter {
            page: CursorPageRequest::default(),
            config_name: None,
            config_key: None,
            config_type: None,
            public_read: None,
            begin_time: None,
            end_time: None,
        };
        let snapshot = crate::application::point(time::OffsetDateTime::UNIX_EPOCH, "config-z".into()).unwrap();
        let window = config_window(None, &snapshot, filter.page.limit).unwrap();
        let mut query = filtered_query(&filter);
        push_window(&mut query, &window).unwrap();
        let sql = query.into_string();

        assert!(sql.contains("(create_time,config_id)<="));
        assert!(sql.contains("ORDER BY config_id ASC"));
        assert!(sql.contains("LIMIT"));
        assert!(!sql.contains("OFFSET"));
    }
}
