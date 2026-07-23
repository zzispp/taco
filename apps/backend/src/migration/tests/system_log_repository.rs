use kernel::pagination::CursorDirection;
use observability::{
    application::{SystemLogBoundary, SystemLogCursorQuery, SystemLogRepository, SystemLogRetentionStore},
    domain::{NewSystemLog, SystemLogFilter, SystemLogLevel},
    infra::StorageSystemLogRepository,
};
use sqlx::{PgPool, query, query_scalar};
use time::{Duration, OffsetDateTime};

use super::{TestDatabase, up};

#[tokio::test]
async fn system_log_repository_reads_a_nonempty_first_page() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let repository = repository(&database);
    let occurred_at = time("2026-07-16T12:00:00Z");
    repository
        .insert_batch(&[event("system-log-page", occurred_at, "test::system_logs", "first page")])
        .await
        .unwrap();

    let page = repository.page(SystemLogFilter::default(), page_query(20)).await.unwrap();

    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].id, "system-log-page");
    database.drop().await;
}

#[tokio::test]
async fn system_log_retention_drops_full_partitions_and_batches_only_the_boundary() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let repository = repository(&database);
    insert_retention_fixture(database.pool(), &repository).await;

    let report = repository.cleanup_before(time("2026-07-16T12:00:00Z"), 1).await.unwrap();

    assert_eq!((report.deleted, report.batches), (5, 5));
    assert_partition_lifecycle(database.pool()).await;
    database.drop().await;
}

#[tokio::test]
async fn system_log_retention_waits_for_locked_expired_partition() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let repository = repository(&database);
    repository
        .insert_batch(&[event("locked-expired-log", time("2026-07-14T12:00:00Z"), "test::retention", "expired")])
        .await
        .unwrap();
    let mut lock = database.pool().begin().await.unwrap();
    query("SELECT id FROM sys_system_log WHERE id='locked-expired-log' FOR UPDATE")
        .execute(&mut *lock)
        .await
        .unwrap();
    let cleanup = tokio::spawn({
        let repository = repository.clone();
        async move { repository.cleanup_before(time("2026-07-16T12:00:00Z"), 1).await.unwrap() }
    });

    tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    assert!(!cleanup.is_finished());
    lock.commit().await.unwrap();
    let report = cleanup.await.unwrap();
    assert_eq!((report.deleted, report.batches), (1, 1));
    assert!(!partition_exists(database.pool(), "sys_system_log_20260714").await);
    database.drop().await;
}

#[tokio::test]
async fn system_log_filtered_cleanup_uses_the_requested_batch_limit() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let repository = repository(&database);
    let occurred_at = time("2026-07-16T12:00:00Z");
    let events = ["batch-one", "batch-two", "batch-three"].map(|id| event(id, occurred_at, "test::cleanup", id));
    repository.insert_batch(&events).await.unwrap();
    let filter = SystemLogFilter {
        begin_time: Some(occurred_at - Duration::seconds(1)),
        end_time: Some(occurred_at + Duration::seconds(1)),
        ..SystemLogFilter::default()
    };

    assert_eq!(repository.delete_filtered_batch(filter.clone(), 2).await.unwrap(), 2);
    assert_eq!(repository.count(filter.clone()).await.unwrap(), 1);
    assert_eq!(repository.delete_filtered_batch(filter.clone(), 2).await.unwrap(), 1);
    assert_eq!(repository.count(filter).await.unwrap(), 0);
    assert!(partition_exists(database.pool(), "sys_system_log_20260716").await);
    database.drop().await;
}

async fn insert_retention_fixture(pool: &PgPool, repository: &StorageSystemLogRepository) {
    query("SELECT ensure_system_log_partition(TIMESTAMPTZ '2026-07-13 12:00:00+00')")
        .execute(pool)
        .await
        .unwrap();
    let events = [
        event("expired-one", time("2026-07-14T01:00:00Z"), "test::retention", "expired"),
        event("expired-two", time("2026-07-14T23:00:00Z"), "test::retention", "expired"),
        event("expired-three", time("2026-07-15T12:00:00Z"), "test::retention", "expired"),
        event("boundary-one", time("2026-07-16T01:00:00Z"), "test::retention", "expired"),
        event("boundary-two", time("2026-07-16T11:59:59Z"), "test::retention", "expired"),
        event("at-cutoff", time("2026-07-16T12:00:00Z"), "test::retention", "kept"),
        event("after-cutoff", time("2026-07-16T20:00:00Z"), "test::retention", "kept"),
        event("future", time("2026-07-17T12:00:00Z"), "test::retention", "kept"),
    ];
    repository.insert_batch(&events).await.unwrap();
}

