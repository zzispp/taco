use async_trait::async_trait;
use audit_contract::{
    AUDIT_OUTBOX_PAYLOAD_VERSION, AuditOutboxEvent, AuditOutboxRecord, AuditOutboxRecorder, AuditOutboxResult, AuditStream, SecurityAuditRecorder,
};
use sqlx::{FromRow, query, query_as};
use storage::{
    Database,
    outbox::{append_audit_record, lock_audit_stream_shared},
};
use time::OffsetDateTime;

use crate::{
    application::{AuditError, AuditResult},
    domain::AuditLocation,
};

use super::{
    limits::{LOCATION_MAX_CHARS, truncate},
    mapping, outbox_projection,
};

const INVALID_PAYLOAD_ERROR_CODE: &str = "invalid_payload";

#[derive(Clone)]
pub struct StorageAuditOutboxRepository {
    database: Database,
}

#[derive(Clone, Debug)]
pub(crate) struct ClaimedAuditEvent {
    pub id: String,
    pub lease_token: String,
    pub occurred_at: OffsetDateTime,
    pub event: AuditOutboxEvent,
}

pub(crate) struct ClaimOptions<'a> {
    pub lease_token: &'a str,
    pub limit: i64,
    pub lease_until: OffsetDateTime,
    pub retry_at: OffsetDateTime,
}

impl ClaimedAuditEvent {
    pub(crate) fn stream(&self) -> AuditStream {
        self.event.stream()
    }

    pub(crate) fn ip_address(&self) -> &str {
        match &self.event {
            AuditOutboxEvent::Operation(event) => &event.operation_ip,
            AuditOutboxEvent::Security(event) => &event.ip_address,
        }
    }
}

#[derive(FromRow)]
struct ClaimedRow {
    outbox_id: String,
    stream: String,
    event_type: String,
    payload_version: i16,
    payload: serde_json::Value,
    occurred_at: OffsetDateTime,
    lease_token: String,
}

struct EncodedEvent<'a> {
    stream: &'a str,
    event_type: &'a str,
    version: i16,
    payload: serde_json::Value,
}

impl StorageAuditOutboxRepository {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn record(&self, record: AuditOutboxRecord) -> AuditOutboxResult<()> {
        let mut transaction = self.database.pool().begin().await.map_err(infrastructure)?;
        append_audit_record(&mut transaction, &record).await.map_err(infrastructure)?;
        transaction.commit().await.map_err(infrastructure)
    }

    pub(crate) async fn claim(&self, options: ClaimOptions<'_>) -> AuditResult<Vec<ClaimedAuditEvent>> {
        let mut transaction = self.database.pool().begin().await.map_err(mapping::sqlx_error)?;
        let rows = query_as::<_, ClaimedRow>(
            r#"
            WITH candidates AS (
                SELECT outbox_id
                FROM audit_outbox
                WHERE processed_at IS NULL
                  AND available_at <= CURRENT_TIMESTAMP
                  AND (lease_until IS NULL OR lease_until <= CURRENT_TIMESTAMP)
                ORDER BY available_at ASC, occurred_at ASC, outbox_id ASC
                FOR UPDATE SKIP LOCKED
                LIMIT $1
            )
            UPDATE audit_outbox AS target
            SET lease_token = $2,
                lease_until = $3,
                attempt_count = target.attempt_count + 1
            FROM candidates
            WHERE target.outbox_id = candidates.outbox_id
            RETURNING target.outbox_id, target.stream, target.event_type, target.payload_version, target.payload, target.occurred_at, target.lease_token
            "#,
        )
        .bind(options.limit)
        .bind(options.lease_token)
        .bind(options.lease_until)
        .fetch_all(&mut *transaction)
        .await
        .map_err(mapping::sqlx_error)?;
        transaction.commit().await.map_err(mapping::sqlx_error)?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            match claimed_event(row) {
                Ok(event) => events.push(event),
                Err((id, token)) => self.release_invalid(&id, &token, options.retry_at).await?,
            }
        }
        Ok(events)
    }

    pub(crate) async fn complete(&self, claimed: &ClaimedAuditEvent, location: AuditLocation) -> AuditResult<bool> {
        let mut transaction = self.database.pool().begin().await.map_err(mapping::sqlx_error)?;
        lock_audit_stream_shared(&mut transaction, claimed.stream()).await.map_err(storage_error)?;
        let projected = outbox_projection::project_and_mark(&mut transaction, claimed, location).await?;
        if !projected {
            return Ok(false);
        }
        transaction.commit().await.map_err(mapping::sqlx_error)?;
        Ok(true)
    }

    pub(crate) async fn retry(&self, claimed: &ClaimedAuditEvent, available_at: OffsetDateTime, error_code: &'static str) -> AuditResult<()> {
        query(
            "UPDATE audit_outbox SET lease_token=NULL, lease_until=NULL, available_at=$3, last_error_code=$4 WHERE outbox_id=$1 AND lease_token=$2 AND processed_at IS NULL",
        )
        .bind(&claimed.id)
        .bind(&claimed.lease_token)
        .bind(available_at)
        .bind(error_code)
        .execute(self.database.pool())
        .await
        .map_err(mapping::sqlx_error)?;
        Ok(())
    }

    pub(crate) async fn cleanup(&self, older_than: OffsetDateTime, limit: i64) -> AuditResult<u64> {
        let result = query(
            r#"
            WITH expired AS (
                SELECT outbox_id
                FROM audit_outbox
                WHERE processed_at IS NOT NULL AND processed_at < $1
                ORDER BY processed_at ASC, outbox_id ASC
                FOR UPDATE SKIP LOCKED
                LIMIT $2
            )
            DELETE FROM audit_outbox AS target
            USING expired
            WHERE target.outbox_id = expired.outbox_id
            "#,
        )
        .bind(older_than)
        .bind(limit)
        .execute(self.database.pool())
        .await
        .map_err(mapping::sqlx_error)?;
        Ok(result.rows_affected())
    }

    async fn release_invalid(&self, id: &str, lease_token: &str, available_at: OffsetDateTime) -> AuditResult<()> {
        query(
            "UPDATE audit_outbox SET lease_token=NULL, lease_until=NULL, available_at=$3, last_error_code=$4 WHERE outbox_id=$1 AND lease_token=$2 AND processed_at IS NULL",
        )
        .bind(id)
        .bind(lease_token)
        .bind(available_at)
        .bind(INVALID_PAYLOAD_ERROR_CODE)
        .execute(self.database.pool())
        .await
        .map_err(mapping::sqlx_error)?;
        Ok(())
    }
}

