use std::collections::BTreeMap;

use sqlx::{PgPool, Postgres, QueryBuilder, Transaction, query, query_scalar};
use time::{Date, OffsetDateTime};

use crate::{
    application::{ObservabilityError, ObservabilityResult, localized},
    domain::{NewSystemLog, SystemLogFilter},
};

use super::{mapping, query as log_query};

const INSERT_PREFIX: &str = "INSERT INTO sys_system_log (id,occurred_at,level,target,message,fields) ";
const EXPIRED_BATCH_SQL: &str = "WITH candidates AS (SELECT occurred_at,id FROM sys_system_log WHERE occurred_at<$1 ORDER BY occurred_at ASC,id ASC LIMIT $2 FOR UPDATE), deleted AS (DELETE FROM sys_system_log AS log USING candidates WHERE log.occurred_at=candidates.occurred_at AND log.id=candidates.id RETURNING log.id) SELECT COUNT(*) FROM deleted";

pub(super) async fn insert_batch(pool: &PgPool, events: &[NewSystemLog]) -> ObservabilityResult<()> {
    if events.is_empty() {
        return Ok(());
    }
    let mut transaction = pool.begin().await.map_err(mapping::sqlx_error)?;
    ensure_partitions(&mut transaction, events).await?;
    insert_events(&mut transaction, events).await?;
    transaction.commit().await.map_err(mapping::sqlx_error)
}

pub(super) async fn delete_ids(pool: &PgPool, ids: &[String]) -> ObservabilityResult<()> {
    let mut transaction = pool.begin().await.map_err(mapping::sqlx_error)?;
    ensure_ids_exist(&mut transaction, ids).await?;
    query("DELETE FROM sys_system_log WHERE id=ANY($1)")
        .bind(ids)
        .execute(&mut *transaction)
        .await
        .map_err(mapping::sqlx_error)?;
    transaction.commit().await.map_err(mapping::sqlx_error)
}

pub(super) async fn delete_filtered_batch(pool: &PgPool, filter: SystemLogFilter, limit: u64) -> ObservabilityResult<u64> {
    let limit = valid_batch_limit(limit)?;
    let mut transaction = pool.begin().await.map_err(mapping::sqlx_error)?;
    let mut builder = QueryBuilder::<Postgres>::new("WITH candidates AS (SELECT occurred_at,id FROM sys_system_log WHERE TRUE");
    log_query::push_filters(&mut builder, &filter);
    builder
        .push(" ORDER BY occurred_at ASC,id ASC LIMIT ")
        .push_bind(limit)
        .push(" FOR UPDATE), deleted AS (DELETE FROM sys_system_log AS log USING candidates WHERE log.occurred_at=candidates.occurred_at AND log.id=candidates.id RETURNING log.id) SELECT COUNT(*) FROM deleted");
    let count = builder
        .build_query_scalar::<i64>()
        .fetch_one(&mut *transaction)
        .await
        .map_err(mapping::sqlx_error)?;
    transaction.commit().await.map_err(mapping::sqlx_error)?;
    u64::try_from(count).map_err(|error| ObservabilityError::Infrastructure(format!("system log manual cleanup count conversion failed: {error}")))
}

pub(super) async fn delete_expired_batch(pool: &PgPool, cutoff: OffsetDateTime, limit: u64) -> ObservabilityResult<u64> {
    let limit = valid_batch_limit(limit)?;
    let mut transaction = pool.begin().await.map_err(mapping::sqlx_error)?;
    let count = query_scalar::<_, i64>(EXPIRED_BATCH_SQL)
        .bind(cutoff)
        .bind(limit)
        .fetch_one(&mut *transaction)
        .await
        .map_err(mapping::sqlx_error)?;
    transaction.commit().await.map_err(mapping::sqlx_error)?;
    u64::try_from(count).map_err(|error| ObservabilityError::Infrastructure(format!("system log cleanup count conversion failed: {error}")))
}

fn valid_batch_limit(limit: u64) -> ObservabilityResult<i64> {
    let limit = i64::try_from(limit).map_err(|error| ObservabilityError::Infrastructure(format!("system log cleanup limit conversion failed: {error}")))?;
    if limit <= 0 {
        return Err(ObservabilityError::InvalidInput(localized("errors.observability.invalid_cleanup_batch_size")));
    }
    Ok(limit)
}

async fn ensure_partitions(transaction: &mut Transaction<'_, Postgres>, events: &[NewSystemLog]) -> ObservabilityResult<()> {
    let partitions = partition_samples(events);
    for occurred_at in partitions.into_values() {
        query("SELECT ensure_system_log_partition($1)")
            .bind(occurred_at)
            .execute(&mut **transaction)
            .await
            .map_err(mapping::sqlx_error)?;
    }
    Ok(())
}

async fn insert_events(transaction: &mut Transaction<'_, Postgres>, events: &[NewSystemLog]) -> ObservabilityResult<()> {
    let mut builder = QueryBuilder::<Postgres>::new(INSERT_PREFIX);
    let rows = builder.push_values(events, |mut row, event| {
        row.push_bind(&event.id)
            .push_bind(event.occurred_at)
            .push_bind(event.level.code())
            .push_bind(&event.target)
            .push_bind(&event.message)
            .push_bind(&event.fields);
    });
    rows.push(" ON CONFLICT DO NOTHING");
    builder.build().execute(&mut **transaction).await.map_err(mapping::sqlx_error)?;
    Ok(())
}

async fn ensure_ids_exist(transaction: &mut Transaction<'_, Postgres>, ids: &[String]) -> ObservabilityResult<()> {
    let existing = query_scalar::<_, String>("SELECT id FROM sys_system_log WHERE id=ANY($1) ORDER BY id FOR UPDATE")
        .bind(ids)
        .fetch_all(&mut **transaction)
        .await
        .map_err(mapping::sqlx_error)?;
    if existing.len() != ids.len() {
        return Err(ObservabilityError::NotFound);
    }
    Ok(())
}

fn partition_samples(events: &[NewSystemLog]) -> BTreeMap<Date, OffsetDateTime> {
    events.iter().fold(BTreeMap::new(), |mut samples, event| {
        samples.entry(event.occurred_at.date()).or_insert(event.occurred_at);
        samples
    })
}
