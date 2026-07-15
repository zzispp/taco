use sqlx::{PgPool, query, query_as, query_scalar};

use super::{TestDatabase, managed_table_exists, up};

#[path = "audit_outbox_contract.rs"]
mod outbox;

const MIGRATIONS_BEFORE_AUDIT: u32 = 16;
const OPERATION_CONSTRAINTS: &[&str] = &[
    "chk_sys_oper_log_business_type",
    "chk_sys_oper_log_operator_type",
    "chk_sys_oper_log_status",
    "chk_sys_oper_log_cost_time",
    "chk_sys_oper_log_location_kind",
    "chk_sys_oper_log_location_value",
];
const LOGIN_CONSTRAINTS: &[&str] = &[
    "chk_sys_logininfor_status",
    "chk_sys_logininfor_event_type",
    "chk_sys_logininfor_location_kind",
    "chk_sys_logininfor_location_value",
];
const INDEXES: &[(&str, &str)] = &[
    ("sys_oper_log", "idx_sys_oper_log_business_type_cursor"),
    ("sys_oper_log", "idx_sys_oper_log_cost_cursor"),
    ("sys_oper_log", "idx_sys_oper_log_ingested_cursor"),
    ("sys_oper_log", "idx_sys_oper_log_operator_cursor"),
    ("sys_oper_log", "idx_sys_oper_log_status_cursor"),
    ("sys_oper_log", "idx_sys_oper_log_time"),
    ("sys_logininfor", "idx_sys_logininfor_event_type"),
    ("sys_logininfor", "idx_sys_logininfor_ingested_cursor"),
    ("sys_logininfor", "idx_sys_logininfor_ip_cursor"),
    ("sys_logininfor", "idx_sys_logininfor_status_cursor"),
    ("sys_logininfor", "idx_sys_logininfor_time"),
    ("sys_logininfor", "idx_sys_logininfor_user_cursor"),
];

#[tokio::test]
async fn audit_log_migration_creates_schema_constraints_indexes_and_seed() {
    let database = TestDatabase::create().await;
    up(database.pool(), None).await.unwrap();

    assert!(managed_table_exists(database.pool(), "sys_oper_log").await);
    assert!(managed_table_exists(database.pool(), "sys_logininfor").await);
    assert!(managed_table_exists(database.pool(), "audit_outbox").await);
    assert_constraints(database.pool(), "sys_oper_log", OPERATION_CONSTRAINTS).await;
    assert_constraints(database.pool(), "sys_logininfor", LOGIN_CONSTRAINTS).await;
    assert_constraints(database.pool(), "audit_outbox", outbox::OUTBOX_CONSTRAINTS).await;
    assert_indexes(database.pool()).await;
    outbox::assert_outbox_partial_indexes(database.pool()).await;
    assert_required_location_kind_columns(database.pool()).await;
    assert_ingested_at_columns(database.pool()).await;
    assert_rejects_missing_location_kind(database.pool()).await;
    assert_rejects_invalid_codes(database.pool()).await;
    outbox::assert_outbox_rejects_invalid_delivery_state(database.pool()).await;
    assert_location_contract(database.pool()).await;

    database.drop().await;
}

#[tokio::test]
async fn audit_log_migration_rejects_conflicting_ip_location_key() {
    let database = TestDatabase::create().await;
    up(database.pool(), Some(MIGRATIONS_BEFORE_AUDIT)).await.unwrap();
    query("INSERT INTO sys_config (config_id,config_name,config_key,config_value,config_type,public_read,create_time) VALUES ('conflict','conflict','sys.client.ipLocationConfig','{}','Y',FALSE,CURRENT_TIMESTAMP)")
        .execute(database.pool())
        .await
        .unwrap();

    let error = up(database.pool(), Some(1)).await.unwrap_err();
    assert!(error.to_string().contains("sys.client.ipLocationConfig"), "unexpected error: {error}");

    database.drop().await;
}

async fn assert_constraints(pool: &PgPool, table: &str, constraints: &[&str]) {
    for constraint in constraints {
        let count: i64 = query_scalar("SELECT COUNT(*) FROM pg_constraint WHERE conrelid=$1::regclass AND conname=$2 AND contype='c'")
            .bind(table)
            .bind(constraint)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "missing constraint {table}.{constraint}");
    }
}

async fn assert_indexes(pool: &PgPool) {
    for (table, index) in INDEXES {
        let count: i64 = query_scalar("SELECT COUNT(*) FROM pg_indexes WHERE tablename=$1 AND indexname=$2")
            .bind(table)
            .bind(index)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(count, 1, "missing index {table}.{index}");
    }
}

