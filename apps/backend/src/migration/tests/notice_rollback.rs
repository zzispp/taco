use sqlx::{query, query_scalar};

use super::{TestDatabase, down, managed_table_exists, up};

const CUSTOM_DICT_CODE: &str = "notice-type-custom";

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn notice_down_removes_custom_notice_dict_data() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();
    insert_custom_notice_dict_data(database.pool()).await;

    down(database.pool(), Some(3)).await.unwrap();

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
