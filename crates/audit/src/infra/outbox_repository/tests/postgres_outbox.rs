use std::time::Duration;

use audit_contract::{AuditStream, BusinessType, LoginEventType};
use sqlx::{PgPool, query, query_scalar};
use storage::{Database, outbox::lock_audit_stream_shared};
use time::OffsetDateTime;
use tokio::time::{sleep, timeout};

use crate::{
    application::{AuditError, AuditRepository},
    domain::AuditLocation,
    infra::StorageAuditRepository,
};

use super::super::{ClaimedAuditEvent, StorageAuditOutboxRepository};
use super::{
    fixtures::{operation_record, security_record},
    postgres::TestDatabase,
};

const CLAIM_LIMIT: i64 = 1;
const LEASE_SECONDS: i64 = 60;
const RETRY_DELAY_SECONDS: i64 = 3_600;
const PAST_SECONDS: i64 = 1;
const LOCK_WAIT_TIMEOUT: Duration = Duration::from_secs(2);
const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(10);
const FIRST_WORKER: &str = "worker-first";
const SECOND_WORKER: &str = "worker-second";
const THIRD_WORKER: &str = "worker-third";
const PROJECTION_FAILURE: &str = "projection_failed";
const INVALID_PAYLOAD: &str = "invalid_payload";
const CLAIM_RETRY_ID: &str = "outbox-claim-retry";
const INVALID_PAYLOAD_ID: &str = "outbox-invalid-payload";
const SECURITY_EVENT_ID: &str = "outbox-security-unknown";
const OLD_OPERATION_ID: &str = "outbox-old-operation";
const CLEAR_OPERATION_ID: &str = "outbox-clear-operation";
const ROLLBACK_OPERATION_ID: &str = "outbox-rollback-operation";

#[tokio::test]
async fn transactional_outbox_claims_retries_projects_and_clears_without_resurrection() {
    let database = TestDatabase::create().await;
    let outbox = StorageAuditOutboxRepository::new(Database::new(database.pool().clone()));
    let audit_logs = StorageAuditRepository::new(Database::new(database.pool().clone()));

    assert_claim_lease_retry_and_idempotence(&outbox, database.pool()).await;
    assert_invalid_payload_is_released_for_retry(&outbox, database.pool()).await;
    assert_security_projection_preserves_unknown_location(&outbox, database.pool()).await;
    assert_clear_waits_for_projection_lock_and_prevents_resurrection(&outbox, &audit_logs, database.pool()).await;
    assert_outbox_failure_rolls_back_log_deletion(&outbox, &audit_logs, database.pool()).await;

    database.close().await;
}

async fn assert_claim_lease_retry_and_idempotence(outbox: &StorageAuditOutboxRepository, pool: &PgPool) {
    let mut record = operation_record(CLAIM_RETRY_ID, BusinessType::Insert);
    // Availability uses PostgreSQL time so an application clock ahead of the database cannot delay delivery.
    record.occurred_at = future(LEASE_SECONDS);
    outbox.record(record).await.unwrap();
    assert_eq!(outbox_event_type(pool, CLAIM_RETRY_ID).await, "operation");

    let first = claim_one(outbox, FIRST_WORKER).await;
    assert_eq!(first.id, CLAIM_RETRY_ID);
    assert_delivery(pool, CLAIM_RETRY_ID, DeliveryExpectation::new(1, Some(FIRST_WORKER), None)).await;
    assert!(outbox.claim(claim_options(SECOND_WORKER)).await.unwrap().is_empty());

    expire_lease(pool, CLAIM_RETRY_ID).await;
    let reclaimed = claim_one(outbox, SECOND_WORKER).await;
    assert_eq!(reclaimed.id, CLAIM_RETRY_ID);
    assert_delivery(pool, CLAIM_RETRY_ID, DeliveryExpectation::new(2, Some(SECOND_WORKER), None)).await;

    outbox.retry(&reclaimed, future(RETRY_DELAY_SECONDS), PROJECTION_FAILURE).await.unwrap();
    assert_delivery(pool, CLAIM_RETRY_ID, DeliveryExpectation::new(2, None, Some(PROJECTION_FAILURE))).await;
    assert!(outbox.claim(claim_options(THIRD_WORKER)).await.unwrap().is_empty());

    make_available(pool, CLAIM_RETRY_ID).await;
    let retried = claim_one(outbox, THIRD_WORKER).await;
    assert_delivery(pool, CLAIM_RETRY_ID, DeliveryExpectation::new(3, Some(THIRD_WORKER), Some(PROJECTION_FAILURE))).await;
    assert!(outbox.complete(&retried, AuditLocation::Unknown).await.unwrap());
    assert_eq!(operation_count(pool, CLAIM_RETRY_ID).await, 1);
    assert!(processed(pool, CLAIM_RETRY_ID).await);
    assert!(!outbox.complete(&retried, AuditLocation::Unknown).await.unwrap());
    assert_eq!(operation_count(pool, CLAIM_RETRY_ID).await, 1);
}

