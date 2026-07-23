use audit_contract::{AUDIT_OUTBOX_PAYLOAD_VERSION, AuditOutboxRecord, AuditStream};
use sqlx::{Postgres, Transaction, query};

use crate::{StorageError, StorageResult};

const AUDIT_STREAM_LOCK_PREFIX: &str = "taco.audit_outbox.";
const LOCK_HASH_SEED: i64 = 0;
const SHARED_STREAM_LOCK_SQL: &str = "SELECT pg_advisory_xact_lock_shared(hashtextextended($1 || $2, $3))";
const EXCLUSIVE_STREAM_LOCK_SQL: &str = "SELECT pg_advisory_xact_lock(hashtextextended($1 || $2, $3))";

pub async fn append_audit_record(transaction: &mut Transaction<'_, Postgres>, record: &AuditOutboxRecord) -> StorageResult<()> {
    let payload = record.payload().map_err(payload_error)?;
    lock_audit_stream_shared(transaction, record.stream()).await?;
    query("INSERT INTO audit_outbox (outbox_id,stream,event_type,payload_version,payload,occurred_at) VALUES ($1,$2,$3,$4,$5,$6)")
        .bind(&record.id)
        .bind(record.stream().code())
        .bind(record.event_type())
        .bind(AUDIT_OUTBOX_PAYLOAD_VERSION)
        .bind(payload)
        .bind(record.occurred_at)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

pub async fn lock_audit_stream_shared(transaction: &mut Transaction<'_, Postgres>, stream: AuditStream) -> StorageResult<()> {
    acquire_stream_lock(transaction, stream, SHARED_STREAM_LOCK_SQL).await
}

pub async fn clear_audit_stream(transaction: &mut Transaction<'_, Postgres>, stream: AuditStream) -> StorageResult<()> {
    acquire_stream_lock(transaction, stream, EXCLUSIVE_STREAM_LOCK_SQL).await?;
    query("DELETE FROM audit_outbox WHERE stream=$1")
        .bind(stream.code())
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

async fn acquire_stream_lock(transaction: &mut Transaction<'_, Postgres>, stream: AuditStream, statement: &'static str) -> StorageResult<()> {
    query(statement)
        .bind(AUDIT_STREAM_LOCK_PREFIX)
        .bind(stream.code())
        .bind(LOCK_HASH_SEED)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

fn payload_error(error: impl std::fmt::Display) -> StorageError {
    StorageError::Database(error.to_string())
}
