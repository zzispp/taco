use sqlx::{query, query_scalar};

use super::{TestDatabase, managed_table_exists, rollback_from, up};

const CUSTOM_DICT_CODE: &str = "notice-type-custom";
const NOTICE_MIGRATION_VERSION: i64 = 20260713000001;

#[tokio::test]
async fn notice_down_removes_custom_notice_dict_data() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    insert_custom_notice_dict_data(database.pool()).await;

    rollback_from(database.pool(), NOTICE_MIGRATION_VERSION).await;

    let remaining: i64 = query_scalar("SELECT COUNT(*) FROM sys_dict_data WHERE dict_type IN ('sys_notice_type', 'sys_notice_status')")
        .fetch_one(database.pool())
        .await
        .unwrap();
    assert_eq!(remaining, 0);
    assert!(!managed_table_exists(database.pool(), "sys_notice").await);
    assert!(!managed_table_exists(database.pool(), "sys_notice_read").await);

    database.drop().await;
}

async fn insert_custom_notice_dict_data(pool: &sqlx::PgPool) {
    query(
        "INSERT INTO sys_dict_data (dict_code,dict_sort,dict_label,dict_value,dict_type,css_class,list_class,is_default,status,create_by,create_time,remark) VALUES ($1,99,'自定义通知','9','sys_notice_type','','info','N','0','admin',CURRENT_TIMESTAMP,'回滚测试')",
    )
    .bind(CUSTOM_DICT_CODE)
    .execute(pool)
    .await
    .unwrap();
}