async fn assert_invalid_payload_is_released_for_retry(outbox: &StorageAuditOutboxRepository, pool: &PgPool) {
    query(
        "INSERT INTO audit_outbox (outbox_id,stream,event_type,payload_version,payload,occurred_at,available_at) VALUES ($1,'operation','operation',1,'{}'::jsonb,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)",
    )
    .bind(INVALID_PAYLOAD_ID)
    .execute(pool)
    .await
    .unwrap();

    assert!(outbox.claim(claim_options(FIRST_WORKER)).await.unwrap().is_empty());
    assert_delivery(pool, INVALID_PAYLOAD_ID, DeliveryExpectation::new(1, None, Some(INVALID_PAYLOAD))).await;
}

async fn assert_security_projection_preserves_unknown_location(outbox: &StorageAuditOutboxRepository, pool: &PgPool) {
    outbox.record(security_record(SECURITY_EVENT_ID, LoginEventType::LoginFailure)).await.unwrap();
    assert_eq!(outbox_event_type(pool, SECURITY_EVENT_ID).await, LoginEventType::LoginFailure.code());

    let claimed = claim_one(outbox, FIRST_WORKER).await;
    assert_eq!(claimed.stream(), AuditStream::Security);
    assert!(outbox.complete(&claimed, AuditLocation::Unknown).await.unwrap());

    let location_kind: String = query_scalar("SELECT login_location_kind FROM sys_logininfor WHERE info_id=$1")
        .bind(SECURITY_EVENT_ID)
        .fetch_one(pool)
        .await
        .unwrap();
    let location: String = query_scalar("SELECT login_location FROM sys_logininfor WHERE info_id=$1")
        .bind(SECURITY_EVENT_ID)
        .fetch_one(pool)
        .await
        .unwrap();
    let event_type: String = query_scalar("SELECT event_type FROM sys_logininfor WHERE info_id=$1")
        .bind(SECURITY_EVENT_ID)
        .fetch_one(pool)
        .await
        .unwrap();

    assert_eq!(location_kind, "unknown");
    assert_eq!(location, "");
    assert_eq!(event_type, LoginEventType::LoginFailure.code());
}

async fn assert_clear_waits_for_projection_lock_and_prevents_resurrection(
    outbox: &StorageAuditOutboxRepository,
    audit_logs: &StorageAuditRepository,
    pool: &PgPool,
) {
    outbox.record(operation_record(OLD_OPERATION_ID, BusinessType::Insert)).await.unwrap();
    let old_claim = claim_one(outbox, FIRST_WORKER).await;
    let mut lock = pool.begin().await.unwrap();
    lock_audit_stream_shared(&mut lock, AuditStream::Operation).await.unwrap();

    let clear_record = operation_record(CLEAR_OPERATION_ID, BusinessType::Clean);
    let clear_task = tokio::spawn({
        let audit_logs = audit_logs.clone();
        async move { audit_logs.clear_operations_with_audit(&clear_record).await }
    });
    let outbox_count_before_clear = outbox_count(pool, AuditStream::Operation).await;
    wait_for_blocked_advisory_lock(pool).await;
    assert_eq!(outbox_count(pool, AuditStream::Operation).await, outbox_count_before_clear);

    lock.commit().await.unwrap();
    clear_task.await.unwrap().unwrap();
    assert!(!outbox.complete(&old_claim, AuditLocation::Unknown).await.unwrap());

    let clear_claim = claim_one(outbox, SECOND_WORKER).await;
    assert_eq!(clear_claim.id, CLEAR_OPERATION_ID);
    assert!(outbox.complete(&clear_claim, AuditLocation::Unknown).await.unwrap());
    assert_eq!(operation_ids(pool).await, vec![CLEAR_OPERATION_ID.to_owned()]);
}

