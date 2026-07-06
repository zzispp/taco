mod support;

use system::application::{PostListFilter, SystemError, SystemService, SystemUseCase};

use support::{MemoryRepository, config_item, dept, dict_type, page, post_input};

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
    let repository = MemoryRepository::default().with_config("sys.user.initPassword", "12345678");
    let service = SystemService::new(repository);

    let value = service.config_by_key("sys.user.initPassword").await.unwrap();
    let missing = service.config_by_key("missing.key").await;

    assert_eq!(value, "12345678");
    assert!(matches!(missing, Err(SystemError::NotFound)));
}

#[tokio::test]
async fn public_configs_return_only_public_values() {
    let repository = MemoryRepository::default()
        .with_config_item(config_item("sys.index.skinName", "skin-blue", "Y", true))
        .with_config_item(config_item("sys.index.modeTheme", "theme-light", "Y", true))
        .with_config_item(config_item("sys.account.captchaEnabled", "true", "Y", true));
    let service = SystemService::new(repository);

    let values = service
        .public_configs(vec![
            "sys.index.skinName".into(),
            "sys.index.modeTheme".into(),
            "sys.account.captchaEnabled".into(),
        ])
        .await
        .unwrap();

    assert_eq!(values.get("sys.index.skinName").map(String::as_str), Some("skin-blue"));
    assert_eq!(values.get("sys.index.modeTheme").map(String::as_str), Some("theme-light"));
    assert_eq!(values.get("sys.account.captchaEnabled").map(String::as_str), Some("true"));
}

#[tokio::test]
async fn public_configs_reject_private_or_missing_keys() {
    let repository = MemoryRepository::default().with_config("sys.user.initPassword", "12345678");
    let service = SystemService::new(repository);

    let private_result = service.public_configs(vec!["sys.user.initPassword".into()]).await;
    let missing_result = service.public_configs(vec!["missing.key".into()]).await;

    assert!(matches!(private_result, Err(SystemError::Forbidden(_))));
    assert!(matches!(missing_result, Err(SystemError::NotFound)));
}

#[tokio::test]
async fn built_in_config_cannot_be_deleted_or_renamed() {
    let repository = MemoryRepository::default().with_config_item(config_item("sys.index.skinName", "skin-blue", "Y", true));
    let service = SystemService::new(repository);

    let delete_result = service.delete_config("sys.index.skinName").await;
    let replace_result = service
        .replace_config("sys.index.skinName", config_input("sys.index.modeTheme", "theme-dark", "Y", true))
        .await;

    assert!(matches!(delete_result, Err(SystemError::Conflict(message)) if message == "built-in config cannot be deleted"));
    assert!(matches!(replace_result, Err(SystemError::Conflict(message)) if message == "built-in config key cannot be changed"));
}

#[tokio::test]
async fn initial_password_config_cannot_be_public() {
    let service = SystemService::new(MemoryRepository::default());

    let result = service.create_config(config_input("sys.user.initPassword", "12345678", "Y", true)).await;

    assert!(matches!(result, Err(SystemError::Conflict(message)) if message == "initial password config cannot be public"));
}

#[tokio::test]
async fn captcha_private_config_cannot_be_public() {
    let service = SystemService::new(MemoryRepository::default());

    let result = service.create_config(config_input("sys.account.captchaPrivateConfig", "{}", "Y", true)).await;

    assert!(matches!(result, Err(SystemError::Conflict(message)) if message == "captcha private config cannot be public"));
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

fn config_input(key: &str, value: &str, config_type: &str, public_read: bool) -> system::domain::ConfigInput {
    system::domain::ConfigInput {
        config_name: key.into(),
        config_key: key.into(),
        config_value: value.into(),
        config_type: config_type.into(),
        public_read,
        remark: None,
    }
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
