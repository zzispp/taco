use serde_json::{Value, json};
use sqlx::{PgPool, query, query_as, query_scalar};

use super::{TestDatabase, down, managed_table_exists, migrate_through, up};

const MIGRATIONS_BEFORE_DETAIL: u32 = 12;
const DETAIL_MIGRATION_VERSION: i64 = 20260710000001;
const DETAIL_MENU_ID: &str = "1093";
const DETAIL_PERMISSION: &str = "system:job:log:detail";
const SYSTEM_ADMIN_ROLE_ID: &str = "admin";
const PERMISSION_CONFLICT_MENU_ID: &str = "scheduler-detail-permission-conflict";
const DETAIL_DOWN_ROLE_ID: &str = "scheduler-detail-down";
const DETAIL_COLUMN_COUNT: i64 = 3;

#[tokio::test]
async fn execution_detail_migration_preserves_history_and_enforces_constraints() {
    let database = TestDatabase::create().await;
    up(database.pool(), Some(MIGRATIONS_BEFORE_DETAIL)).await.unwrap();
    insert_terminal_execution(database.pool(), "legacy-execution").await;

    up(database.pool(), None).await.unwrap();

    assert_legacy_detail_is_null(database.pool()).await;
    assert_detail_value_constraints(database.pool()).await;
    assert_active_execution_detail_constraints(database.pool()).await;

    database.drop().await;
}

#[tokio::test]
async fn execution_detail_menu_conflict_rolls_back_the_whole_migration() {
    let database = TestDatabase::create().await;
    up(database.pool(), Some(MIGRATIONS_BEFORE_DETAIL)).await.unwrap();
    insert_detail_menu_conflict(database.pool()).await;

    assert!(up(database.pool(), None).await.is_err());
    assert_eq!(detail_column_count(database.pool()).await, 0);
    assert_eq!(detail_menu_count(database.pool()).await, 1);
    assert_eq!(detail_role_binding_count(database.pool()).await, 0);

    query("DELETE FROM sys_menu WHERE menu_id=$1")
        .bind(DETAIL_MENU_ID)
        .execute(database.pool())
        .await
        .unwrap();
    up(database.pool(), None).await.unwrap();
    assert_detail_seed(database.pool()).await;

    database.drop().await;
}

#[tokio::test]
async fn execution_detail_permission_conflict_rolls_back_the_whole_migration() {
    let database = TestDatabase::create().await;
    up(database.pool(), Some(MIGRATIONS_BEFORE_DETAIL)).await.unwrap();
    insert_detail_permission_conflict(database.pool()).await;

    assert!(up(database.pool(), None).await.is_err());
    assert_eq!(detail_column_count(database.pool()).await, 0);
    assert_eq!(detail_menu_count(database.pool()).await, 0);
    assert_eq!(detail_role_binding_count(database.pool()).await, 0);
    assert_eq!(menu_count(database.pool(), PERMISSION_CONFLICT_MENU_ID).await, 1);

    query("DELETE FROM sys_menu WHERE menu_id=$1")
        .bind(PERMISSION_CONFLICT_MENU_ID)
        .execute(database.pool())
        .await
        .unwrap();
    up(database.pool(), None).await.unwrap();
    assert_detail_seed(database.pool()).await;

    database.drop().await;
}

#[tokio::test]
async fn execution_detail_down_removes_owned_schema_menu_and_bindings() {
    let database = TestDatabase::create().await;
    migrate_through(database.pool(), DETAIL_MIGRATION_VERSION).await;
    insert_detail_down_role(database.pool()).await;
    query("INSERT INTO sys_role_menu (role_id, menu_id) VALUES ($1, $2)")
        .bind(DETAIL_DOWN_ROLE_ID)
        .bind(DETAIL_MENU_ID)
        .execute(database.pool())
        .await
        .unwrap();

    down(database.pool(), Some(1)).await.unwrap();

    assert_eq!(detail_column_count(database.pool()).await, 0);
    assert_eq!(detail_menu_count(database.pool()).await, 0);
    assert_eq!(detail_role_binding_count(database.pool()).await, 0);
    assert!(managed_table_exists(database.pool(), "sys_job_execution").await);
    assert_eq!(menu_count(database.pool(), "1092").await, 1);

    database.drop().await;
}

