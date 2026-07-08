use super::*;

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn create_user_rejects_duplicate_username() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.create_user(new_user("alice")).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn replace_user_allows_same_user_identity() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let user = service.replace_user(user_id(1), replace_user("alice", false)).await.unwrap();

    assert_eq!(user.status, "1");
    assert_eq!(repository.replaced_records()[0].1.password_hash.as_deref(), Some("hashed:secret123"));
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn replace_user_rejects_seeded_super_admin_id() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "admin", "hashed:secret123").with_id(super_admin_user_id()));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service.replace_user(super_admin_user_id(), replace_user("admin", false)).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
    assert!(repository.replaced_records().is_empty());
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn delete_user_rejects_seeded_super_admin_id() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "admin", "hashed:secret123").with_id(super_admin_user_id()));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let result = service.delete_user(super_admin_user_id()).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
    assert!(repository.deleted_records().is_empty());
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn list_users_rejects_zero_page() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.list_users(user_filter(0, 10)).await;

    assert!(matches!(result, Err(AppError::InvalidInput(_))));
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn list_users_rejects_page_size_above_maximum() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.list_users(user_filter(1, MAX_PAGE_SIZE + 1)).await;

    assert!(matches!(result, Err(AppError::InvalidInput(_))));
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
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

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
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

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn list_users_filters_by_extra_toolbar_fields() {
    let alice = toolbar_user(ToolbarUserFixture::alice());
    let bob = toolbar_user(ToolbarUserFixture::bob());
    let service = UserService::new(MemoryUserRepository::with_users(vec![alice, bob]), TestPasswordHasher);

    let page = service.list_users(toolbar_filter()).await.unwrap();

    assert_eq!(page.items.into_iter().map(|user| user.username).collect::<Vec<_>>(), vec!["alice"]);
}

#[cfg_attr(miri, ignore = "Miri does not support Tokio runtime I/O on macOS")]
#[tokio::test]
async fn list_users_scoped_applies_extra_toolbar_filters() {
    let alice = toolbar_user(ToolbarUserFixture::alice());
    let bob = toolbar_user(ToolbarUserFixture::bob());
    let service = UserService::new(MemoryUserRepository::with_users(vec![alice, bob]), TestPasswordHasher);

    let page = service
        .list_users_scoped(
            crate::application::UserListFilter {
                role_ids: vec![" 3 ".into()],
                ..user_filter(1, 10)
            },
            DataScopeFilter {
                data_scope: DATA_SCOPE_CUSTOM.into(),
                user_id: user_id(1).0,
                dept_id: Some("101".into()),
                dept_ids: vec!["101".into(), "102".into()],
            },
        )
        .await
        .unwrap();

    assert_eq!(page.items.into_iter().map(|user| user.username).collect::<Vec<_>>(), vec!["bob"]);
}

fn toolbar_filter() -> crate::application::UserListFilter {
    crate::application::UserListFilter {
        username: Some(" ALI ".into()),
        nick_name: Some(" 研发 ".into()),
        email: Some("CORP.EXAMPLE".into()),
        sex: Some(" 0 ".into()),
        dept_name: Some(" 门101 ".into()),
        post_ids: vec![" 2 ".into()],
        role_ids: vec![" 2 ".into()],
        ..user_filter(1, 10)
    }
}

struct ToolbarUserFixture {
    id: u64,
    username: &'static str,
    dept_id: &'static str,
    nick_name: &'static str,
    email: &'static str,
    sex: &'static str,
    post_id: &'static str,
    role_id: &'static str,
}

impl ToolbarUserFixture {
    const fn alice() -> Self {
        Self {
            id: 1,
            username: "alice",
            dept_id: "101",
            nick_name: "Alice研发",
            email: "alice@corp.example",
            sex: "0",
            post_id: "2",
            role_id: "2",
        }
    }

    const fn bob() -> Self {
        Self {
            id: 2,
            username: "bob",
            dept_id: "102",
            nick_name: "Bob运营",
            email: "bob@corp.example",
            sex: "1",
            post_id: "3",
            role_id: "3",
        }
    }
}

fn toolbar_user(input: ToolbarUserFixture) -> crate::test_support::StoredUser {
    stored_user(input.id, input.username, "hashed:secret123")
        .with_dept_id(input.dept_id)
        .with_nick_name(input.nick_name)
        .with_email(input.email)
        .with_sex(input.sex)
        .with_post_ids(vec![input.post_id])
        .with_role_ids(vec![input.role_id])
}
