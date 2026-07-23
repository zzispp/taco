use crate::{
    application::{AppError, BootstrapAdministratorInput, BootstrapAdministratorOutcome, UserRepository, UserService},
    test_support::{MemoryUserRepository, NON_SYSTEM_ADMIN_ROLE_ID, SYSTEM_ADMIN_ROLE_ID, TestPasswordHasher, stored_user},
};

const BOOTSTRAP_PASSWORD: &str = "secure-bootstrap-password-123";

#[tokio::test]
async fn bootstrap_creates_an_enabled_system_administrator_once() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let outcome = service.bootstrap_administrator(valid_input()).await.unwrap();

    assert_eq!(outcome, BootstrapAdministratorOutcome::Created);
    assert!(service.has_enabled_system_administrator().await.unwrap());
    let records = repository.created_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].username, "bootstrap-admin");
    assert_eq!(records[0].email, "bootstrap-admin@example.test");
    assert_eq!(records[0].password_hash.as_deref(), Some("hashed:secure-bootstrap-password-123"));
    assert_eq!(records[0].role_ids, vec![SYSTEM_ADMIN_ROLE_ID]);
}

#[tokio::test]
async fn bootstrap_keeps_an_existing_system_administrator_unchanged() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "existing-admin", "hashed:existing-password").with_role_ids(vec![SYSTEM_ADMIN_ROLE_ID]));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let outcome = service
        .bootstrap_administrator(BootstrapAdministratorInput {
            username: "!invalid!".into(),
            email: "invalid-email".into(),
            password: "short".into(),
        })
        .await
        .unwrap();

    assert_eq!(outcome, BootstrapAdministratorOutcome::AlreadyPresent);
    assert!(repository.created_records().is_empty());
    let existing = repository.find_auth_by_username("existing-admin").await.unwrap().unwrap();
    assert_eq!(existing.password_hash, "hashed:existing-password");
}

#[tokio::test]
async fn bootstrap_does_not_treat_a_non_system_admin_role_as_the_protected_administrator() {
    let repository =
        MemoryUserRepository::with_user(stored_user(1, "non-system-admin", "hashed:existing-password").with_role_ids(vec![NON_SYSTEM_ADMIN_ROLE_ID]));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let outcome = service.bootstrap_administrator(valid_input()).await.unwrap();

    assert_eq!(outcome, BootstrapAdministratorOutcome::Created);
    assert!(service.has_enabled_system_administrator().await.unwrap());
    assert_eq!(repository.created_records().len(), 1);
}

#[tokio::test]
async fn bootstrap_validates_input_before_creating_the_administrator() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service
        .bootstrap_administrator(BootstrapAdministratorInput {
            username: "bootstrap-admin".into(),
            email: "bootstrap-admin@example.test".into(),
            password: "short".into(),
        })
        .await;

    assert!(matches!(result, Err(AppError::InvalidInput(_))));
    assert!(repository.created_records().is_empty());
    assert!(!service.has_enabled_system_administrator().await.unwrap());
}

#[tokio::test]
async fn bootstrap_rejects_an_input_that_conflicts_with_an_existing_user() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "bootstrap-admin", "hashed:existing-password"));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service.bootstrap_administrator(valid_input()).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
    assert!(repository.created_records().is_empty());
    assert!(!service.has_enabled_system_administrator().await.unwrap());
}

fn valid_input() -> BootstrapAdministratorInput {
    BootstrapAdministratorInput {
        username: "bootstrap-admin".into(),
        email: "bootstrap-admin@example.test".into(),
        password: BOOTSTRAP_PASSWORD.into(),
    }
}
