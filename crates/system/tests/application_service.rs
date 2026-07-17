#[allow(dead_code)]
mod support;

use rbac::domain::{DataScope, DataScopeFilter};
use system::application::{DeptListFilter, PostListFilter, SystemError, SystemService, SystemUseCase};
use types::http::{DateTimeRange, parse_date_time_range};

use support::{ConfigInputSeed, MemoryRepository, config_input, dept, dict_type, page, post_input, public_config_item};

fn created_time_range() -> DateTimeRange {
    parse_date_time_range(Some("2026-07-01"), Some("2026-07-08")).unwrap()
}

#[tokio::test]
async fn delete_dict_type_rejects_existing_data() {
    let repository = MemoryRepository::default().with_dict_type(dict_type("1", "sys_user_sex")).with_dict_data(true);
    let service = SystemService::new(repository.clone());

    let result = service.delete_dict_type("1").await;

    assert!(matches!(result, Err(SystemError::Conflict(message)) if message.key() == "errors.system.dict_type_has_data"));
    assert_eq!(repository.deleted_dict_types(), Vec::<String>::new());
}

#[tokio::test]
async fn config_by_key_returns_value_or_not_found() {
    let repository = MemoryRepository::default().with_config("sys.index.modeTheme", "theme-light");
    let service = SystemService::new(repository);

    let value = service.config_by_key("sys.index.modeTheme").await.unwrap();
    let missing = service.config_by_key("missing.key").await;

    assert_eq!(value, "theme-light");
    assert!(matches!(missing, Err(SystemError::NotFound)));
}

