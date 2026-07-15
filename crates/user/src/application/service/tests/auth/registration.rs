use super::super::*;

#[tokio::test]
async fn sign_up_hashes_password_and_persists_user() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let user = service.sign_up(new_user("alice")).await.unwrap();
    let created = repository.created_records();

    assert_eq!(user.username, "alice");
    assert_eq!(created[0].password_hash.as_deref(), Some("hashed:secret123"));
}

#[tokio::test]
async fn sign_up_trims_username_email_and_password_before_persisting() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);
    let input = new_user("  alice  ").with_email("  alice@example.com  ").with_password("  secret123  ");

    let user = service.sign_up(input).await.unwrap();
    let created = repository.created_records();

    assert_eq!(user.username, "alice");
    assert_eq!(created[0].username, "alice");
    assert_eq!(created[0].email, "alice@example.com");
    assert_eq!(created[0].password_hash.as_deref(), Some("hashed:secret123"));
}

#[tokio::test]
async fn sign_up_rejects_invalid_username_constraints() {
    for username in ["ab", "alice!", "-alice", "alice_"] {
        let repository = MemoryUserRepository::default();
        let service = UserService::new(repository, TestPasswordHasher);

        let result = service.sign_up(new_user(username)).await;

        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }
}

#[tokio::test]
async fn sign_up_rejects_invalid_password_constraints() {
    for password in ["short", ""] {
        let repository = MemoryUserRepository::default();
        let service = UserService::new(repository, TestPasswordHasher);

        let result = service.sign_up(new_user("alice").with_password(password)).await;

        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }
}
