use sqlx::query_scalar;

use super::{TestDatabase, managed_table_exists, rollback_from, up};

const AUDIT_LOGS_MIGRATION_VERSION: i64 = 20260713000004;

#[tokio::test]
async fn audit_log_migration_down_removes_outbox_and_projected_logs() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();

    rollback_from(database.pool(), AUDIT_LOGS_MIGRATION_VERSION).await;

    assert!(!managed_table_exists(database.pool(), "audit_outbox").await);
    assert!(!managed_table_exists(database.pool(), "sys_oper_log").await);
    assert!(!managed_table_exists(database.pool(), "sys_logininfor").await);
    assert_eq!(count_key(database.pool(), "sys.auth.ipLocationConfig").await, 1);
    assert_eq!(count_key(database.pool(), "sys.client.ipLocationConfig").await, 0);
    assert_eq!(count_key(database.pool(), "sys.auth.loginLockConfig").await, 0);
    assert_eq!(menu_count(database.pool()).await, 0);
    assert_eq!(operation_type_count(database.pool()).await, 0);

    database.drop().await;
}

async fn count_key(pool: &sqlx::PgPool, key: &str) -> i64 {
    query_scalar("SELECT COUNT(*) FROM sys_config WHERE config_key=$1")
        .bind(key)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn menu_count(pool: &sqlx::PgPool) -> i64 {
    query_scalar("SELECT COUNT(*) FROM sys_menu WHERE menu_id = ANY($1)")
        .bind(["111", "112", "113", "1120", "1121", "1122", "1130", "1131", "1132"])
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn operation_type_count(pool: &sqlx::PgPool) -> i64 {
    query_scalar("SELECT COUNT(*) FROM sys_dict_data WHERE dict_type='sys_oper_type'")
        .fetch_one(pool)
        .await
        .unwrap()
}
