use sqlx::{PgPool, query_as, query_scalar};
use storage::Database;
use user::{
    application::{BootstrapAdministratorOutcome, BootstrapAdministratorRecord, BootstrapAdministratorRepository},
    infra::StorageUserRepository,
};

use super::{TestDatabase, fresh};

#[tokio::test]
async fn runtime_requires_an_enabled_system_administrator() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();

    let error = crate::composition::ensure_enabled_system_administrator(Database::new(database.pool().clone()))
        .await
        .unwrap_err();

    assert!(error.to_string().contains("no enabled protected system administrator exists"));
    database.drop().await;
}

#[tokio::test]
async fn bootstrap_administrator_creation_is_atomic_and_binds_the_system_role() {
    let database = TestDatabase::create().await;
    fresh(database.pool()).await.unwrap();
    let repository = StorageUserRepository::new(Database::new(database.pool().clone()));

    let (first, second) = tokio::join!(
        repository.create_system_administrator_if_absent(record("first-admin", "first-admin@example.test")),
        repository.create_system_administrator_if_absent(record("second-admin", "second-admin@example.test"))
    );

    let outcomes = [first.unwrap(), second.unwrap()];
    assert_eq!(
        outcomes
            .into_iter()
            .filter(|outcome| *outcome == BootstrapAdministratorOutcome::Created)
            .count(),
        1
    );
    assert!(repository.has_enabled_system_administrator().await.unwrap());
    assert_system_administrator_binding(database.pool()).await;
    database.drop().await;
}

fn record(username: &str, email: &str) -> BootstrapAdministratorRecord {
    BootstrapAdministratorRecord {
        username: username.into(),
        nick_name: username.into(),
        email: email.into(),
        password_hash: "test-password-hash".into(),
    }
}

async fn assert_system_administrator_binding(pool: &PgPool) {
    let count: i64 = query_scalar("SELECT COUNT(*) FROM sys_user WHERE del_flag='0'").fetch_one(pool).await.unwrap();
    let binding: (bool, String) =
        query_as("SELECT role.system,role.role_key FROM sys_user_role user_role INNER JOIN sys_role role ON role.role_id=user_role.role_id")
            .fetch_one(pool)
            .await
            .unwrap();

    assert_eq!(count, 1);
    assert_eq!(binding, (true, "admin".into()));
}
