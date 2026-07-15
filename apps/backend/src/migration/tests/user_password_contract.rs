use sqlx::{PgPool, postgres::PgDatabaseError, query, query_as};

use super::{TestDatabase, fresh};

const NOT_NULL_VIOLATION: &str = "23502";

#[tokio::test]
async fn user_password_is_required_without_a_database_default() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();

    assert_password_column(database.pool()).await;
    assert_missing_password_is_rejected(database.pool()).await;

    database.drop().await;
}

async fn assert_password_column(pool: &PgPool) {
    let definition: (String, Option<String>) = query_as(
        "SELECT is_nullable,column_default FROM information_schema.columns \
         WHERE table_schema='public' AND table_name='sys_user' AND column_name='password'",
    )
    .fetch_one(pool)
    .await
    .unwrap();

    assert_eq!(definition, ("NO".into(), None));
}

async fn assert_missing_password_is_rejected(pool: &PgPool) {
    let error = query(
        "INSERT INTO sys_user (user_id,user_name,nick_name,create_time) \
         VALUES ('missing-password','missing-password','Missing Password',CURRENT_TIMESTAMP)",
    )
    .execute(pool)
    .await
    .unwrap_err();
    let database_error = error.as_database_error().unwrap().downcast_ref::<PgDatabaseError>();

    assert_eq!(database_error.code(), NOT_NULL_VIOLATION);
    assert_eq!(database_error.table(), Some("sys_user"));
    assert_eq!(database_error.column(), Some("password"));
}
