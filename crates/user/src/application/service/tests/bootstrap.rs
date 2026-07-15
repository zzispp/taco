use super::*;
use crate::application::BootstrapAdminInput;

#[tokio::test]
async fn bootstrap_admin_creates_the_only_super_admin_with_the_admin_role() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let user = service.bootstrap_admin(bootstrap_input("root-admin")).await.unwrap();

    assert_eq!(user.username, "root-admin");
    assert_eq!(user.role_ids, vec!["1"]);
    assert_eq!(user.status, "0");
    assert_eq!(repository.created_records()[0].password_hash.as_deref(), Some("hashed:safe-secret-123"));
}

#[tokio::test]
async fn bootstrap_admin_allows_an_existing_ordinary_user() {
    let ordinary = stored_user(1, "alice", "hashed:secret123")
        .with_id(crate::domain::UserId("1".into()))
        .with_role_ids(vec!["2"]);
    let repository = MemoryUserRepository::with_user(ordinary);
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service.bootstrap_admin(bootstrap_input("root-admin")).await;

    assert_eq!(result.unwrap().username, "root-admin");
    assert_eq!(repository.created_records().len(), 1);
}

#[tokio::test]
async fn bootstrap_admin_rejects_an_existing_active_admin_role_binding() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "existing-admin", "hashed:secret123"));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service.bootstrap_admin(bootstrap_input("root-admin")).await;

    assert!(matches!(result, Err(AppError::Conflict(error)) if error.key() == "errors.user.bootstrap_admin_exists"));
    assert_eq!(repository.created_records(), Vec::new());
}

#[tokio::test]
async fn bootstrap_admin_rejects_a_disabled_admin_role_binding() {
    let existing_admin = stored_user(1, "disabled-admin", "hashed:secret123").with_status("1");
    let repository = MemoryUserRepository::with_user(existing_admin);
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service.bootstrap_admin(bootstrap_input("root-admin")).await;

    assert!(matches!(result, Err(AppError::Conflict(error)) if error.key() == "errors.user.bootstrap_admin_exists"));
    assert_eq!(repository.created_records(), Vec::new());
}

fn bootstrap_input(username: &str) -> BootstrapAdminInput {
    BootstrapAdminInput {
        username: username.into(),
        email: format!("{username}@example.com"),
        password: "safe-secret-123".into(),
    }
}
