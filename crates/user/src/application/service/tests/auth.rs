use super::*;

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn sign_up_hashes_password_and_persists_user() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let user = service.sign_up(new_user("alice")).await.unwrap();
    let created = repository.created_records();

    assert_eq!(user.username, "alice");
    assert_eq!(created[0].password_hash.as_deref(), Some("hashed:secret123"));
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
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

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn sign_in_rejects_invalid_password() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service
        .sign_in(Credentials {
            identifier: "alice".into(),
            password: "bad-password".into(),
        })
        .await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn sign_in_accepts_email_identifier() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let user = service
        .sign_in(Credentials {
            identifier: "alice@example.com".into(),
            password: VALID_PASSWORD.into(),
        })
        .await
        .unwrap();

    assert_eq!(user.username, "alice");
    assert_eq!(repository.login_records(), vec![user_id(1)]);
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn sign_in_trims_identifier_and_password() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service
        .sign_in(Credentials {
            identifier: "  alice  ".into(),
            password: "  secret123  ".into(),
        })
        .await
        .unwrap();

    assert_eq!(user.email, "alice@example.com");
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn sign_up_rejects_invalid_username_constraints() {
    for username in ["ab", "alice!", "-alice", "alice_"] {
        let repository = MemoryUserRepository::default();
        let service = UserService::new(repository, TestPasswordHasher);

        let result = service.sign_up(new_user(username)).await;

        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn sign_up_rejects_invalid_password_constraints() {
    for password in ["short", ""] {
        let repository = MemoryUserRepository::default();
        let service = UserService::new(repository, TestPasswordHasher);

        let result = service.sign_up(new_user("alice").with_password(password)).await;

        assert!(matches!(result, Err(AppError::InvalidInput(_))));
    }
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn authenticated_user_returns_user_from_token_subject() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service.authenticated_user(user_id(1)).await.unwrap();

    assert_eq!(user.email, "alice@example.com");
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn authenticated_user_rejects_unknown_user() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.authenticated_user(user_id(1)).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}
