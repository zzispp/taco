use sqlx::{PgPool, query_scalar};
use time::{Date, OffsetDateTime, Time, UtcOffset, macros::format_description};

use crate::application::{ObservabilityError, ObservabilityResult, SystemLogRetentionReport};

use super::{command, mapping};

const BOUNDARY_DELETE_SQL: &str = r#"
WITH candidates AS (
    SELECT occurred_at,id
    FROM sys_system_log
    WHERE occurred_at >= $1 AND occurred_at < $2
    ORDER BY occurred_at ASC,id ASC
    LIMIT $3
    FOR UPDATE
), deleted AS (
    DELETE FROM sys_system_log AS log
    USING candidates
    WHERE log.occurred_at = candidates.occurred_at AND log.id = candidates.id
    RETURNING log.id
)
SELECT COUNT(*) FROM deleted
"#;
const PARTITION_DISCOVERY_SQL: &str = r#"
SELECT child.relname
FROM pg_inherits inheritance
JOIN pg_class child ON child.oid = inheritance.inhrelid
JOIN pg_namespace child_namespace ON child_namespace.oid = child.relnamespace
WHERE inheritance.inhparent = 'public.sys_system_log'::regclass
  AND child_namespace.nspname = 'public'
ORDER BY child.relname
"#;
const PARTITION_PREFIX: &str = "sys_system_log_";

pub(super) async fn cleanup_before(pool: &PgPool, cutoff: OffsetDateTime, boundary_batch_size: u64) -> ObservabilityResult<SystemLogRetentionReport> {
    let cutoff = cutoff.to_offset(UtcOffset::UTC);
    let limit = command::valid_batch_limit(boundary_batch_size)?;
    let partitions = expired_partition_names(pool, cutoff).await?;
    let mut report = SystemLogRetentionReport::default();
    for partition in partitions {
        let deleted = drop_partition(pool, &partition, cutoff)
            .await
            .map_err(|error| partial_cleanup_error(report, error))?;
        if let Some(deleted) = deleted {
            report = record_batch(report, deleted).map_err(|error| partial_cleanup_error(report, error))?;
        }
    }
    cleanup_boundary(pool, cutoff, limit, report).await
}

async fn expired_partition_names(pool: &PgPool, cutoff: OffsetDateTime) -> ObservabilityResult<Vec<String>> {
    let names = query_scalar::<_, String>(PARTITION_DISCOVERY_SQL)
        .fetch_all(pool)
        .await
        .map_err(mapping::sqlx_error)?;
    names
        .into_iter()
        .map(|name| Ok((partition_end(&name)?, name)))
        .filter_map(|value: ObservabilityResult<_>| match value {
            Ok((end, name)) if end <= cutoff => Some(Ok(name)),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn partition_end(name: &str) -> ObservabilityResult<OffsetDateTime> {
    let suffix = name.strip_prefix(PARTITION_PREFIX).ok_or_else(|| invalid_partition_name(name))?;
    let date = Date::parse(suffix, format_description!("[year][month][day]")).map_err(|_| invalid_partition_name(name))?;
    let next = date.next_day().ok_or_else(|| invalid_partition_name(name))?;
    Ok(next.with_time(Time::MIDNIGHT).assume_utc())
}

async fn drop_partition(pool: &PgPool, name: &str, cutoff: OffsetDateTime) -> ObservabilityResult<Option<u64>> {
    let count = query_scalar::<_, Option<i64>>("SELECT drop_expired_system_log_partition($1,$2)")
        .bind(name)
        .bind(cutoff)
        .fetch_one(pool)
        .await
        .map_err(mapping::sqlx_error)?;
    count
        .map(|value| {
            u64::try_from(value).map_err(|error| ObservabilityError::Infrastructure(format!("system log partition row count conversion failed: {error}")))
        })
        .transpose()
}

async fn cleanup_boundary(
    pool: &PgPool,
    cutoff: OffsetDateTime,
    limit: i64,
    mut report: SystemLogRetentionReport,
) -> ObservabilityResult<SystemLogRetentionReport> {
    let boundary_start = cutoff.replace_time(Time::MIDNIGHT);
    if boundary_start == cutoff {
        return Ok(report);
    }
    loop {
        let deleted = delete_boundary_batch(pool, boundary_start, cutoff, limit)
            .await
            .map_err(|error| partial_cleanup_error(report, error))?;
        if deleted == 0 {
            return Ok(report);
        }
        report = record_batch(report, deleted).map_err(|error| partial_cleanup_error(report, error))?;
    }
}

async fn delete_boundary_batch(pool: &PgPool, boundary_start: OffsetDateTime, cutoff: OffsetDateTime, limit: i64) -> ObservabilityResult<u64> {
    let mut transaction = pool.begin().await.map_err(mapping::sqlx_error)?;
    let count = query_scalar::<_, i64>(BOUNDARY_DELETE_SQL)
        .bind(boundary_start)
        .bind(cutoff)
        .bind(limit)
        .fetch_one(&mut *transaction)
        .await
        .map_err(mapping::sqlx_error)?;
    transaction.commit().await.map_err(mapping::sqlx_error)?;
    u64::try_from(count).map_err(|error| ObservabilityError::Infrastructure(format!("system log boundary cleanup count conversion failed: {error}")))
}

fn record_batch(report: SystemLogRetentionReport, deleted: u64) -> ObservabilityResult<SystemLogRetentionReport> {
    Ok(SystemLogRetentionReport {
        deleted: report
            .deleted
            .checked_add(deleted)
            .ok_or_else(|| ObservabilityError::Infrastructure("system log cleanup deleted count overflow".into()))?,
        batches: report
            .batches
            .checked_add(1)
            .ok_or_else(|| ObservabilityError::Infrastructure("system log cleanup batch count overflow".into()))?,
    })
}

fn partial_cleanup_error(report: SystemLogRetentionReport, error: ObservabilityError) -> ObservabilityError {
    if report.batches == 0 {
        return error;
    }
    ObservabilityError::partial_cleanup(report, error.to_string())
}

fn invalid_partition_name(name: &str) -> ObservabilityError {
    ObservabilityError::Infrastructure(format!("invalid system log partition name: {name}"))
}
