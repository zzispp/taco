use kernel::pagination::PageRequest;

use super::tests::WithPassword;
use crate::{
    application::service::StaticPasswordPolicyProvider,
    application::{AppError, UserService, UserUseCase},
    domain::Credentials,
    test_support::{MemoryUserRepository, TestPasswordHasher, VALID_PASSWORD, new_user, replace_user, stored_user, system_user, user_id},
};

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
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

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
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

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn authenticated_user_returns_system_user_from_provider() {
    let service = service_with_system_user(MemoryUserRepository::default());

    let user = service.authenticated_user(user_id(0)).await.unwrap();

    assert_eq!(user.username, "admin");
    assert!(user.system);
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn create_user_rejects_system_username() {
    let service = service_with_system_user(MemoryUserRepository::default());

    let result = service.create_user(new_user("admin")).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn create_user_rejects_system_email() {
    let service = service_with_system_user(MemoryUserRepository::default());
    let input = new_user("alice").with_email("admin@example.com");

    let result = service.create_user(input).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn replace_user_rejects_system_user_id() {
    let repository = MemoryUserRepository::default();
    let service = service_with_system_user(repository.clone());

    let result = service.replace_user(user_id(0), replace_user("admin", false)).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
    assert!(repository.replaced_records().is_empty());
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn replace_user_rejects_system_username() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = service_with_system_user(repository);

    let result = service.replace_user(user_id(1), replace_user("admin", true)).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn delete_user_rejects_system_user_id() {
    let repository = MemoryUserRepository::default();
    let service = service_with_system_user(repository.clone());

    let result = service.delete_user(user_id(0)).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
    assert!(repository.deleted_records().is_empty());
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn list_users_prepends_system_user_to_first_page() {
    let service = service_with_system_user(users_repository());

    let page = service.list_users(user_filter(1, 2)).await.unwrap();

    let usernames = page.items.iter().map(|user| user.username.as_str()).collect::<Vec<_>>();
    assert_eq!(usernames, vec!["admin", "alice"]);
    assert_eq!(page.total, 4);
    assert_eq!(page.items.len(), 2);
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn list_users_filters_system_user_case_insensitively() {
    let service = service_with_system_user(MemoryUserRepository::default());

    let page = service
        .list_users(crate::application::UserListFilter {
            username: Some("ADM".into()),
            ..user_filter(1, 10)
        })
        .await
        .unwrap();

    assert_eq!(page.items.into_iter().map(|user| user.username).collect::<Vec<_>>(), vec!["admin"]);
    assert_eq!(page.total, 1);
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn list_users_offsets_database_page_after_system_user() {
    let service = service_with_system_user(users_repository());

    let page = service.list_users(user_filter(2, 2)).await.unwrap();

    let usernames = page.items.iter().map(|user| user.username.as_str()).collect::<Vec<_>>();
    assert_eq!(usernames, vec!["bob", "carol"]);
    assert_eq!(page.total, 4);
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn list_users_applies_dept_filter_when_system_user_is_configured() {
    let repository = MemoryUserRepository::with_users(vec![
        stored_user(1, "alice", "hashed:secret123").with_dept_id("103"),
        stored_user(2, "bob", "hashed:secret123").with_dept_id("105"),
    ]);
    let service = service_with_system_user(repository);

    let page = service.list_users(user_filter_by_dept("105")).await.unwrap();

    let usernames = page.items.iter().map(|user| user.username.as_str()).collect::<Vec<_>>();
    assert_eq!(usernames, vec!["bob"]);
    assert_eq!(page.total, 1);
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn list_users_returns_empty_for_unmatched_dept_filter_when_system_user_is_configured() {
    let service = service_with_system_user(users_repository());

    let page = service.list_users(user_filter_by_dept("106")).await.unwrap();

    assert!(page.items.is_empty());
    assert_eq!(page.total, 0);
}

fn service_with_system_user(
    repository: MemoryUserRepository,
) -> UserService<MemoryUserRepository, TestPasswordHasher, StaticPasswordPolicyProvider, crate::test_support::TestSystemUserProvider> {
    UserService {
        repository,
        password_hasher: TestPasswordHasher,
        password_policy: StaticPasswordPolicyProvider,
        system_users: system_user(),
    }
}

fn users_repository() -> MemoryUserRepository {
    MemoryUserRepository::with_users(vec![
        stored_user(1, "alice", "hashed:secret123"),
        stored_user(2, "bob", "hashed:secret123"),
        stored_user(3, "carol", "hashed:secret123"),
    ])
}

fn user_filter(page: u64, page_size: u64) -> crate::application::UserListFilter {
    crate::application::UserListFilter {
        page: PageRequest { page, page_size },
        username: None,
        nick_name: None,
        phonenumber: None,
        email: None,
        sex: None,
        status: None,
        dept_id: None,
        dept_name: None,
        post_ids: vec![],
        role_ids: vec![],
        begin_time: None,
        end_time: None,
    }
}

fn user_filter_by_dept(dept_id: &str) -> crate::application::UserListFilter {
    crate::application::UserListFilter {
        dept_id: Some(dept_id.into()),
        ..user_filter(1, 10)
    }
}
