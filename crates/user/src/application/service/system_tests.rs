use kernel::pagination::PageRequest;

use super::tests::WithPassword;
use crate::{
    application::{AppError, UserService, UserUseCase},
    domain::Credentials,
    test_support::{MemoryUserRepository, TestPasswordHasher, VALID_PASSWORD, new_user, replace_user, stored_user, system_user, user_id},
};

#[tokio::test]
async fn system_user_can_sign_in_by_username() {
    let service = service_with_system_user(MemoryUserRepository::default());

    let user = service
        .sign_in(Credentials {
            identifier: "admin".into(),
            password: VALID_PASSWORD.into(),
        })
        .await
        .unwrap();

    assert_eq!(user.email, "admin@example.com");
    assert!(user.system);
}

#[tokio::test]
async fn system_user_can_sign_in_by_email() {
    let service = service_with_system_user(MemoryUserRepository::default());

    let user = service
        .sign_in(Credentials {
            identifier: "admin@example.com".into(),
            password: VALID_PASSWORD.into(),
        })
        .await
        .unwrap();

    assert_eq!(user.username, "admin");
    assert!(user.system);
}

#[tokio::test]
async fn authenticated_user_returns_system_user_from_provider() {
    let service = service_with_system_user(MemoryUserRepository::default());

    let user = service.authenticated_user(user_id(0)).await.unwrap();

    assert_eq!(user.username, "admin");
    assert!(user.system);
}

#[tokio::test]
async fn create_user_rejects_system_username() {
    let service = service_with_system_user(MemoryUserRepository::default());

    let result = service.create_user(new_user("admin")).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn create_user_rejects_system_email() {
    let service = service_with_system_user(MemoryUserRepository::default());
    let input = new_user("alice").with_email("admin@example.com");

    let result = service.create_user(input).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn replace_user_rejects_system_user_id() {
    let repository = MemoryUserRepository::default();
    let service = service_with_system_user(repository.clone());

    let result = service.replace_user(user_id(0), replace_user("admin", false)).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
    assert!(repository.replaced_records().is_empty());
}

#[tokio::test]
async fn replace_user_rejects_system_username() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = service_with_system_user(repository);

    let result = service.replace_user(user_id(1), replace_user("admin", true)).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn delete_user_rejects_system_user_id() {
    let repository = MemoryUserRepository::default();
    let service = service_with_system_user(repository.clone());

    let result = service.delete_user(user_id(0)).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
    assert!(repository.deleted_records().is_empty());
}

#[tokio::test]
async fn list_users_prepends_system_user_to_first_page() {
    let service = service_with_system_user(users_repository());

    let page = service.list_users(PageRequest { page: 1, page_size: 2 }).await.unwrap();

    let usernames = page.items.iter().map(|user| user.username.as_str()).collect::<Vec<_>>();
    assert_eq!(usernames, vec!["admin", "alice"]);
    assert_eq!(page.total, 4);
    assert_eq!(page.items.len(), 2);
}

#[tokio::test]
async fn list_users_offsets_database_page_after_system_user() {
    let service = service_with_system_user(users_repository());

    let page = service.list_users(PageRequest { page: 2, page_size: 2 }).await.unwrap();

    let usernames = page.items.iter().map(|user| user.username.as_str()).collect::<Vec<_>>();
    assert_eq!(usernames, vec!["bob", "carol"]);
    assert_eq!(page.total, 4);
}

fn service_with_system_user(
    repository: MemoryUserRepository,
) -> UserService<MemoryUserRepository, TestPasswordHasher, crate::test_support::TestSystemUserProvider> {
    UserService::with_system_user(repository, TestPasswordHasher, system_user())
}

fn users_repository() -> MemoryUserRepository {
    MemoryUserRepository::with_users(vec![
        stored_user(1, "alice", "hashed:secret123"),
        stored_user(2, "bob", "hashed:secret123"),
        stored_user(3, "carol", "hashed:secret123"),
    ])
}