async fn assert_outbox_failure_rolls_back_log_deletion(outbox: &StorageAuditOutboxRepository, audit_logs: &StorageAuditRepository, pool: &PgPool) {
    let record = operation_record(ROLLBACK_OPERATION_ID, BusinessType::Insert);
    outbox.record(record.clone()).await.unwrap();
    let claimed = claim_one(outbox, FIRST_WORKER).await;
    assert!(outbox.complete(&claimed, AuditLocation::Unknown).await.unwrap());

    let result = audit_logs.delete_operations_with_audit(&[ROLLBACK_OPERATION_ID.into()], &record).await;
    assert!(matches!(result, Err(AuditError::Infrastructure(_))));
    assert_eq!(operation_count(pool, ROLLBACK_OPERATION_ID).await, 1);
}

async fn claim_one(outbox: &StorageAuditOutboxRepository, worker: &str) -> ClaimedAuditEvent {
    let mut claimed = outbox.claim(claim_options(worker)).await.unwrap();
    assert_eq!(claimed.len(), 1);
    claimed.pop().unwrap()
}

fn claim_options(worker: &str) -> super::super::ClaimOptions<'_> {
    super::super::ClaimOptions {
        lease_token: worker,
        limit: CLAIM_LIMIT,
        lease_until: future(LEASE_SECONDS),
        retry_at: future(RETRY_DELAY_SECONDS),
    }
}

struct DeliveryExpectation<'a> {
    attempts: i32,
    lease: Option<&'a str>,
    error_code: Option<&'a str>,
}

impl<'a> DeliveryExpectation<'a> {
    fn new(attempts: i32, lease: Option<&'a str>, error_code: Option<&'a str>) -> Self {
        Self { attempts, lease, error_code }
    }
}

async fn assert_delivery(pool: &PgPool, id: &str, expected: DeliveryExpectation<'_>) {
    let actual_attempts: i32 = query_scalar("SELECT attempt_count FROM audit_outbox WHERE outbox_id=$1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap();
    let actual_lease: Option<String> = query_scalar("SELECT lease_token FROM audit_outbox WHERE outbox_id=$1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap();
    let actual_error: Option<String> = query_scalar("SELECT last_error_code FROM audit_outbox WHERE outbox_id=$1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap();

    assert_eq!(actual_attempts, expected.attempts);
    assert_eq!(actual_lease.as_deref(), expected.lease);
    assert_eq!(actual_error.as_deref(), expected.error_code);
}

async fn expire_lease(pool: &PgPool, id: &str) {
    query("UPDATE audit_outbox SET lease_until=$2 WHERE outbox_id=$1")
        .bind(id)
        .bind(past())
        .execute(pool)
        .await
        .unwrap();
}

async fn make_available(pool: &PgPool, id: &str) {
    query("UPDATE audit_outbox SET available_at=$2 WHERE outbox_id=$1")
        .bind(id)
        .bind(past())
        .execute(pool)
        .await
        .unwrap();
}

async fn processed(pool: &PgPool, id: &str) -> bool {
    query_scalar("SELECT processed_at IS NOT NULL FROM audit_outbox WHERE outbox_id=$1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn operation_count(pool: &PgPool, id: &str) -> i64 {
    query_scalar("SELECT COUNT(*) FROM sys_oper_log WHERE oper_id=$1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn operation_ids(pool: &PgPool) -> Vec<String> {
    query_scalar("SELECT oper_id FROM sys_oper_log ORDER BY oper_id").fetch_all(pool).await.unwrap()
}

async fn outbox_count(pool: &PgPool, stream: AuditStream) -> i64 {
    query_scalar("SELECT COUNT(*) FROM audit_outbox WHERE stream=$1")
        .bind(stream.code())
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn outbox_event_type(pool: &PgPool, id: &str) -> String {
    query_scalar("SELECT event_type FROM audit_outbox WHERE outbox_id=$1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn wait_for_blocked_advisory_lock(pool: &PgPool) {
    timeout(LOCK_WAIT_TIMEOUT, async {
        loop {
            let blocked: bool = query_scalar("SELECT EXISTS (SELECT 1 FROM pg_locks WHERE locktype='advisory' AND NOT granted)")
                .fetch_one(pool)
                .await
                .unwrap();
            if blocked {
                return;
            }
            sleep(LOCK_POLL_INTERVAL).await;
        }
    })
    .await
    .expect("clear operation must wait for the stream advisory lock");
}

fn future(seconds: i64) -> OffsetDateTime {
    OffsetDateTime::now_utc().checked_add(time::Duration::seconds(seconds)).unwrap()
}

fn past() -> OffsetDateTime {
    OffsetDateTime::now_utc().checked_sub(time::Duration::seconds(PAST_SECONDS)).unwrap()
}
