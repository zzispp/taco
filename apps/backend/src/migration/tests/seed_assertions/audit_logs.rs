use sqlx::{PgPool, query_scalar};

const AUDIT_MENUS: &[(&str, &str, &str, Option<&str>)] = &[
    ("111", "3", "M", None),
    ("112", "111", "C", Some("system:operlog:list")),
    ("113", "111", "C", Some("system:logininfor:list")),
    ("1120", "112", "F", Some("system:operlog:query")),
    ("1121", "112", "F", Some("system:operlog:remove")),
    ("1122", "112", "F", Some("system:operlog:export")),
    ("1130", "113", "F", Some("system:logininfor:remove")),
    ("1131", "113", "F", Some("system:logininfor:export")),
    ("1132", "113", "F", Some("system:logininfor:unlock")),
];

const OPERATION_TYPES: &[(&str, &str)] = &[
    ("0", "其他"),
    ("1", "新增"),
    ("2", "修改"),
    ("3", "删除"),
    ("4", "授权"),
    ("5", "导出"),
    ("6", "导入"),
    ("7", "强退"),
    ("8", "生成代码"),
    ("9", "清空"),
];

pub(super) async fn assert_audit_log_seed(pool: &PgPool) {
    assert_menus(pool).await;
    assert_operation_type_dictionary(pool).await;
    assert_private_runtime_config(pool).await;
    assert_single_ip_location_key(pool).await;
}

async fn assert_menus(pool: &PgPool) {
    for (id, parent, menu_type, permission) in AUDIT_MENUS {
        let count: i64 =
            query_scalar("SELECT COUNT(*) FROM sys_menu WHERE menu_id=$1 AND parent_id=$2 AND menu_type=$3 AND perms IS NOT DISTINCT FROM $4 AND status='0'")
                .bind(id)
                .bind(parent)
                .bind(menu_type)
                .bind(permission)
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!(count, 1, "missing audit menu {id}");
    }
}

async fn assert_operation_type_dictionary(pool: &PgPool) {
    let dict: i64 = query_scalar("SELECT COUNT(*) FROM sys_dict_type WHERE dict_type='sys_oper_type' AND status='0'")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(dict, 1);

    for (value, label) in OPERATION_TYPES {
        let count: i64 = query_scalar("SELECT COUNT(*) FROM sys_dict_data WHERE dict_type='sys_oper_type' AND dict_value=$1 AND dict_label=$2 AND status='0'")
            .bind(value)
            .bind(label)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "missing operation type {value}");
    }
}

async fn assert_private_runtime_config(pool: &PgPool) {
    let count: i64 = query_scalar("SELECT COUNT(*) FROM sys_config WHERE config_key='sys.auth.loginLockConfig' AND public_read=FALSE")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

async fn assert_single_ip_location_key(pool: &PgPool) {
    let old: i64 = query_scalar("SELECT COUNT(*) FROM sys_config WHERE config_key='sys.auth.ipLocationConfig'")
        .fetch_one(pool)
        .await
        .unwrap();
    let new: i64 = query_scalar("SELECT COUNT(*) FROM sys_config WHERE config_key='sys.client.ipLocationConfig'")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!((old, new), (0, 1));
}
