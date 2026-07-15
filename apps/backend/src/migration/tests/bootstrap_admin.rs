use sqlx::{query, query_scalar};
use user::{
    application::{BootstrapAdminInput, PasswordHasher},
    infra::Argon2PasswordHasher,
};

use super::{TestDatabase, fresh};
use crate::composition::{bootstrap_admin, tests::test_settings};

#[tokio::test]
async fn bootstrap_admin_allows_ordinary_users_then_rejects_a_second_super_admin() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();
    insert_ordinary_user(database.pool()).await;
    let mut settings = test_settings();
    settings.database.url = Some(database.database_url());
    settings.database.password = None;

    let user = bootstrap_admin(&settings, input("root-admin")).await.unwrap();

    assert_eq!(user.username, "root-admin");
    assert_eq!(user.role_ids, vec!["1"]);
    assert_eq!(user_count(database.pool()).await, 2);
    assert!(
        Argon2PasswordHasher
            .verify("safe-secret-123", &password_hash(database.pool(), &user.id.0).await)
            .unwrap()
    );

    let error = bootstrap_admin(&settings, input("second-admin")).await.unwrap_err();
    assert!(error.to_string().contains("errors.user.bootstrap_admin_exists"));
    assert_eq!(user_count(database.pool()).await, 2);
    database.drop().await;
}

#[tokio::test]
async fn bootstrap_admin_rejects_a_disabled_existing_super_admin() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();
    let mut settings = test_settings();
    settings.database.url = Some(database.database_url());
    settings.database.password = None;
    let admin = bootstrap_admin(&settings, input("disabled-admin")).await.unwrap();
    query("UPDATE sys_user SET status='1' WHERE user_id=$1")
        .bind(&admin.id.0)
        .execute(database.pool())
        .await
        .unwrap();

    let error = bootstrap_admin(&settings, input("second-admin")).await.unwrap_err();

    assert!(error.to_string().contains("errors.user.bootstrap_admin_exists"));
    assert_eq!(user_count(database.pool()).await, 1);
    database.drop().await;
}

async fn insert_ordinary_user(pool: &sqlx::PgPool) {
    query(
        "INSERT INTO sys_user (user_id,user_name,nick_name,email,password,create_time) VALUES ('1','ordinary','Ordinary','ordinary@test.invalid','test-hash',CURRENT_TIMESTAMP)",
    )
    .execute(pool)
    .await
    .unwrap();
    query("INSERT INTO sys_user_role (user_id,role_id) VALUES ('1','2')")
        .execute(pool)
        .await
        .unwrap();
}

fn input(username: &str) -> BootstrapAdminInput {
    BootstrapAdminInput {
        username: username.into(),
        email: format!("{username}@example.com"),
        password: "safe-secret-123".into(),
    }
}

async fn user_count(pool: &sqlx::PgPool) -> i64 {
    query_scalar("SELECT COUNT(*) FROM sys_user WHERE del_flag='0'").fetch_one(pool).await.unwrap()
}

async fn password_hash(pool: &sqlx::PgPool, user_id: &str) -> String {
    query_scalar("SELECT password FROM sys_user WHERE user_id=$1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap()
}
