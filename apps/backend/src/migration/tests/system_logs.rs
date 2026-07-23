use sqlx::{AssertSqlSafe, PgPool, query, query_as, query_scalar};

use super::{TestDatabase, managed_table_exists, migrate_through, rollback_from, up};

const SYSTEM_LOG_MIGRATION_VERSION: i64 = 20260716000001;
const SYSTEM_LOG_RETENTION_MIGRATION_VERSION: i64 = 20260720000003;
const REMOVED_INDEX: &str = "idx_sys_system_log_cursor";
const INDEXES: &[&str] = &[
    "idx_sys_system_log_id",
    "idx_sys_system_log_ingested_seq",
    "idx_sys_system_log_level_cursor",
    "idx_sys_system_log_search_content_trgm",
    "idx_sys_system_log_search_document",
    "idx_sys_system_log_search_ngrams",
    "idx_sys_system_log_target_cursor",
];

#[tokio::test]
async fn system_log_migration_creates_partitioned_searchable_schema_and_seeds() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();

    assert!(managed_table_exists(database.pool(), "sys_system_log").await);
    assert_parent_partitioning(database.pool()).await;
    assert_indexes(database.pool()).await;
    assert_index_absent(database.pool(), REMOVED_INDEX).await;
    assert_partition_function_and_generated_search(database.pool()).await;
    assert_snapshot_and_ngram_columns(database.pool()).await;
    assert_constraints(database.pool()).await;
    assert_runtime_config(database.pool()).await;
    assert_menus_and_permissions(database.pool()).await;
    assert_cleanup_job(database.pool()).await;

    database.drop().await;
}

#[tokio::test]
async fn system_log_migration_down_removes_owned_schema_and_seeds() {
    let database = TestDatabase::create().await;
    migrate_through(database.pool(), SYSTEM_LOG_MIGRATION_VERSION).await;

    rollback_from(database.pool(), SYSTEM_LOG_MIGRATION_VERSION).await;

    assert!(!managed_table_exists(database.pool(), "sys_system_log").await);
    assert_eq!(
        count(
            database.pool(),
            "SELECT COUNT(*) FROM sys_config WHERE config_key='sys.observability.tracingConfig'"
        )
        .await,
        0
    );
    assert_eq!(
        count(database.pool(), "SELECT COUNT(*) FROM sys_menu WHERE menu_id IN ('114','1140','1141','1142')").await,
        0
    );
    assert_eq!(
        count(database.pool(), "SELECT COUNT(*) FROM sys_job WHERE job_id='system-log-cleanup'").await,
        0
    );
    let extension_exists: bool = query_scalar("SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname='pg_trgm')")
        .fetch_one(database.pool())
        .await
        .unwrap();
    assert!(extension_exists);

    database.drop().await;
}

#[tokio::test]
async fn system_log_retention_migration_restores_the_removed_index_on_rollback() {
    let database = TestDatabase::create().await;
    migrate_through(database.pool(), SYSTEM_LOG_RETENTION_MIGRATION_VERSION).await;
    assert_index_absent(database.pool(), REMOVED_INDEX).await;
    assert!(retention_drop_function_exists(database.pool()).await);

    rollback_from(database.pool(), SYSTEM_LOG_RETENTION_MIGRATION_VERSION).await;

    assert_index_present(database.pool(), REMOVED_INDEX).await;
    assert!(!retention_drop_function_exists(database.pool()).await);
    database.drop().await;
}

async fn assert_parent_partitioning(pool: &PgPool) {
    let strategy: String = query_scalar("SELECT partstrat::text FROM pg_partitioned_table WHERE partrelid='sys_system_log'::regclass")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(strategy, "r");
}

async fn assert_indexes(pool: &PgPool) {
    for index in INDEXES {
        let exists: bool = query_scalar("SELECT EXISTS(SELECT 1 FROM pg_indexes WHERE schemaname='public' AND tablename='sys_system_log' AND indexname=$1)")
            .bind(*index)
            .fetch_one(pool)
            .await
            .unwrap();
        assert!(exists, "missing system log index {index}");
    }
}

async fn assert_index_absent(pool: &PgPool, index: &str) {
    let exists: bool = query_scalar("SELECT EXISTS(SELECT 1 FROM pg_indexes WHERE schemaname='public' AND tablename='sys_system_log' AND indexname=$1)")
        .bind(index)
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(!exists, "unexpected redundant system log index {index}");
}

async fn assert_index_present(pool: &PgPool, index: &str) {
    let exists: bool = query_scalar("SELECT EXISTS(SELECT 1 FROM pg_indexes WHERE schemaname='public' AND tablename='sys_system_log' AND indexname=$1)")
        .bind(index)
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(exists, "missing system log index {index}");
}

