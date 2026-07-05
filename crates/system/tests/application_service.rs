mod support;

use system::application::{PostListFilter, SystemError, SystemService, SystemUseCase};

use support::{MemoryRepository, dept, dict_type, page, post_input};

#[tokio::test]
async fn delete_dict_type_rejects_existing_data() {
    let repository = MemoryRepository::default().with_dict_type(dict_type("1", "sys_user_sex")).with_dict_data(true);
    let service = SystemService::new(repository.clone());

    let result = service.delete_dict_type("1").await;

    assert!(matches!(result, Err(SystemError::Conflict(message)) if message == "dictionary type still has data"));
    assert_eq!(repository.deleted_dict_types(), Vec::<String>::new());
}

#[tokio::test]
async fn config_by_key_returns_value_or_not_found() {
    let repository = MemoryRepository::default().with_config("sys.user.initPassword", "123456");
    let service = SystemService::new(repository);

    let value = service.config_by_key("sys.user.initPassword").await.unwrap();
    let missing = service.config_by_key("missing.key").await;

    assert_eq!(value, "123456");
    assert!(matches!(missing, Err(SystemError::NotFound)));
}

#[tokio::test]
async fn create_post_rejects_duplicate_code_and_name() {
    let code_repository = MemoryRepository::default().with_duplicate_post_code(true);
    let name_repository = MemoryRepository::default().with_duplicate_post_name(true);

    let code_result = SystemService::new(code_repository).create_post(post_input("ceo", "董事长")).await;
    let name_result = SystemService::new(name_repository).create_post(post_input("cto", "董事长")).await;

    assert!(matches!(code_result, Err(SystemError::Conflict(message)) if message == "post code already exists"));
    assert!(matches!(name_result, Err(SystemError::Conflict(message)) if message == "post name already exists"));
}

#[tokio::test]
async fn delete_dept_rejects_children_or_users() {
    let children_result = SystemService::new(MemoryRepository::default().with_dept_children(true))
        .delete_dept("103")
        .await;
    let users_result = SystemService::new(MemoryRepository::default().with_dept_users(true)).delete_dept("103").await;

    assert!(matches!(children_result, Err(SystemError::Conflict(message)) if message == "department still has children or users"));
    assert!(matches!(users_result, Err(SystemError::Conflict(message)) if message == "department still has children or users"));
}

#[tokio::test]
async fn page_post_filter_is_trimmed_and_empty_values_are_removed() {
    let repository = MemoryRepository::default();
    let filter = PostListFilter {
        page: page(),
        post_code: Some(" ceo ".into()),
        post_name: Some("   ".into()),
        status: Some(" 0 ".into()),
    };

    SystemService::new(repository.clone()).page_posts(filter).await.unwrap();

    assert_eq!(
        repository.last_post_filter(),
        Some(PostListFilter {
            page: page(),
            post_code: Some("ceo".into()),
            post_name: None,
            status: Some("0".into())
        })
    );
}

#[tokio::test]
async fn update_dept_sort_forwards_requested_order() {
    let repository = MemoryRepository::default().with_dept(dept("103", "100", "研发部门"));
    let service = SystemService::new(repository.clone());

    let updated = service.update_dept_sort("103", 7).await.unwrap();

    assert_eq!(updated.order_num, 7);
    assert_eq!(repository.updated_dept_sorts(), vec![("103".into(), 7)]);
}
