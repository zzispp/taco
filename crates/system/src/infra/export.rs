use sqlx::PgConnection;
use storage::{StorageError, database::to_i64};

use crate::{
    application::{
        ConfigListFilter, DictDataListFilter, DictTypeListFilter, PostListFilter, SystemBoundary, SystemExportBatch, SystemExportKind, SystemExportRequest,
        SystemExportSink, SystemResult,
    },
    infra::{
        config, dict,
        mapping::{config as map_config, dict_data, dict_type, post, storage_error},
        post as post_queries,
        record::{ConfigRecord, DictDataRecord, DictTypeRecord, PostRecord},
    },
};

pub(super) async fn export(database: &storage::Database, request: SystemExportRequest, sink: &mut dyn SystemExportSink) -> SystemResult<()> {
    let mut transaction = database.begin_consistent_snapshot().await.map_err(storage_error)?;
    let mut boundary = None;
    loop {
        let fetched = fetch_batch(
            &mut transaction,
            &request.kind,
            BatchWindow {
                boundary: boundary.as_ref(),
                limit: request.batch_size,
            },
        )
        .await
        .map_err(storage_error)?;
        if fetched.batch.is_empty() {
            break;
        }
        let loaded = u64::try_from(fetched.batch.len()).map_err(numeric_error)?;
        boundary = fetched.boundary;
        sink.append(fetched.batch)?;
        if loaded < request.batch_size {
            break;
        }
    }
    transaction.commit().await.map_err(StorageError::from).map_err(storage_error)
}

#[derive(Clone, Copy)]
struct BatchWindow<'a> {
    boundary: Option<&'a SystemBoundary>,
    limit: u64,
}

struct FetchedBatch {
    batch: SystemExportBatch,
    boundary: Option<SystemBoundary>,
}

async fn fetch_batch(connection: &mut PgConnection, kind: &SystemExportKind, window: BatchWindow<'_>) -> storage::StorageResult<FetchedBatch> {
    match kind {
        SystemExportKind::Posts(filter) => post_batch(connection, filter, window).await,
        SystemExportKind::DictTypes(filter) => dict_type_batch(connection, filter, window).await,
        SystemExportKind::DictData(filter) => dict_data_batch(connection, filter, window).await,
        SystemExportKind::Configs(filter) => config_batch(connection, filter, window).await,
    }
}

async fn post_batch(connection: &mut PgConnection, filter: &PostListFilter, window: BatchWindow<'_>) -> storage::StorageResult<FetchedBatch> {
    let mut query = post_queries::filtered_query(filter);
    if let Some(SystemBoundary::Post { post_sort, post_id }) = window.boundary {
        query.push(" AND (post_sort,post_id)>(").push_bind(*post_sort);
        query.push(",").push_bind(post_id.clone()).push(")");
    }
    query.push(" ORDER BY post_sort ASC,post_id ASC LIMIT ").push_bind(to_i64(window.limit)?);
    let records = query.build_query_as::<PostRecord>().fetch_all(connection).await?;
    let boundary = records.last().map(|record| SystemBoundary::Post {
        post_sort: record.post_sort,
        post_id: record.post_id.clone(),
    });
    let items = records.into_iter().map(post).collect::<storage::StorageResult<Vec<_>>>()?;
    Ok(FetchedBatch {
        batch: SystemExportBatch::Posts(items),
        boundary,
    })
}

async fn dict_type_batch(connection: &mut PgConnection, filter: &DictTypeListFilter, window: BatchWindow<'_>) -> storage::StorageResult<FetchedBatch> {
    let mut query = dict::type_filtered_query(filter);
    if let Some(SystemBoundary::DictType { dict_id }) = window.boundary {
        query.push(" AND dict_id>").push_bind(dict_id.clone());
    }
    query.push(" ORDER BY dict_id ASC LIMIT ").push_bind(to_i64(window.limit)?);
    let records = query.build_query_as::<DictTypeRecord>().fetch_all(connection).await?;
    let boundary = records.last().map(|record| SystemBoundary::DictType {
        dict_id: record.dict_id.clone(),
    });
    let items = records.into_iter().map(dict_type).collect::<storage::StorageResult<Vec<_>>>()?;
    Ok(FetchedBatch {
        batch: SystemExportBatch::DictTypes(items),
        boundary,
    })
}

async fn dict_data_batch(connection: &mut PgConnection, filter: &DictDataListFilter, window: BatchWindow<'_>) -> storage::StorageResult<FetchedBatch> {
    let mut query = dict::data_filtered_query(filter);
    if let Some(SystemBoundary::DictData { dict_sort, dict_code }) = window.boundary {
        query.push(" AND (dict_sort,dict_code)>(").push_bind(*dict_sort);
        query.push(",").push_bind(dict_code.clone()).push(")");
    }
    query.push(" ORDER BY dict_sort ASC,dict_code ASC LIMIT ").push_bind(to_i64(window.limit)?);
    let records = query.build_query_as::<DictDataRecord>().fetch_all(connection).await?;
    let boundary = records.last().map(|record| SystemBoundary::DictData {
        dict_sort: record.dict_sort,
        dict_code: record.dict_code.clone(),
    });
    let items = records.into_iter().map(dict_data).collect::<storage::StorageResult<Vec<_>>>()?;
    Ok(FetchedBatch {
        batch: SystemExportBatch::DictData(items),
        boundary,
    })
}

async fn config_batch(connection: &mut PgConnection, filter: &ConfigListFilter, window: BatchWindow<'_>) -> storage::StorageResult<FetchedBatch> {
    let mut query = config::filtered_query(filter);
    if let Some(SystemBoundary::Config { config_id }) = window.boundary {
        query.push(" AND config_id>").push_bind(config_id.clone());
    }
    query.push(" ORDER BY config_id ASC LIMIT ").push_bind(to_i64(window.limit)?);
    let records = query.build_query_as::<ConfigRecord>().fetch_all(connection).await?;
    let boundary = records.last().map(|record| SystemBoundary::Config {
        config_id: record.config_id.clone(),
    });
    let items = records.into_iter().map(map_config).collect::<storage::StorageResult<Vec<_>>>()?;
    Ok(FetchedBatch {
        batch: SystemExportBatch::Configs(items),
        boundary,
    })
}

fn numeric_error(error: impl std::fmt::Display) -> crate::application::SystemError {
    crate::application::SystemError::Infrastructure(format!("system export numeric conversion error: {error}"))
}
