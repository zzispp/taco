use sqlx::query;

use super::{MIGRATION_TOTAL, TestDatabase, managed_table_exists, up};

const MIGRATIONS_BEFORE_SCHEDULER: u32 = MIGRATION_TOTAL as u32 - 2;

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn scheduler_seed_conflicts_roll_back_the_whole_migration() {
    let database = TestDatabase::create().await;
    let pool = database.pool();
    up(pool, Some(MIGRATIONS_BEFORE_SCHEDULER)).await.unwrap();

    insert_menu_conflict(pool).await;
    assert_scheduler_migration_fails(pool).await;
    query("DELETE FROM sys_menu WHERE menu_id = '108'").execute(pool).await.unwrap();

    insert_dict_type_conflict(pool).await;
    assert_scheduler_migration_fails(pool).await;
    query("DELETE FROM sys_dict_type WHERE dict_id = 'scheduler-job-group'")
        .execute(pool)
        .await
        .unwrap();

    insert_dict_data_conflict(pool).await;
    assert_scheduler_migration_fails(pool).await;
    query("DELETE FROM sys_dict_data WHERE dict_code = 'scheduler-job-group-default'")
        .execute(pool)
        .await
        .unwrap();

    up(pool, None).await.unwrap();
    assert!(managed_table_exists(pool, "sys_job_execution").await);
    database.drop().await;
}

async fn assert_scheduler_migration_fails(pool: &sqlx::PgPool) {
    assert!(up(pool, None).await.is_err());
    assert!(!managed_table_exists(pool, "sys_job").await);
    assert!(!managed_table_exists(pool, "sys_job_execution").await);
}

async fn insert_menu_conflict(pool: &sqlx::PgPool) {
    query("INSERT INTO sys_menu (menu_id, menu_name, parent_id, create_time) VALUES ('108', 'collision', '1', CURRENT_TIMESTAMP)")
        .execute(pool)
        .await
        .unwrap();
}

async fn insert_dict_type_conflict(pool: &sqlx::PgPool) {
    query("INSERT INTO sys_dict_type (dict_id, dict_name, dict_type, create_time) VALUES ('scheduler-job-group', 'collision', 'collision', CURRENT_TIMESTAMP)")
        .execute(pool)
        .await
        .unwrap();
}

async fn insert_dict_data_conflict(pool: &sqlx::PgPool) {
    query("INSERT INTO sys_dict_data (dict_code, dict_label, dict_value, dict_type, create_time) VALUES ('scheduler-job-group-default', 'collision', 'collision', 'collision', CURRENT_TIMESTAMP)")
        .execute(pool)
        .await
        .unwrap();
}