async fn assert_partition_lifecycle(pool: &PgPool) {
    for suffix in ["20260713", "20260714", "20260715"] {
        assert!(!partition_exists(pool, &format!("sys_system_log_{suffix}")).await);
    }
    for suffix in ["20260716", "20260717"] {
        assert!(partition_exists(pool, &format!("sys_system_log_{suffix}")).await);
    }
    let ids: Vec<String> = query_scalar("SELECT id FROM sys_system_log ORDER BY id").fetch_all(pool).await.unwrap();
    assert_eq!(ids, ["after-cutoff", "at-cutoff", "future"]);
}

async fn partition_exists(pool: &PgPool, name: &str) -> bool {
    query_scalar("SELECT to_regclass('public.' || $1) IS NOT NULL")
        .bind(name)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[tokio::test]
async fn system_log_cursor_snapshot_excludes_late_ingestion_without_losing_it_after_refresh() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let repository = repository(&database);
    repository
        .insert_batch(&[
            event("first", time("2026-07-16T12:00:00Z"), "test::cursor", "first"),
            event("boundary", time("2026-07-16T11:00:00Z"), "test::cursor", "boundary"),
            event("older", time("2026-07-16T10:00:00Z"), "test::cursor", "older"),
        ])
        .await
        .unwrap();
    let first = repository.page(SystemLogFilter::default(), page_query(2)).await.unwrap();
    let snapshot = first.snapshot.clone().unwrap();
    let boundary = SystemLogBoundary::from_summary(first.items.last().unwrap());

    repository
        .insert_batch(&[event("late", time("2026-07-16T11:30:00Z"), "test::cursor", "late")])
        .await
        .unwrap();
    let next = repository
        .page(
            SystemLogFilter::default(),
            SystemLogCursorQuery {
                limit: 2,
                direction: CursorDirection::Next,
                boundary: Some(boundary),
                snapshot: Some(snapshot),
            },
        )
        .await
        .unwrap();
    let fresh = repository.page(SystemLogFilter::default(), page_query(3)).await.unwrap();

    assert_eq!(next.items.into_iter().map(|value| value.id).collect::<Vec<_>>(), ["older"]);
    assert_eq!(fresh.items.into_iter().map(|value| value.id).collect::<Vec<_>>(), ["first", "late", "boundary"]);
    database.drop().await;
}

#[tokio::test]
async fn system_log_query_supports_target_prefix_and_short_cjk_substring() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    let repository = repository(&database);
    let occurred_at = time("2026-07-16T12:00:00Z");
    repository
        .insert_batch(&[
            event("api", occurred_at, "user::api::handlers", "中文请求完成"),
            event("repo", occurred_at - Duration::seconds(1), "user::infra", "English"),
            event("other", occurred_at - Duration::seconds(2), "audit::api", "中文审计"),
        ])
        .await
        .unwrap();
    let target_page = repository
        .page(
            SystemLogFilter {
                target: Some("user".into()),
                ..SystemLogFilter::default()
            },
            page_query(10),
        )
        .await
        .unwrap();
    let keyword_page = repository
        .page(
            SystemLogFilter {
                keyword: Some("中文".into()),
                ..SystemLogFilter::default()
            },
            page_query(10),
        )
        .await
        .unwrap();
    let ascii_keyword_page = repository
        .page(
            SystemLogFilter {
                keyword: Some("EN".into()),
                ..SystemLogFilter::default()
            },
            page_query(10),
        )
        .await
        .unwrap();

    assert_eq!(target_page.items.into_iter().map(|value| value.id).collect::<Vec<_>>(), ["api", "repo"]);
    assert_eq!(keyword_page.items.into_iter().map(|value| value.id).collect::<Vec<_>>(), ["api", "other"]);
    assert_eq!(ascii_keyword_page.items.into_iter().map(|value| value.id).collect::<Vec<_>>(), ["repo"]);
    database.drop().await;
}

fn repository(database: &TestDatabase) -> StorageSystemLogRepository {
    StorageSystemLogRepository::new(storage::Database::new(database.pool().clone()))
}

fn page_query(limit: u64) -> SystemLogCursorQuery {
    SystemLogCursorQuery {
        limit,
        direction: CursorDirection::Next,
        boundary: None,
        snapshot: None,
    }
}

fn event(id: &str, occurred_at: OffsetDateTime, target: &str, message: &str) -> NewSystemLog {
    NewSystemLog {
        id: id.into(),
        occurred_at,
        level: SystemLogLevel::Info,
        target: target.into(),
        message: message.into(),
        fields: serde_json::json!({}),
    }
}

fn time(value: &str) -> OffsetDateTime {
    OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).unwrap()
}