async fn insert_detail_down_role(pool: &PgPool) {
    query("INSERT INTO sys_role (role_id,role_name,role_key,role_sort,status,create_time) VALUES ($1,$1,$1,1,'0',CURRENT_TIMESTAMP)")
        .bind(DETAIL_DOWN_ROLE_ID)
        .execute(pool)
        .await
        .unwrap();
}

async fn assert_legacy_detail_is_null(pool: &PgPool) {
    let detail: (Option<String>, Option<i16>, Option<Value>) =
        query_as("SELECT detail_kind, detail_schema_version, detail_payload FROM sys_job_execution WHERE execution_id='legacy-execution'")
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(detail, (None, None, None));
}

async fn assert_detail_value_constraints(pool: &PgPool) {
    insert_terminal_execution(pool, "detail-execution").await;
    assert_constraint(
        query("UPDATE sys_job_execution SET detail_kind='http' WHERE execution_id='detail-execution'")
            .execute(pool)
            .await,
        "chk_sys_job_execution_detail_bundle",
    );
    for case in [
        DetailConstraintCase {
            kind: "",
            schema_version: 1,
            payload: json!({}),
            constraint: "chk_sys_job_execution_detail_kind",
        },
        DetailConstraintCase {
            kind: "\t",
            schema_version: 1,
            payload: json!({}),
            constraint: "chk_sys_job_execution_detail_kind",
        },
        DetailConstraintCase {
            kind: "http",
            schema_version: 0,
            payload: json!({}),
            constraint: "chk_sys_job_execution_detail_schema_version",
        },
        DetailConstraintCase {
            kind: "http",
            schema_version: 1,
            payload: json!([]),
            constraint: "chk_sys_job_execution_detail_payload",
        },
    ] {
        assert_detail_update_constraint(pool, case).await;
    }

    query("UPDATE sys_job_execution SET detail_kind='http', detail_schema_version=1, detail_payload=$1 WHERE execution_id='detail-execution'")
        .bind(json!({"request": {}, "response": {}}))
        .execute(pool)
        .await
        .unwrap();
    let payload: Value = query_scalar("SELECT detail_payload FROM sys_job_execution WHERE execution_id='detail-execution'")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(payload, json!({"request": {}, "response": {}}));
}

async fn assert_active_execution_detail_constraints(pool: &PgPool) {
    insert_pending_execution(pool).await;
    insert_running_execution(pool).await;
    for execution_id in ["pending-execution", "running-execution"] {
        let result = query("UPDATE sys_job_execution SET detail_kind='http', detail_schema_version=1, detail_payload='{}'::jsonb WHERE execution_id=$1")
            .bind(execution_id)
            .execute(pool)
            .await;
        assert_constraint(result, "chk_sys_job_execution_detail_lifecycle");
    }
}

async fn assert_detail_update_constraint(pool: &PgPool, case: DetailConstraintCase<'_>) {
    let result = query("UPDATE sys_job_execution SET detail_kind=$1, detail_schema_version=$2, detail_payload=$3 WHERE execution_id='detail-execution'")
        .bind(case.kind)
        .bind(case.schema_version)
        .bind(case.payload)
        .execute(pool)
        .await;
    assert_constraint(result, case.constraint);
}

fn assert_constraint(result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>, expected: &str) {
    let error = result.unwrap_err();
    assert_eq!(error.as_database_error().and_then(|value| value.constraint()), Some(expected));
}

struct DetailConstraintCase<'a> {
    kind: &'a str,
    schema_version: i16,
    payload: Value,
    constraint: &'static str,
}

