use sqlx::{PgPool, query, query_scalar};

use super::{TestDatabase, managed_table_exists, up};

const OWNER_TABLE: &str = "sys_installation_owner";
const OWNER_USER_ID: &str = "installation-owner";
const SECOND_OWNER_USER_ID: &str = "installation-owner-second";
const SINGLETON_ID: i16 = 1;
const INVALID_SINGLETON_ID: i16 = 2;
const PRIMARY_KEY_CONSTRAINT: &str = "sys_installation_owner_pkey";
const SINGLETON_CHECK_CONSTRAINT: &str = "chk_sys_installation_owner_singleton";
const OWNER_UNIQUE_CONSTRAINT: &str = "uq_sys_installation_owner_owner_user";
const OWNER_FOREIGN_KEY_CONSTRAINT: &str = "fk_sys_installation_owner_owner_user";
const PRIMARY_KEY_KIND: &str = "p";
const CHECK_KIND: &str = "c";
const UNIQUE_KIND: &str = "u";
const SINGLETON_PRIMARY_KEY: KeyConstraintSpec = KeyConstraintSpec::new(PRIMARY_KEY_CONSTRAINT, PRIMARY_KEY_KIND, "singleton_id");
const OWNER_UNIQUE_KEY: KeyConstraintSpec = KeyConstraintSpec::new(OWNER_UNIQUE_CONSTRAINT, UNIQUE_KIND, "owner_user_id");

#[tokio::test]
async fn installation_owner_migration_enforces_singleton_and_restricts_owner_deletion() {
    let database = TestDatabase::create().await;
    let pool = database.pool();

    up(pool, None).await.unwrap();

    assert_owner_schema(pool).await;
    insert_user(pool, OWNER_USER_ID).await;
    insert_user(pool, SECOND_OWNER_USER_ID).await;
    insert_owner(pool, SINGLETON_ID, OWNER_USER_ID).await.unwrap();

    assert_constraint(
        insert_owner(pool, INVALID_SINGLETON_ID, SECOND_OWNER_USER_ID).await.unwrap_err(),
        SINGLETON_CHECK_CONSTRAINT,
    );
    assert_constraint(
        insert_owner(pool, SINGLETON_ID, SECOND_OWNER_USER_ID).await.unwrap_err(),
        PRIMARY_KEY_CONSTRAINT,
    );
    assert_constraint(
        query("DELETE FROM sys_user WHERE user_id=$1")
            .bind(OWNER_USER_ID)
            .execute(pool)
            .await
            .unwrap_err(),
        OWNER_FOREIGN_KEY_CONSTRAINT,
    );

    database.drop().await;
}

async fn assert_owner_schema(pool: &PgPool) {
    assert!(managed_table_exists(pool, OWNER_TABLE).await);
    assert_eq!(owner_count(pool).await, 0);
    assert_eq!(constraint_count(pool, PRIMARY_KEY_CONSTRAINT, PRIMARY_KEY_KIND).await, 1);
    assert_eq!(constraint_count(pool, SINGLETON_CHECK_CONSTRAINT, CHECK_KIND).await, 1);
    assert_eq!(constraint_count(pool, OWNER_UNIQUE_CONSTRAINT, UNIQUE_KIND).await, 1);
    assert_eq!(key_column_constraint_count(pool, SINGLETON_PRIMARY_KEY).await, 1);
    assert_eq!(key_column_constraint_count(pool, OWNER_UNIQUE_KEY).await, 1);
    assert_eq!(owner_foreign_key_count(pool).await, 1);
}

async fn insert_user(pool: &PgPool, user_id: &str) {
    query("INSERT INTO sys_user (user_id,user_name,nick_name,email,password,create_time) VALUES ($1,$1,$1,$2,'password-hash',CURRENT_TIMESTAMP)")
        .bind(user_id)
        .bind(format!("{user_id}@example.invalid"))
        .execute(pool)
        .await
        .unwrap();
}

async fn insert_owner(pool: &PgPool, singleton_id: i16, owner_user_id: &str) -> Result<(), sqlx::Error> {
    query("INSERT INTO sys_installation_owner (singleton_id,owner_user_id) VALUES ($1,$2)")
        .bind(singleton_id)
        .bind(owner_user_id)
        .execute(pool)
        .await
        .map(|_| ())
}

async fn owner_count(pool: &PgPool) -> i64 {
    query_scalar("SELECT COUNT(*) FROM sys_installation_owner").fetch_one(pool).await.unwrap()
}

async fn constraint_count(pool: &PgPool, name: &str, kind: &str) -> i64 {
    query_scalar("SELECT COUNT(*) FROM pg_constraint WHERE conrelid='sys_installation_owner'::regclass AND conname=$1 AND contype=$2")
        .bind(name)
        .bind(kind)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn key_column_constraint_count(pool: &PgPool, spec: KeyConstraintSpec) -> i64 {
    query_scalar(
        "SELECT COUNT(*) FROM pg_constraint AS con JOIN pg_attribute AS attr ON attr.attrelid=con.conrelid AND attr.attnum=ANY(con.conkey) WHERE con.conrelid='sys_installation_owner'::regclass AND con.conname=$1 AND con.contype=$2 AND attr.attname=$3",
    )
    .bind(spec.name)
    .bind(spec.kind)
    .bind(spec.column)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn owner_foreign_key_count(pool: &PgPool) -> i64 {
    query_scalar(
        "SELECT COUNT(*) FROM pg_constraint WHERE conrelid='sys_installation_owner'::regclass AND conname=$1 AND contype='f' AND confrelid='sys_user'::regclass AND confdeltype='r'",
    )
    .bind(OWNER_FOREIGN_KEY_CONSTRAINT)
    .fetch_one(pool)
    .await
    .unwrap()
}

fn assert_constraint(error: sqlx::Error, expected: &str) {
    let actual = error.as_database_error().and_then(|database| database.constraint());
    assert_eq!(actual, Some(expected));
}

#[derive(Clone, Copy)]
struct KeyConstraintSpec {
    name: &'static str,
    kind: &'static str,
    column: &'static str,
}

impl KeyConstraintSpec {
    const fn new(name: &'static str, kind: &'static str, column: &'static str) -> Self {
        Self { name, kind, column }
    }
}
