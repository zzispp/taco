use super::super::*;

#[tokio::test]
async fn authenticated_user_returns_user_from_token_subject() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service.authenticated_user(user_id(1)).await.unwrap();

    assert_eq!(user.email, "alice@example.com");
}

#[tokio::test]
async fn authenticated_user_rejects_unknown_user() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.authenticated_user(user_id(1)).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn authenticated_user_rejects_disabled_user() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123").with_status("1"));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.authenticated_user(user_id(1)).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}