async fn insert_terminal_execution(pool: &PgPool, execution_id: &str) {
    query(
        "INSERT INTO sys_job_execution (execution_id, job_id, job_revision, job_name, job_group, task_key, task_params, params_schema_version, repeatable, invoke_target, concurrent, trigger_type, scheduled_at, state, outcome, message_key, end_time, create_time) \
         VALUES ($1, 'detail-job', 1, 'detail job', 'SYSTEM', 'httpClient.request', '{}'::jsonb, 1, TRUE, 'httpClient.request', '0', 'S', CURRENT_TIMESTAMP, 'T', '2', 'scheduler.execution.skipped_misfire', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .bind(execution_id)
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_pending_execution(pool: &PgPool) {
    query(
        "INSERT INTO sys_job_execution (execution_id, job_id, job_revision, job_name, job_group, task_key, task_params, params_schema_version, repeatable, invoke_target, concurrent, trigger_type, scheduled_at, state, create_time) \
         VALUES ('pending-execution', 'pending-job', 1, 'pending', 'SYSTEM', 'httpClient.request', '{}'::jsonb, 1, TRUE, 'httpClient.request', '0', 'S', CURRENT_TIMESTAMP, 'P', CURRENT_TIMESTAMP)",
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_running_execution(pool: &PgPool) {
    query(
        "INSERT INTO sys_job_execution (execution_id, job_id, job_revision, job_name, job_group, task_key, task_params, params_schema_version, repeatable, invoke_target, concurrent, trigger_type, scheduled_at, state, executor_epoch, start_time, create_time) \
         VALUES ('running-execution', 'running-job', 1, 'running', 'SYSTEM', 'httpClient.request', '{}'::jsonb, 1, TRUE, 'httpClient.request', '0', 'S', CURRENT_TIMESTAMP, 'R', 'test-executor', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn insert_detail_menu_conflict(pool: &PgPool) {
    query("INSERT INTO sys_menu (menu_id, menu_name, parent_id, create_time) VALUES ($1, 'collision', '109', CURRENT_TIMESTAMP)")
        .bind(DETAIL_MENU_ID)
        .execute(pool)
        .await
        .unwrap();
}

async fn insert_detail_permission_conflict(pool: &PgPool) {
    query("INSERT INTO sys_menu (menu_id, menu_name, parent_id, perms, create_time) VALUES ($1, 'permission collision', '109', $2, CURRENT_TIMESTAMP)")
        .bind(PERMISSION_CONFLICT_MENU_ID)
        .bind(DETAIL_PERMISSION)
        .execute(pool)
        .await
        .unwrap();
}

async fn assert_detail_seed(pool: &PgPool) {
    assert_eq!(detail_column_count(pool).await, DETAIL_COLUMN_COUNT);
    assert_eq!(detail_menu_count(pool).await, 1);
    assert_eq!(detail_role_binding_count(pool).await, 1);
    assert_eq!(role_menu_binding_count(pool, SYSTEM_ADMIN_ROLE_ID, DETAIL_MENU_ID).await, 1);
    let permission: String = query_scalar("SELECT perms FROM sys_menu WHERE menu_id=$1")
        .bind(DETAIL_MENU_ID)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(permission, DETAIL_PERMISSION);
}

async fn detail_column_count(pool: &PgPool) -> i64 {
    query_scalar(
        "SELECT COUNT(*) FROM information_schema.columns WHERE table_schema='public' AND table_name='sys_job_execution' \
         AND column_name IN ('detail_kind','detail_schema_version','detail_payload')",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn detail_menu_count(pool: &PgPool) -> i64 {
    menu_count(pool, DETAIL_MENU_ID).await
}

async fn detail_role_binding_count(pool: &PgPool) -> i64 {
    query_scalar("SELECT COUNT(*) FROM sys_role_menu WHERE menu_id=$1")
        .bind(DETAIL_MENU_ID)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn role_menu_binding_count(pool: &PgPool, role_id: &str, menu_id: &str) -> i64 {
    query_scalar("SELECT COUNT(*) FROM sys_role_menu WHERE role_id=$1 AND menu_id=$2")
        .bind(role_id)
        .bind(menu_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn menu_count(pool: &PgPool, menu_id: &str) -> i64 {
    query_scalar("SELECT COUNT(*) FROM sys_menu WHERE menu_id=$1")
        .bind(menu_id)
        .fetch_one(pool)
        .await
        .unwrap()
}
