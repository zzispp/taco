use audit_contract::{ActorSnapshot, AuditOutboxEvent, AuditOutboxRecord, AuditStatus, BusinessType, OperationAuditEvent, OperatorType};
use observability::{
    application::{ObservabilityError, SystemLogRepository},
    domain::{NewSystemLog, SystemLogLevel},
    infra::StorageSystemLogRepository,
};
use sqlx::{PgPool, query_scalar};
use time::OffsetDateTime;

use super::{TestDatabase, up};

#[tokio::test]
async fn system_log_exact_delete_commits_with_its_operation_audit() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let repository = repository(&database);
    repository.insert_batch(&[event("audited-delete")]).await.unwrap();
    let audit = operation_record("system-log-delete-success");

    repository.delete_ids_with_audit(&["audited-delete".into()], &audit).await.unwrap();

    assert!(repository.find("audited-delete").await.unwrap().is_none());
    assert_eq!(outbox_count(database.pool(), &audit.id).await, 1);
    database.drop().await;
}

#[tokio::test]
async fn system_log_exact_delete_rolls_back_when_audit_append_fails() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let repository = repository(&database);
    repository.insert_batch(&[event("rollback-delete")]).await.unwrap();
    let audit = operation_record("duplicate-system-log-delete-audit");
    append_outbox(database.pool(), &audit).await;

    let result = repository.delete_ids_with_audit(&["rollback-delete".into()], &audit).await;

    assert!(result.is_err());
    assert!(repository.find("rollback-delete").await.unwrap().is_some());
    assert_eq!(outbox_count(database.pool(), &audit.id).await, 1);
    database.drop().await;
}

#[tokio::test]
async fn system_log_exact_delete_rejects_cross_partition_duplicate_ids() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let repository = repository(&database);
    repository
        .insert_batch(&[
            event_at("duplicate-log-id", timestamp()),
            event_at("duplicate-log-id", timestamp() + time::Duration::days(1)),
        ])
        .await
        .unwrap();
    let audit = operation_record("duplicate-log-id-audit");

    let error = repository.delete_ids_with_audit(&["duplicate-log-id".into()], &audit).await.unwrap_err();

    assert!(matches!(error, ObservabilityError::NotFound));
    assert_eq!(system_log_count(database.pool(), "duplicate-log-id").await, 2);
    assert_eq!(outbox_count(database.pool(), &audit.id).await, 0);
    database.drop().await;
}

#[tokio::test]
async fn system_log_exact_delete_rolls_back_a_batch_with_a_missing_id() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let repository = repository(&database);
    repository.insert_batch(&[event("existing-log")]).await.unwrap();
    let audit = operation_record("missing-log-audit");

    let error = repository
        .delete_ids_with_audit(&["existing-log".into(), "missing-log".into()], &audit)
        .await
        .unwrap_err();

    assert!(matches!(error, ObservabilityError::NotFound));
    assert_eq!(system_log_count(database.pool(), "existing-log").await, 1);
    assert_eq!(outbox_count(database.pool(), &audit.id).await, 0);
    database.drop().await;
}

async fn append_outbox(pool: &PgPool, record: &AuditOutboxRecord) {
    let mut transaction = pool.begin().await.unwrap();
    storage::outbox::append_audit_record(&mut transaction, record).await.unwrap();
    transaction.commit().await.unwrap();
}

async fn outbox_count(pool: &PgPool, id: &str) -> i64 {
    query_scalar("SELECT COUNT(*) FROM audit_outbox WHERE outbox_id=$1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn system_log_count(pool: &PgPool, id: &str) -> i64 {
    query_scalar("SELECT COUNT(*) FROM sys_system_log WHERE id=$1")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

fn repository(database: &TestDatabase) -> StorageSystemLogRepository {
    StorageSystemLogRepository::new(storage::Database::new(database.pool().clone()))
}

fn event(id: &str) -> NewSystemLog {
    event_at(id, timestamp())
}

fn event_at(id: &str, occurred_at: OffsetDateTime) -> NewSystemLog {
    NewSystemLog {
        id: id.into(),
        occurred_at,
        level: SystemLogLevel::Info,
        target: "test::audited_delete".into(),
        message: "delete me".into(),
        fields: serde_json::json!({}),
    }
}

fn operation_record(id: &str) -> AuditOutboxRecord {
    AuditOutboxRecord {
        id: id.into(),
        occurred_at: timestamp(),
        event: AuditOutboxEvent::Operation(OperationAuditEvent {
            title_key: "audit.module.system_log".into(),
            business_type: BusinessType::Delete,
            handler: "observability::delete_system_logs".into(),
            request_method: "DELETE".into(),
            operator_type: OperatorType::Manage,
            actor: ActorSnapshot {
                user_id: Some("user-1".into()),
                username: "alice".into(),
                department_id: None,
                department_name: String::new(),
            },
            operation_url: "/api/system/system-logs/batch".into(),
            operation_ip: "127.0.0.1".into(),
            status: AuditStatus::Success,
            request_id: "request-1".into(),
            request_params: String::new(),
            response_result: String::new(),
            error_message: String::new(),
            cost_time_ms: 1,
        }),
    }
}

fn timestamp() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_768_521_600).unwrap()
}
