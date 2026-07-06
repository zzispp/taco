use constants::pagination::MAX_PAGE_SIZE;
use kernel::pagination::PageRequest;

use crate::{
    application::{AppError, UserService, UserUseCase},
    domain::{Credentials, NewUser, ProfileUpdate, UserId},
    test_support::{MemoryUserRepository, TestPasswordHasher, VALID_PASSWORD, new_user, replace_user, stored_user, user_id},
};
use types::rbac::{DATA_SCOPE_CUSTOM, DATA_SCOPE_SELF, DataScopeFilter};

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
async fn profile_returns_user_groups() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

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
    let service = UserService::new(repository, TestPasswordHasher);

    service.change_password(user_id(1), VALID_PASSWORD.into(), "newsecret123".into()).await.unwrap();

    let result = service
        .sign_in(Credentials {
            identifier: "alice".into(),
            password: "newsecret123".into(),
        })
        .await
        .unwrap();
    assert_eq!(result.username, "alice");
}

#[tokio::test]
async fn update_avatar_persists_url() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service.update_avatar(user_id(1), "/uploads/avatars/a.png".into()).await.unwrap();

    assert_eq!(user.avatar.as_deref(), Some("/uploads/avatars/a.png"));
}

#[tokio::test]
async fn create_user_rejects_duplicate_username() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.create_user(new_user("alice")).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn replace_user_allows_same_user_identity() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let user = service.replace_user(user_id(1), replace_user("alice", false)).await.unwrap();

    assert_eq!(user.status, "1");
    assert_eq!(repository.replaced_records()[0].1.password_hash.as_deref(), Some("hashed:secret123"));
}

#[tokio::test]
async fn replace_user_rejects_seeded_super_admin_id() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "admin", "hashed:secret123").with_id(super_admin_user_id()));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service.replace_user(super_admin_user_id(), replace_user("admin", false)).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
    assert!(repository.replaced_records().is_empty());
}

#[tokio::test]
async fn delete_user_rejects_seeded_super_admin_id() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "admin", "hashed:secret123").with_id(super_admin_user_id()));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service.delete_user(super_admin_user_id()).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
    assert!(repository.deleted_records().is_empty());
}

#[tokio::test]
async fn list_users_rejects_zero_page() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.list_users(user_filter(0, 10)).await;

    assert!(matches!(result, Err(AppError::InvalidInput(_))));
}

#[tokio::test]
async fn list_users_rejects_page_size_above_maximum() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.list_users(user_filter(1, MAX_PAGE_SIZE + 1)).await;

    assert!(matches!(result, Err(AppError::InvalidInput(_))));
}

#[tokio::test]
async fn list_users_scoped_supports_self_scope() {
    let repository = MemoryUserRepository::with_users(vec![stored_user(1, "alice", "hashed:secret123"), stored_user(2, "bob", "hashed:secret123")]);
    let service = UserService::new(repository, TestPasswordHasher);

    let page = service
        .list_users_scoped(
            user_filter(1, 10),
            DataScopeFilter {
                data_scope: DATA_SCOPE_SELF.into(),
                user_id: user_id(2).0,
                dept_id: Some("103".into()),
                dept_ids: vec![],
            },
        )
        .await
        .unwrap();

    assert_eq!(page.items.into_iter().map(|user| user.username).collect::<Vec<_>>(), vec!["bob"]);
}

#[tokio::test]
async fn list_users_scoped_supports_custom_departments() {
    let alice = stored_user(1, "alice", "hashed:secret123").with_dept_id("101");
    let bob = stored_user(2, "bob", "hashed:secret123").with_dept_id("102");
    let service = UserService::new(MemoryUserRepository::with_users(vec![alice, bob]), TestPasswordHasher);

    let page = service
        .list_users_scoped(
            user_filter(1, 10),
            DataScopeFilter {
                data_scope: DATA_SCOPE_CUSTOM.into(),
                user_id: user_id(1).0,
                dept_id: Some("101".into()),
                dept_ids: vec!["102".into()],
            },
        )
        .await
        .unwrap();

    assert_eq!(page.items.into_iter().map(|user| user.username).collect::<Vec<_>>(), vec!["bob"]);
}

pub(super) trait WithPassword {
    fn with_password(self, password: &str) -> Self;
    fn with_email(self, email: &str) -> Self;
}

impl WithPassword for NewUser {
    fn with_password(self, password: &str) -> Self {
        Self {
            password: password.into(),
            ..self
        }
    }

    fn with_email(self, email: &str) -> Self {
        Self { email: email.into(), ..self }
    }
}

fn user_filter(page: u64, page_size: u64) -> crate::application::UserListFilter {
    crate::application::UserListFilter {
        page: PageRequest { page, page_size },
        username: None,
        phonenumber: None,
        status: None,
        dept_id: None,
        begin_time: None,
        end_time: None,
    }
}

fn profile_update(email: &str, phonenumber: Option<&str>) -> ProfileUpdate {
    ProfileUpdate {
        nick_name: "Alice".into(),
        phonenumber: phonenumber.map(str::to_owned),
        email: email.into(),
        sex: "2".into(),
    }
}

fn super_admin_user_id() -> UserId {
    UserId(constants::system::SUPER_ADMIN_USER_ID.into())
}
