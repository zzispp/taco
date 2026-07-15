use super::*;

#[tokio::test]
async fn create_user_rejects_duplicate_username() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.create_user(new_user("alice")).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn create_user_normalizes_email_and_allows_no_roles() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository.clone(), TestPasswordHasher);
    let mut input = new_user("alice");
    input.email = " Alice@Example.COM ".into();
    input.role_ids.clear();

    let created = service.create_user(input).await.unwrap();

    assert_eq!(created.email, "alice@example.com");
    assert_eq!(created.role_ids, Vec::<String>::new());
    assert_eq!(repository.created_records()[0].email, "alice@example.com");
}

#[tokio::test]
async fn create_user_rejects_invalid_email_and_phone() {
    let invalid_email = UserService::new(MemoryUserRepository::default(), TestPasswordHasher)
        .create_user(new_user("alice").with_email("invalid"))
        .await;
    let mut invalid_phone_input = new_user("bob");
    invalid_phone_input.phonenumber = Some("123".into());
    let invalid_phone = UserService::new(MemoryUserRepository::default(), TestPasswordHasher)
        .create_user(invalid_phone_input)
        .await;

    assert!(matches!(invalid_email, Err(AppError::InvalidInput(error)) if error.key() == "errors.validation.email_format"));
    assert!(matches!(invalid_phone, Err(AppError::InvalidInput(error)) if error.key() == "errors.validation.phone_format"));
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
async fn replace_user_allows_an_explicit_empty_role_set() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository.clone(), TestPasswordHasher);
    let mut input = replace_user("alice", false);
    input.role_ids.clear();

    let user = service.replace_user(user_id(1), input).await.unwrap();

    assert_eq!(user.role_ids, Vec::<String>::new());
    assert_eq!(repository.replaced_records()[0].1.role_ids, Vec::<String>::new());
}

#[tokio::test]
async fn independent_role_assignment_allows_an_explicit_empty_set() {
    let repository = MemoryUserRepository::with_user(stored_user(1, "alice", "hashed:secret123"));
    let service = UserService::new(repository, TestPasswordHasher);

    let user = service.replace_roles(user_id(1), Vec::new()).await.unwrap();

    assert_eq!(user.role_ids, Vec::<String>::new());
    assert_eq!(user.roles, Vec::<types::rbac::RoleSummary>::new());
}

#[tokio::test]
async fn replace_user_allows_fixed_legacy_id() {
    let legacy_id = crate::domain::UserId("1".into());
    let repository = MemoryUserRepository::with_user(stored_user(1, "legacy", "hashed:secret123").with_id(legacy_id.clone()));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    let user = service.replace_user(legacy_id, replace_user("legacy", false)).await.unwrap();

    assert_eq!(user.status, "1");
    assert_eq!(repository.replaced_records().len(), 1);
}

#[tokio::test]
async fn delete_user_allows_fixed_legacy_id() {
    let legacy_id = crate::domain::UserId("1".into());
    let repository = MemoryUserRepository::with_user(stored_user(1, "legacy", "hashed:secret123").with_id(legacy_id.clone()));
    let service = UserService::new(repository.clone(), TestPasswordHasher);

    service.delete_user(legacy_id.clone()).await.unwrap();

    assert_eq!(repository.deleted_records(), vec![legacy_id]);
}

#[tokio::test]
async fn list_users_rejects_zero_limit() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.list_users(user_filter(0)).await;

    assert!(matches!(result, Err(AppError::InvalidInput(message)) if message.key() == "errors.validation.cursor_limit_range"));
}

#[tokio::test]
async fn list_users_rejects_limit_above_maximum() {
    let repository = MemoryUserRepository::default();
    let service = UserService::new(repository, TestPasswordHasher);

    let result = service.list_users(user_filter(MAX_CURSOR_LIMIT + 1)).await;

    assert!(matches!(result, Err(AppError::InvalidInput(message)) if message.key() == "errors.validation.cursor_limit_range"));
}

#[tokio::test]
async fn list_users_scoped_supports_self_scope() {
    let repository = MemoryUserRepository::with_users(vec![stored_user(1, "alice", "hashed:secret123"), stored_user(2, "bob", "hashed:secret123")]);
    let service = UserService::new(repository, TestPasswordHasher);

    let page = service
        .list_users_scoped(
            user_filter(10),
            DataScopeFilter {
                data_scope: DataScope::SelfOnly,
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
            user_filter(10),
            DataScopeFilter {
                data_scope: DataScope::Custom,
                user_id: user_id(1).0,
                dept_id: Some("101".into()),
                dept_ids: vec!["102".into()],
            },
        )
        .await
        .unwrap();

    assert_eq!(page.items.into_iter().map(|user| user.username).collect::<Vec<_>>(), vec!["bob"]);
}

#[tokio::test]
async fn list_users_filters_by_extra_toolbar_fields() {
    let alice = toolbar_user(ToolbarUserFixture::alice());
    let bob = toolbar_user(ToolbarUserFixture::bob());
    let service = UserService::new(MemoryUserRepository::with_users(vec![alice, bob]), TestPasswordHasher);

    let page = service.list_users(toolbar_filter()).await.unwrap();

    assert_eq!(page.items.into_iter().map(|user| user.username).collect::<Vec<_>>(), vec!["alice"]);
}

#[tokio::test]
async fn list_users_scoped_applies_extra_toolbar_filters() {
    let alice = toolbar_user(ToolbarUserFixture::alice());
    let bob = toolbar_user(ToolbarUserFixture::bob());
    let service = UserService::new(MemoryUserRepository::with_users(vec![alice, bob]), TestPasswordHasher);

    let page = service
        .list_users_scoped(
            crate::application::UserListFilter {
                role_ids: vec![" 3 ".into()],
                ..user_filter(10)
            },
            DataScopeFilter {
                data_scope: DataScope::Custom,
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
        ..user_filter(10)
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
