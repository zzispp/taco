use sqlx::{PgPool, query_as, query_scalar};

const SCHEDULER_MENUS: &[(&str, &str)] = &[
    ("108", "system:job:list"),
    ("1080", "system:job:query"),
    ("1081", "system:job:import"),
    ("1082", "system:job:edit"),
    ("1083", "system:job:remove"),
    ("1084", "system:job:export"),
    ("1085", "system:job:changeStatus"),
    ("1086", "system:job:run"),
    ("109", "system:job:log:list"),
    ("1090", "system:job:log:query"),
    ("1091", "system:job:log:remove"),
    ("1092", "system:job:log:export"),
    ("1093", "system:job:log:detail"),
];

const SCHEDULER_DICT_TYPES: &[(&str, &str)] = &[("scheduler-job-group", "sys_job_group"), ("scheduler-job-status", "sys_job_status")];

const SCHEDULER_DICT_DATA: &[(&str, &str, &str)] = &[
    ("scheduler-job-group-default", "sys_job_group", "DEFAULT"),
    ("scheduler-job-group-system", "sys_job_group", "SYSTEM"),
    ("scheduler-job-status-normal", "sys_job_status", "0"),
    ("scheduler-job-status-paused", "sys_job_status", "1"),
];

const SCHEDULER_INDEXES: &[&str] = &[
    "idx_sys_job_due",
    "idx_sys_job_execution_active_disallow",
    "idx_sys_job_execution_dispatch",
    "idx_sys_job_execution_job_time",
    "idx_sys_job_execution_occurrence",
    "idx_sys_job_execution_outcome_time",
    "idx_sys_job_group_status",
    "idx_sys_job_task_key_singleton",
];

pub(super) async fn assert_scheduler_seed(pool: &PgPool) {
    assert_eq!(scheduler_menus(pool).await, owned_pairs(SCHEDULER_MENUS));
    assert_eq!(scheduler_role_bindings(pool).await, 0);
    assert_eq!(scheduler_dict_types(pool).await, owned_pairs(SCHEDULER_DICT_TYPES));
    assert_eq!(scheduler_dict_data(pool).await, owned_triples(SCHEDULER_DICT_DATA));
    assert_eq!(scheduler_indexes(pool).await, owned_strings(SCHEDULER_INDEXES));
}

async fn scheduler_menus(pool: &PgPool) -> Vec<(String, String)> {
    query_as("SELECT menu_id, perms FROM sys_menu WHERE menu_id IN ('108','109','1080','1081','1082','1083','1084','1085','1086','1090','1091','1092','1093') ORDER BY menu_id")
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn scheduler_role_bindings(pool: &PgPool) -> i64 {
    query_scalar("SELECT COUNT(*) FROM sys_role_menu WHERE role_id = '2' AND menu_id IN ('108','109','1080','1081','1082','1083','1084','1085','1086','1090','1091','1092','1093')")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn scheduler_dict_types(pool: &PgPool) -> Vec<(String, String)> {
    query_as("SELECT dict_id, dict_type FROM sys_dict_type WHERE dict_id LIKE 'scheduler-%' ORDER BY dict_id")
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn scheduler_dict_data(pool: &PgPool) -> Vec<(String, String, String)> {
    query_as("SELECT dict_code, dict_type, dict_value FROM sys_dict_data WHERE dict_code LIKE 'scheduler-%' ORDER BY dict_code")
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn scheduler_indexes(pool: &PgPool) -> Vec<String> {
    query_scalar("SELECT indexname FROM pg_indexes WHERE schemaname = 'public' AND indexname LIKE 'idx_sys_job%' ORDER BY indexname")
        .fetch_all(pool)
        .await
        .unwrap()
}

fn owned_pairs(values: &[(&str, &str)]) -> Vec<(String, String)> {
    values.iter().map(|(left, right)| ((*left).into(), (*right).into())).collect()
}

fn owned_triples(values: &[(&str, &str, &str)]) -> Vec<(String, String, String)> {
    values
        .iter()
        .map(|(first, second, third)| ((*first).into(), (*second).into(), (*third).into()))
        .collect()
}

fn owned_strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).into()).collect()
}
