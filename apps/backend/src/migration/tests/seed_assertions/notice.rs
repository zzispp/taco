use sqlx::{PgPool, query_scalar};

const NOTICE_PERMISSIONS: &[(&str, &str, &str, &str)] = &[
    ("110", "1", "C", "system:notice:list"),
    ("1100", "110", "F", "system:notice:query"),
    ("1101", "110", "F", "system:notice:add"),
    ("1102", "110", "F", "system:notice:edit"),
    ("1103", "110", "F", "system:notice:remove"),
];
const NOTICE_DICT_TYPES: &[(&str, &str, &str)] = &[
    ("notice-type", "通知类型", "sys_notice_type"),
    ("notice-status", "通知状态", "sys_notice_status"),
];
const NOTICE_DICT_DATA: &[(&str, &str, &str, &str, &str, &str)] = &[
    ("notice-type-notice", "sys_notice_type", "1", "通知", "Y", "warning"),
    ("notice-type-announcement", "sys_notice_type", "2", "公告", "N", "success"),
    ("notice-status-normal", "sys_notice_status", "0", "正常", "Y", "primary"),
    ("notice-status-closed", "sys_notice_status", "1", "关闭", "N", "danger"),
];
const NOTICE_INDEXES: &[(&str, &str)] = &[
    ("sys_notice", "idx_sys_notice_order"),
    ("sys_notice", "idx_sys_notice_active"),
    ("sys_notice_read", "idx_sys_notice_read_notice_time"),
];

pub(super) async fn assert_notice_seed(pool: &PgPool) {
    assert_notice_permissions(pool).await;
    assert_notice_role_bindings(pool).await;
    assert_notice_dict_types(pool).await;
    assert_notice_dict_data(pool).await;
    assert_notice_schema(pool).await;
    assert_notice_tables_are_empty(pool).await;
}

async fn assert_notice_permissions(pool: &PgPool) {
    for (menu_id, parent_id, menu_type, permission) in NOTICE_PERMISSIONS {
        let count: i64 = query_scalar("SELECT COUNT(*) FROM sys_menu WHERE menu_id=$1 AND parent_id=$2 AND menu_type=$3 AND perms=$4 AND status='0'")
            .bind(menu_id)
            .bind(parent_id)
            .bind(menu_type)
            .bind(permission)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "missing notice permission {menu_id}:{permission}");
    }
    let menu_count: i64 = query_scalar("SELECT COUNT(*) FROM sys_menu WHERE menu_id='110' AND path='/dashboard/admin/notices' AND icon='icon.notice'")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(menu_count, 1);
}

async fn assert_notice_role_bindings(pool: &PgPool) {
    for (menu_id, _, _, _) in NOTICE_PERMISSIONS {
        let count: i64 = query_scalar("SELECT COUNT(*) FROM sys_role_menu WHERE role_id='2' AND menu_id=$1")
            .bind(menu_id)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "role 2 missing notice menu {menu_id}");
    }
}

async fn assert_notice_dict_types(pool: &PgPool) {
    for (dict_id, dict_name, dict_type) in NOTICE_DICT_TYPES {
        let count: i64 = query_scalar("SELECT COUNT(*) FROM sys_dict_type WHERE dict_id=$1 AND dict_name=$2 AND dict_type=$3 AND status='0'")
            .bind(dict_id)
            .bind(dict_name)
            .bind(dict_type)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "missing notice dict type {dict_type}");
    }
}

async fn assert_notice_dict_data(pool: &PgPool) {
    for (code, dict_type, value, label, is_default, list_class) in NOTICE_DICT_DATA {
        let count: i64 = query_scalar(
            "SELECT COUNT(*) FROM sys_dict_data WHERE dict_code=$1 AND dict_type=$2 AND dict_value=$3 AND dict_label=$4 AND is_default=$5 AND list_class=$6 AND status='0'",
        )
        .bind(code)
        .bind(dict_type)
        .bind(value)
        .bind(label)
        .bind(is_default)
        .bind(list_class)
        .fetch_one(pool)
        .await
        .unwrap();
        assert_eq!(count, 1, "invalid notice dict data {code}");
    }
}

async fn assert_notice_schema(pool: &PgPool) {
    assert_notice_checks(pool).await;
    assert_notice_indexes(pool).await;
    assert_notice_unique_constraint(pool).await;
    assert_notice_cascades(pool).await;
}

async fn assert_notice_checks(pool: &PgPool) {
    for constraint in ["chk_sys_notice_type", "chk_sys_notice_status"] {
        let count: i64 = query_scalar("SELECT COUNT(*) FROM pg_constraint WHERE conrelid='sys_notice'::regclass AND conname=$1 AND contype='c'")
            .bind(constraint)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "missing notice check {constraint}");
    }
}

async fn assert_notice_indexes(pool: &PgPool) {
    for (table, index) in NOTICE_INDEXES {
        let count: i64 = query_scalar("SELECT COUNT(*) FROM pg_indexes WHERE tablename=$1 AND indexname=$2")
            .bind(table)
            .bind(index)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "missing notice index {table}.{index}");
    }
}

async fn assert_notice_unique_constraint(pool: &PgPool) {
    let count: i64 = query_scalar(
        "SELECT COUNT(*) FROM pg_constraint WHERE conrelid='sys_notice_read'::regclass AND conname='uk_sys_notice_read_user_notice' AND contype='u'",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(count, 1);
}

async fn assert_notice_cascades(pool: &PgPool) {
    for (constraint, referenced_table) in [("fk_sys_notice_read_notice", "sys_notice"), ("fk_sys_notice_read_user", "sys_user")] {
        let count: i64 = query_scalar(
            "SELECT COUNT(*) FROM pg_constraint WHERE conrelid='sys_notice_read'::regclass AND conname=$1 AND contype='f' AND confrelid=$2::regclass AND confdeltype='c'",
        )
        .bind(constraint)
        .bind(referenced_table)
        .fetch_one(pool)
        .await
        .unwrap();
        assert_eq!(count, 1, "missing notice cascade {constraint}");
    }
}

async fn assert_notice_tables_are_empty(pool: &PgPool) {
    let notice_count: i64 = query_scalar("SELECT COUNT(*) FROM sys_notice").fetch_one(pool).await.unwrap();
    let read_count: i64 = query_scalar("SELECT COUNT(*) FROM sys_notice_read").fetch_one(pool).await.unwrap();
    assert_eq!(notice_count, 0);
    assert_eq!(read_count, 0);
}