async fn assert_rejects_invalid_codes(pool: &PgPool) {
    let operation = query("INSERT INTO sys_oper_log (oper_id,title,business_type,method,request_method,operator_type,oper_location_kind,status,oper_time,cost_time) VALUES ('invalid','x',10,'x','POST',1,'unknown',0,CURRENT_TIMESTAMP,0)")
        .execute(pool)
        .await;
    let login = query(
        "INSERT INTO sys_logininfor (info_id,user_name,login_location_kind,status,event_type,message_key,login_time) VALUES ('invalid','x','unknown',0,'unknown','x',CURRENT_TIMESTAMP)",
    )
    .execute(pool)
    .await;
    assert!(operation.is_err());
    assert!(login.is_err());
}

async fn assert_required_location_kind_columns(pool: &PgPool) {
    for (table, column) in [("sys_oper_log", "oper_location_kind"), ("sys_logininfor", "login_location_kind")] {
        let value: (String, Option<String>) =
            query_as("SELECT is_nullable,column_default FROM information_schema.columns WHERE table_name=$1 AND column_name=$2")
                .bind(table)
                .bind(column)
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!(value, ("NO".into(), None), "unexpected definition for {table}.{column}");
    }
}

async fn assert_ingested_at_columns(pool: &PgPool) {
    for table in ["sys_oper_log", "sys_logininfor"] {
        let value: (String, String, Option<String>) =
            query_as("SELECT data_type,is_nullable,column_default FROM information_schema.columns WHERE table_name=$1 AND column_name='ingested_at'")
                .bind(table)
                .fetch_one(pool)
                .await
                .unwrap();
        assert_eq!(value.0, "timestamp with time zone", "unexpected {table}.ingested_at type");
        assert_eq!(value.1, "NO", "unexpected {table}.ingested_at nullability");
        assert!(
            value.2.is_some_and(|default| default.contains("clock_timestamp")),
            "missing {table}.ingested_at default"
        );
    }
}

async fn assert_rejects_missing_location_kind(pool: &PgPool) {
    let operation = query("INSERT INTO sys_oper_log (oper_id,title,business_type,method,request_method,operator_type,status,oper_time,cost_time) VALUES ('missing-kind','x',0,'x','POST',1,0,CURRENT_TIMESTAMP,0)")
        .execute(pool)
        .await;
    let login = query(
        "INSERT INTO sys_logininfor (info_id,user_name,status,event_type,message_key,login_time) VALUES ('missing-kind','x',0,'login_success','x',CURRENT_TIMESTAMP)",
    )
    .execute(pool)
    .await;
    assert!(operation.is_err());
    assert!(login.is_err());
}

async fn assert_location_contract(pool: &PgPool) {
    let invalid_operations = [
        LocationFixture::new("oper-kind", "other", ""),
        LocationFixture::new("oper-empty", "resolved", ""),
        LocationFixture::new("oper-internal", "internal", "Shanghai"),
    ];
    for fixture in invalid_operations {
        let result = insert_operation_location(pool, &fixture).await;
        assert!(result.is_err(), "accepted invalid operation location {}:{}", fixture.kind, fixture.value);
    }
    insert_operation_location(pool, &LocationFixture::new("oper-valid", "resolved", "Shanghai"))
        .await
        .unwrap();

    let invalid_logins = [
        LocationFixture::new("login-kind", "other", ""),
        LocationFixture::new("login-empty", "resolved", ""),
        LocationFixture::new("login-unknown", "unknown", "Shanghai"),
    ];
    for fixture in invalid_logins {
        let result = insert_login_location(pool, &fixture).await;
        assert!(result.is_err(), "accepted invalid login location {}:{}", fixture.kind, fixture.value);
    }
    insert_login_location(pool, &LocationFixture::new("login-valid", "unknown", "")).await.unwrap();
}

async fn insert_operation_location(pool: &PgPool, fixture: &LocationFixture<'_>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    query("INSERT INTO sys_oper_log (oper_id,title,business_type,method,request_method,operator_type,oper_location_kind,oper_location,status,oper_time,cost_time) VALUES ($1,'x',0,'x','POST',1,$2,$3,0,CURRENT_TIMESTAMP,0)")
        .bind(fixture.id)
        .bind(fixture.kind)
        .bind(fixture.value)
        .execute(pool)
        .await
}

async fn insert_login_location(pool: &PgPool, fixture: &LocationFixture<'_>) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    query("INSERT INTO sys_logininfor (info_id,user_name,login_location_kind,login_location,status,event_type,message_key,login_time) VALUES ($1,'x',$2,$3,0,'login_success','x',CURRENT_TIMESTAMP)")
        .bind(fixture.id)
        .bind(fixture.kind)
        .bind(fixture.value)
        .execute(pool)
        .await
}

struct LocationFixture<'a> {
    id: &'a str,
    kind: &'a str,
    value: &'a str,
}

impl<'a> LocationFixture<'a> {
    const fn new(id: &'a str, kind: &'a str, value: &'a str) -> Self {
        Self { id, kind, value }
    }
}