#[async_trait]
impl AuditOutboxRecorder for StorageAuditOutboxRepository {
    async fn record(&self, record: AuditOutboxRecord) -> AuditOutboxResult<()> {
        StorageAuditOutboxRepository::record(self, record).await
    }
}

#[async_trait]
impl SecurityAuditRecorder for StorageAuditOutboxRepository {
    async fn record(&self, record: AuditOutboxRecord) -> AuditOutboxResult<()> {
        StorageAuditOutboxRepository::record(self, record).await
    }
}

fn claimed_event(row: ClaimedRow) -> Result<ClaimedAuditEvent, (String, String)> {
    let id = row.outbox_id;
    let token = row.lease_token;
    let event = decode_event(EncodedEvent {
        stream: &row.stream,
        event_type: &row.event_type,
        version: row.payload_version,
        payload: row.payload,
    })
    .map_err(|_| (id.clone(), token.clone()))?;
    Ok(ClaimedAuditEvent {
        id,
        lease_token: token,
        occurred_at: row.occurred_at,
        event,
    })
}

fn decode_event(input: EncodedEvent<'_>) -> AuditResult<AuditOutboxEvent> {
    if input.version != AUDIT_OUTBOX_PAYLOAD_VERSION {
        return Err(AuditError::Infrastructure(format!(
            "unsupported audit outbox payload version: {}",
            input.version
        )));
    }
    let event = serde_json::from_value::<AuditOutboxEvent>(input.payload).map_err(|error| AuditError::Infrastructure(error.to_string()))?;
    if event.stream().code() != input.stream {
        return Err(AuditError::Infrastructure("audit outbox stream does not match payload kind".into()));
    }
    if event.event_type() != input.event_type {
        return Err(AuditError::Infrastructure("audit outbox event type does not match payload".into()));
    }
    Ok(event)
}

fn infrastructure(error: impl std::fmt::Display) -> audit_contract::AuditOutboxError {
    audit_contract::AuditOutboxError::Infrastructure(error.to_string())
}

fn storage_error(error: storage::StorageError) -> AuditError {
    AuditError::Infrastructure(error.to_string())
}

fn truncate_location(value: String) -> AuditLocation {
    AuditLocation::Resolved(truncate(&value, LOCATION_MAX_CHARS))
}

pub(crate) fn audit_location(location: client_info::IpLocation) -> AuditLocation {
    match location {
        client_info::IpLocation::Resolved(value) => truncate_location(value),
        client_info::IpLocation::Internal => AuditLocation::Internal,
        client_info::IpLocation::Unknown => AuditLocation::Unknown,
    }
}

#[cfg(test)]
mod tests;
