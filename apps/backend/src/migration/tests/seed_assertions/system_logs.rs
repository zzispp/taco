use sqlx::{PgPool, query_as, query_scalar};

const LOG_PAGE_RELATIONS: &[(&str, i64)] = &[("112", 1), ("113", 2), ("109", 3), ("114", 4)];

pub(super) async fn assert_system_log_seed(pool: &PgPool) {
    assert_runtime_config(pool).await;
    assert_menus(pool).await;
    assert_cleanup_job(pool).await;
}

async fn assert_runtime_config(pool: &PgPool) {
    let row: (bool, String) = query_as("SELECT public_read,remark FROM sys_config WHERE config_key='sys.observability.tracingConfig'")
        .fetch_one(pool)
        .await
        .unwrap();
    assert!(!row.0);
    for field in ["log_level", "max_body_capture_bytes", "slow_operation_ms"] {
        assert!(row.1.contains(field), "runtime config remark is missing {field}");
    }
}

async fn assert_menus(pool: &PgPool) {
    let bindings: i64 = query_scalar("SELECT COUNT(*) FROM sys_role_menu WHERE role_id='2' AND menu_id IN ('114','1140','1141','1142')")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(bindings, 4);
    assert_log_page_relations(pool).await;
}

async fn assert_log_page_relations(pool: &PgPool) {
    let relations: Vec<(String, i64)> =
        query_as("SELECT menu_id,order_num FROM sys_menu WHERE menu_id IN ('109','112','113','114') AND parent_id='111' ORDER BY order_num,menu_id")
            .fetch_all(pool)
            .await
            .unwrap();
    let expected = LOG_PAGE_RELATIONS.iter().map(|(id, order)| ((*id).into(), *order)).collect::<Vec<_>>();
    assert_eq!(relations, expected);
}

async fn assert_cleanup_job(pool: &PgPool) {
    let row: (String, String, String) =
        query_as("SELECT cron_expression,status,remark FROM sys_job WHERE job_id='system-log-cleanup' AND task_key='observability.cleanupSystemLogs'")
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!((row.0, row.1), ("0 0 19 * * *".into(), "0".into()));
    assert!(row.2.contains("batch_size"));
}
