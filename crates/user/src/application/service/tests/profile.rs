use super::*;

#[tokio::test]
async fn profile_returns_user_groups() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = user_service_with_login_security(repository);

    let profile = service.profile(user_id(1)).await.unwrap();

    assert_eq!(profile.user.username, "alice");
    assert_eq!(profile.role_group, "超级管理员");
    assert_eq!(profile.dept_name.as_deref(), Some("部门103"));
}

#[tokio::test]
async fn update_profile_rejects_duplicate_email() {
    let users = vec![stored_user(1, "alice", "hashed:secret123"), stored_user(2, "bob", "hashed:secret123")];
    let service = UserService::new(MemoryUserRepository::with_users(users), TestPasswordHasher);

    let result = service.update_profile(user_id(1), profile_update("bob@example.com", None)).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn update_profile_rejects_invalid_email_format() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.update_profile(user_id(1), profile_update("alice.example.com", None)).await;

    assert!(matches!(
        result,
        Err(AppError::InvalidInput(error)) if error.key() == "errors.validation.email_format"
    ));
}

#[tokio::test]
async fn update_profile_rejects_invalid_phone_format() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.update_profile(user_id(1), profile_update("alice-new@example.com", Some("12345"))).await;

    assert!(matches!(
        result,
        Err(AppError::InvalidInput(error)) if error.key() == "errors.validation.phone_format"
    ));
}

#[tokio::test]
async fn update_profile_changes_allowed_fields() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service
        .update_profile(user_id(1), profile_update("alice-new@example.com", Some("13900000000")))
        .await
        .unwrap();

    assert_eq!(user.email, "alice-new@example.com");
    assert_eq!(user.phonenumber.as_deref(), Some("13900000000"));
}

#[tokio::test]
async fn change_password_rejects_wrong_old_password() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.change_password(user_id(1), "wrong-password".into(), "newsecret123".into()).await;

    assert!(matches!(result, Err(AppError::Unauthorized)));
}

#[tokio::test]
async fn change_password_rejects_same_password() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.change_password(user_id(1), VALID_PASSWORD.into(), VALID_PASSWORD.into()).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn change_password_updates_password_hash() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = user_service_with_login_security(repository);

    service.change_password(user_id(1), VALID_PASSWORD.into(), "newsecret123".into()).await.unwrap();

    let result = service
        .sign_in(Credentials {
            identifier: "alice".into(),
            password: "newsecret123".into(),
        })
        .await
        .unwrap();
    assert_eq!(result.user().username, "alice");
}

#[tokio::test]
async fn update_avatar_persists_url() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service.update_avatar(user_id(1), "/uploads/avatars/a.png".into()).await.unwrap();

    assert_eq!(user.avatar.as_deref(), Some("/uploads/avatars/a.png"));
}