async fn retention_drop_function_exists(pool: &PgPool) -> bool {
    query_scalar("SELECT to_regprocedure('drop_expired_system_log_partition(text,timestamptz)') IS NOT NULL")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn assert_partition_function_and_generated_search(pool: &PgPool) {
    query("SELECT ensure_system_log_partition(TIMESTAMPTZ '2026-07-16 12:00:00+00')")
        .execute(pool)
        .await
        .unwrap();
    let partition: bool = query_scalar("SELECT to_regclass('public.sys_system_log_20260716') IS NOT NULL")
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(partition);
    query("INSERT INTO sys_system_log (id,occurred_at,level,target,message,fields) VALUES ('log-1',TIMESTAMPTZ '2026-07-16 12:00:00+00','info','http','request completed','{\"request_id\":\"req-1\"}'::jsonb)")
        .execute(pool)
        .await
        .unwrap();
    let generated: (String, bool) =
        query_as("SELECT searchable_content,search_document @@ websearch_to_tsquery('simple','req-1') FROM sys_system_log WHERE id='log-1'")
            .fetch_one(pool)
            .await
            .unwrap();
    assert!(generated.0.contains("req-1"));
    assert!(generated.1);
}

async fn assert_snapshot_and_ngram_columns(pool: &PgPool) {
    let generated: (i64, Vec<String>) = query_as("SELECT ingested_seq,search_ngrams FROM sys_system_log WHERE id='log-1'")
        .fetch_one(pool)
        .await
        .unwrap();

    assert!(generated.0 > 0);
    assert!(generated.1.contains(&"re".into()));
    assert!(generated.1.contains(&"r".into()));
}

async fn assert_constraints(pool: &PgPool) {
    let invalid_level = query(
        "INSERT INTO sys_system_log (id,occurred_at,level,target,message) VALUES ('bad-level',TIMESTAMPTZ '2026-07-16 12:01:00+00','fatal','test','bad')",
    )
    .execute(pool)
    .await;
    let invalid_fields = query("INSERT INTO sys_system_log (id,occurred_at,level,target,message,fields) VALUES ('bad-fields',TIMESTAMPTZ '2026-07-16 12:01:00+00','info','test','bad','[]'::jsonb)")
        .execute(pool)
        .await;
    assert!(invalid_level.is_err());
    assert!(invalid_fields.is_err());
}

async fn assert_runtime_config(pool: &PgPool) {
    let config: (String, bool, String) = query_as("SELECT config_value,public_read,remark FROM sys_config WHERE config_key='sys.observability.tracingConfig'")
        .fetch_one(pool)
        .await
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(&config.0).unwrap();
    assert_eq!(value["log_level"], "info");
    assert_eq!(
        value["http"],
        serde_json::json!({
            "access_enabled": true,
            "capture_request_body": false,
            "capture_response_body": false,
            "capture_query_parameters": true,
            "capture_request_headers": false,
            "max_body_capture_bytes": 16_384
        })
    );
    assert_eq!(
        value["slow_operation_ms"],
        serde_json::json!({"postgres": 500, "redis": 100, "outbound_http": 1_000})
    );
    assert!(!config.1);
    assert!(config.2.contains("log_level"));
}

async fn assert_menus_and_permissions(pool: &PgPool) {
    let menus: Vec<(String, String)> = query_as("SELECT menu_id,perms FROM sys_menu WHERE menu_id IN ('114','1140','1141','1142') ORDER BY menu_id")
        .fetch_all(pool)
        .await
        .unwrap();
    assert_eq!(
        menus,
        vec![
            ("114".into(), "system:systemlog:list".into()),
            ("1140".into(), "system:systemlog:query".into()),
            ("1141".into(), "system:systemlog:remove".into()),
            ("1142".into(), "system:systemlog:export".into()),
        ]
    );
}

async fn assert_cleanup_job(pool: &PgPool) {
    let job: (String, String, String, String, String, String, String) =
        query_as("SELECT task_key,task_params::text,cron_expression,misfire_policy,concurrent,status,remark FROM sys_job WHERE job_id='system-log-cleanup'")
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(job.0, "observability.cleanupSystemLogs");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&job.1).unwrap(),
        serde_json::json!({"retention_days": 7, "batch_size": 1000})
    );
    assert_eq!((job.2, job.3, job.4, job.5), ("0 0 19 * * *".into(), "2".into(), "1".into(), "0".into()));
    assert!(job.6.contains("batch_size 仅控制截止日"));
}

async fn count(pool: &PgPool, sql: &str) -> i64 {
    query_scalar::<_, i64>(AssertSqlSafe(sql.to_owned())).fetch_one(pool).await.unwrap()
}