#[tokio::test]
async fn public_configs_return_only_public_values() {
    let repository = MemoryRepository::default()
        .with_config_item(public_config_item("sys.index.skinName", "skin-blue"))
        .with_config_item(public_config_item("sys.index.modeTheme", "theme-light"))
        .with_config_item(public_config_item("sys.site.displayConfig", r#"{"site_name":"taco"}"#));
    let service = SystemService::new(repository);

    let values = service
        .public_configs(vec!["sys.index.skinName".into(), "sys.index.modeTheme".into(), "sys.site.displayConfig".into()])
        .await
        .unwrap();

    assert_eq!(values.get("sys.index.skinName").map(String::as_str), Some("skin-blue"));
    assert_eq!(values.get("sys.index.modeTheme").map(String::as_str), Some("theme-light"));
    assert_eq!(values.get("sys.site.displayConfig").map(String::as_str), Some(r#"{"site_name":"taco"}"#));
}

#[tokio::test]
async fn public_configs_reject_private_or_missing_keys() {
    let repository = MemoryRepository::default().with_config("sys.account.captchaConfig", "{}");
    let service = SystemService::new(repository);

    let private_result = service.public_configs(vec!["sys.account.captchaConfig".into()]).await;
    let missing_result = service.public_configs(vec!["missing.key".into()]).await;

    assert!(matches!(private_result, Err(SystemError::Forbidden(_))));
    assert!(matches!(missing_result, Err(SystemError::NotFound)));
}

#[tokio::test]
async fn built_in_config_cannot_be_deleted_or_renamed() {
    let repository = MemoryRepository::default().with_config_item(public_config_item("sys.index.skinName", "skin-blue"));
    let service = SystemService::new(repository);

    let delete_result = service.delete_config("sys.index.skinName").await;
    let replace_result = service
        .replace_config("sys.index.skinName", config_input(ConfigInputSeed::public("sys.index.modeTheme", "theme-dark")))
        .await;

    assert!(matches!(delete_result, Err(SystemError::Conflict(message)) if message.key() == "errors.system.builtin_config_delete"));
    assert!(matches!(replace_result, Err(SystemError::Conflict(message)) if message.key() == "errors.system.builtin_config_key_change"));
}

#[tokio::test]
async fn captcha_config_cannot_be_public() {
    let service = SystemService::new(MemoryRepository::default());

    let result = service.create_config(public_config_input("sys.account.captchaConfig", "{}")).await;

    assert!(matches!(result, Err(SystemError::Conflict(message)) if message.key() == "errors.system.sensitive_config_private"));
}

#[tokio::test]
async fn create_post_rejects_duplicate_code_and_name() {
    let code_repository = MemoryRepository::default().with_duplicate_post_code(true);
    let name_repository = MemoryRepository::default().with_duplicate_post_name(true);

    let code_result = SystemService::new(code_repository).create_post(post_input("ceo", "董事长")).await;
    let name_result = SystemService::new(name_repository).create_post(post_input("cto", "董事长")).await;

    assert!(matches!(code_result, Err(SystemError::Conflict(message)) if message.key() == "errors.system.post_code_exists"));
    assert!(matches!(name_result, Err(SystemError::Conflict(message)) if message.key() == "errors.system.post_name_exists"));
}

fn public_config_input(key: &str, value: &str) -> system::domain::ConfigInput {
    system::domain::ConfigInput {
        config_name: key.into(),
        config_key: key.into(),
        config_value: value.into(),
        config_type: "Y".into(),
        public_read: true,
        remark: None,
    }
}

#[tokio::test]
async fn delete_dept_rejects_children_or_users() {
    let children_result = SystemService::new(MemoryRepository::default().with_dept_children(true))
        .delete_dept("103")
        .await;
    let users_result = SystemService::new(MemoryRepository::default().with_dept_users(true)).delete_dept("103").await;

    assert!(matches!(children_result, Err(SystemError::Conflict(message)) if message.key() == "errors.system.dept_has_children_or_users"));
    assert!(matches!(users_result, Err(SystemError::Conflict(message)) if message.key() == "errors.system.dept_has_children_or_users"));
}

#[tokio::test]
async fn page_post_filter_is_trimmed_and_empty_values_are_removed() {
    let repository = MemoryRepository::default();
    let range = created_time_range();
    let filter = PostListFilter {
        page: page(),
        post_code: Some(" ceo ".into()),
        post_name: Some("   ".into()),
        status: Some(" 0 ".into()),
        remark: Some(" ops ".into()),
        begin_time: range.begin_time,
        end_time: range.end_time,
    };

    SystemService::new(repository.clone()).page_posts(filter).await.unwrap();

    assert_eq!(
        repository.last_post_filter(),
        Some(PostListFilter {
            page: page(),
            post_code: Some("ceo".into()),
            post_name: None,
            status: Some("0".into()),
            remark: Some("ops".into()),
            begin_time: range.begin_time,
            end_time: range.end_time,
        })
    );
}

#[tokio::test]
async fn page_dept_filter_is_trimmed_and_empty_values_are_removed() {
    let repository = MemoryRepository::default();
    let range = created_time_range();
    let filter = DeptListFilter {
        page: page(),
        dept_name: Some(" 研发 ".into()),
        leader: Some(" taco ".into()),
        phone: Some(" 13900000000 ".into()),
        email: Some("   ".into()),
        status: Some(" 0 ".into()),
        begin_time: range.begin_time,
        end_time: range.end_time,
    };

    SystemService::new(repository.clone()).page_depts(filter).await.unwrap();

    assert_eq!(
        repository.last_dept_filter(),
        Some(DeptListFilter {
            page: page(),
            dept_name: Some("研发".into()),
            leader: Some("taco".into()),
            phone: Some("13900000000".into()),
            email: None,
            status: Some("0".into()),
            begin_time: range.begin_time,
            end_time: range.end_time,
        })
    );
}

#[tokio::test]
async fn page_config_filter_is_trimmed_and_public_read_is_preserved() {
    let repository = MemoryRepository::default();
    let range = created_time_range();
    let filter = system::application::ConfigListFilter {
        page: page(),
        config_name: Some(" site ".into()),
        config_key: Some("   ".into()),
        config_type: Some(" Y ".into()),
        public_read: Some(true),
        begin_time: range.begin_time,
        end_time: None,
    };

    SystemService::new(repository.clone()).page_configs(filter).await.unwrap();

    assert_eq!(
        repository.last_config_filter(),
        Some(system::application::ConfigListFilter {
            page: page(),
            config_name: Some("site".into()),
            config_key: None,
            config_type: Some("Y".into()),
            public_read: Some(true),
            begin_time: range.begin_time,
            end_time: None,
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

#[tokio::test]
async fn ensure_dept_ids_scoped_rejects_out_of_scope_dept() {
    let service = SystemService::new(MemoryRepository::default().with_dept(dept("104", "100", "市场部门")));

    let result = service.ensure_dept_ids_scoped(vec!["104".into()], data_scope(DataScope::SelfOnly, "103")).await;

    assert!(matches!(result, Err(SystemError::Forbidden(message)) if message.key() == "errors.system.data_scope_forbidden"));
}

#[tokio::test]
async fn ensure_dept_ids_scoped_allows_all_scope() {
    let service = SystemService::new(MemoryRepository::default().with_dept(dept("104", "100", "市场部门")));

    let result = service.ensure_dept_ids_scoped(vec!["104".into()], data_scope(DataScope::All, "103")).await;

    assert!(result.is_ok());
}

fn data_scope(kind: DataScope, dept_id: &str) -> DataScopeFilter {
    DataScopeFilter {
        data_scope: kind,
        user_id: "1".into(),
        dept_id: Some(dept_id.into()),
        dept_ids: vec![],
    }
}
