use crate::{
    application::{
        AppError, INSTALLATION_OWNER_PASSWORD_MIN_LENGTH, InstallationOwnerInput, InstallationOwnerUseCase, PasswordPolicy, UserService,
        validate_initial_installation_owner,
    },
    test_support::{MemoryUserRepository, TestPasswordHasher, stored_user, user_id},
};

#[tokio::test]
async fn setup_creates_the_owner_without_business_roles() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let user = service.create_installation_owner(input("root-owner")).await.unwrap();

    assert_eq!(user.username, "root-owner");
    assert!(user.role_ids.is_empty());
    assert!(user.is_installation_owner);
    assert_eq!(repository.installation_owner_id(), Some(user.id));
}

#[tokio::test]
async fn setup_cannot_create_a_second_owner() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "existing-owner", "hashed:secret123"));
    repository.mark_installation_owner(user_id(1));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.create_installation_owner(input("replacement-owner")).await;

    assert!(matches!(result, Err(AppError::Conflict(error)) if error.key() == "errors.user.installation_owner_exists"));
}

#[tokio::test]
async fn setup_owner_password_rejects_fewer_than_eight_characters() {
    let service = UserService::new(MemoryUserRepository::default(), TestPasswordHasher);

    let result = service.create_installation_owner(input_with_password("root-owner", "1234567")).await;

    let Err(AppError::InvalidInput(error)) = result else {
        panic!("a seven-character installation owner password must be rejected");
    };
    assert_eq!(error.key(), "errors.validation.length_between");
    let expected_minimum = INSTALLATION_OWNER_PASSWORD_MIN_LENGTH.to_string();
    let expected_maximum = PasswordPolicy::default().max_length.to_string();
    assert_eq!(
        error.params().iter().map(|parameter| (parameter.key(), parameter.value())).collect::<Vec<_>>(),
        vec![("field", "password"), ("min", expected_minimum.as_str()), ("max", expected_maximum.as_str())]
    );
}

#[tokio::test]
async fn setup_owner_password_accepts_eight_characters() {
    let service = UserService::new(MemoryUserRepository::default(), TestPasswordHasher);

    let user = service.create_installation_owner(input_with_password("root-owner", "12345678")).await.unwrap();

    assert_eq!(user.username, "root-owner");
}

#[tokio::test]
async fn setup_owner_password_cannot_contain_the_username() {
    let service = UserService::new(MemoryUserRepository::default(), TestPasswordHasher);

    let result = service
        .create_installation_owner(input_with_password("Root-Owner", "prefix-root-owner-suffix"))
        .await;

    assert!(matches!(result, Err(AppError::InvalidInput(error)) if error.key() == "errors.user.password_contains_username"));
}

#[tokio::test]
async fn setup_owner_password_is_trimmed_before_hashing() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    service
        .create_installation_owner(input_with_password("root-owner", "  secure-password-123  "))
        .await
        .unwrap();

    assert_eq!(repository.created_records()[0].password_hash.as_deref(), Some("hashed:secure-password-123"));
}

#[test]
fn initial_owner_preflight_validates_username_email_and_password_without_a_repository() {
    let mut invalid_email = input_with_password("root-owner", "secure-password-123");
    invalid_email.email = "not-an-email".into();
    let cases = vec![
        (input_with_password("!owner", "secure-password-123"), "errors.user.username_chars"),
        (invalid_email, "errors.validation.email_format"),
        (input_with_password("root-owner", "1234567"), "errors.validation.length_between"),
        (
            input_with_password("root-owner", "prefix-root-owner-suffix"),
            "errors.user.password_contains_username",
        ),
    ];

    for (input, expected_key) in cases {
        assert!(
            matches!(validate_initial_installation_owner(&input), Err(AppError::InvalidInput(error)) if error.key() == expected_key),
            "expected {expected_key}"
        );
    }
    assert!(validate_initial_installation_owner(&input_with_password("root-owner", "secure-password-123")).is_ok());
}

fn input(username: &str) -> InstallationOwnerInput {
    input_with_password(username, "safe-secret-123")
}

fn input_with_password(username: &str, password: &str) -> InstallationOwnerInput {
    InstallationOwnerInput {
        username: username.into(),
        email: format!("{username}@example.com"),
        password: password.into(),
    }
}
