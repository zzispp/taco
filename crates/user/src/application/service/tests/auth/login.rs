use super::super::*;

#[tokio::test]
async fn sign_in_rejects_invalid_password() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let failures = MemoryLoginFailureStore::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher).with_login_security(failures.clone(), TestLoginLockConfigProvider::default());

    let result = service
        .sign_in(Credentials {
            identifier: "alice".into(),
            password: "bad-password".into(),
        })
        .await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
    assert_eq!(failures.count(&user_id(1)), 1);
    assert_eq!(failures.ttl_seconds(&user_id(1)), Some(600));
}

#[tokio::test]
async fn sign_in_accepts_email_identifier() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = user_service_with_login_security(repository.clone());

    let login = service
        .sign_in(Credentials {
            identifier: "alice@example.com".into(),
            password: VALID_PASSWORD.into(),
        })
        .await
        .unwrap();

    assert!(repository.login_records().is_empty());
    let user = service.complete_sign_in(login, "203.0.113.9".into()).await.unwrap();
    assert_eq!(user.username, "alice");
    assert_eq!(repository.login_records(), vec![user_id(1)]);
    assert_eq!(repository.login_ip_records(), vec![(user_id(1), "203.0.113.9".into())]);
}

#[tokio::test]
async fn sign_in_trims_identifier_and_password() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = user_service_with_login_security(repository);

    let login = service
        .sign_in(Credentials {
            identifier: "  alice  ".into(),
            password: "  secret123  ".into(),
        })
        .await
        .unwrap();

    assert_eq!(login.user().email, "alice@example.com");
}
