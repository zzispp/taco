use chrono::{DateTime, Utc};
use scheduler::{
    application::{
        NEXT_TIMES_MAX_COUNT, next_times_after,
        task::{ScheduledTaskMetadata, StaticTaskCatalog, TaskCatalog, TaskParams},
        tasks::{
            CleanupUploadSessionsTask, HttpRequestParams, HttpRequestTask, NoTaskParams, PurgeTrashTask, RefreshConfigCacheTask, RefreshDictCacheTask,
            SystemLogCleanupTask,
        },
    },
    domain::{ParamSchema, ParamWidget},
};
use scheduler_macros::ScheduledTaskParams;
use serde::{Deserialize, Serialize};
use serde_json::json;
use types::http::{Locale, translate_message};

const CONTRACT_SCHEMA_VERSION: i16 = 7;

#[derive(Debug, Deserialize, PartialEq, ScheduledTaskParams, Serialize)]
#[task_params(schema_version = CONTRACT_SCHEMA_VERSION)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContractParams {
    #[serde(default)]
    enabled_flag: bool,
    #[param_field(required, disabled_when_path = "enabled_flag", disabled_when_values = [false])]
    required_note: Option<String>,
    #[serde(default = "default_attempt_limit")]
    attempt_limit: u32,
    #[serde(default, rename = "feature_flags")]
    feature_flags: Vec<bool>,
    #[serde(default)]
    #[param_field(disabled_when_path = "attempt_limit", disabled_when_values = [0, 3])]
    sample_sizes: Vec<u64>,
}

fn default_attempt_limit() -> u32 {
    3
}

#[test]
fn explicit_catalog_contains_each_builtin_task_once() {
    let definitions = vec![
        HttpRequestTask::descriptor(),
        RefreshConfigCacheTask::descriptor(),
        RefreshDictCacheTask::descriptor(),
        SystemLogCleanupTask::descriptor(),
        PurgeTrashTask::descriptor(),
        CleanupUploadSessionsTask::descriptor(),
    ];
    let catalog = StaticTaskCatalog::try_new(definitions.clone()).expect("builtin task keys must be unique");
    let keys = catalog.all().into_iter().map(|item| item.task_key).collect::<Vec<_>>();
    let http = catalog.get("httpClient.request").expect("HTTP task must be registered");

    assert_eq!(
        keys,
        vec![
            "file.cleanupUploadSessions",
            "file.purgeTrash",
            "httpClient.request",
            "observability.cleanupSystemLogs",
            "system.refreshConfigCache",
            "system.refreshDictCache"
        ]
    );
    assert_eq!(http.group, "SYSTEM");
    assert!(http.repeatable);

    let duplicate = StaticTaskCatalog::try_new([definitions[0], definitions[0]]);
    assert!(duplicate.is_err());
}

#[test]
fn http_task_param_form_uses_typed_schema_and_ui_metadata() {
    let form = HttpRequestParams::form();
    let field_paths = form.ui.fields.iter().map(|field| field.path.as_str()).collect::<Vec<_>>();
    let header_field = form.ui.fields.iter().find(|field| field.path == "headers").expect("headers field must exist");

    assert_eq!(form.schema_version, HttpRequestParams::SCHEMA_VERSION);
    assert_eq!(field_paths, vec!["method", "url", "headers", "body"]);
    assert_eq!(header_field.widget, ParamWidget::KeyValue);
    assert!(header_field.options.is_empty());
    assert!(matches!(form.schema, ParamSchema::Object(_)));
}

#[test]
fn http_params_validate_method_url_and_object_shape() {
    let valid = json!({
        "method": "POST",
        "url": "https://example.com/webhook",
        "headers": {"Content-Type": "application/json"},
        "body": "{\"ok\":true}"
    });

    assert!(HttpRequestParams::validate(&valid).is_ok());
    assert!(HttpRequestParams::validate(&json!({"method":"TRACE","url":"https://example.com","headers":{}})).is_err());
    assert!(HttpRequestParams::validate(&json!({"method":"GET","url":"ftp://example.com","headers":{}})).is_err());
    assert!(HttpRequestParams::validate(&json!({"method":"GET","url":"https://example.com","unexpected":true})).is_err());
}

#[test]
fn no_task_params_accepts_only_an_empty_object() {
    assert!(NoTaskParams::validate(&json!({})).is_ok());
    assert!(NoTaskParams::validate(&json!({"unexpected": true})).is_err());
    assert!(NoTaskParams::validate(&json!(null)).is_err());
}

#[test]
fn derived_param_contract_uses_wire_names_and_typed_collections() {
    let form = ContractParams::form();
    let ParamSchema::Object(schema) = form.schema else {
        panic!("derived parameter root must be an object");
    };
    let fields = form.ui.fields.iter().map(|field| (field.path.as_str(), field.widget)).collect::<Vec<_>>();
    let required_condition = form.ui.fields[1].disabled_when.as_ref().expect("required note condition must exist");
    let sample_condition = form.ui.fields[4].disabled_when.as_ref().expect("sample size condition must exist");

    assert_eq!(form.schema_version, CONTRACT_SCHEMA_VERSION);
    assert_eq!(schema.required, vec!["requiredNote"]);
    assert!(matches!(schema.properties["enabledFlag"], ParamSchema::Boolean(_)));
    assert!(matches!(schema.properties["requiredNote"], ParamSchema::String(_)));
    assert!(matches!(schema.properties["attemptLimit"], ParamSchema::Number(_)));
    assert!(matches!(
        &schema.properties["feature_flags"],
        ParamSchema::Array(array) if matches!(array.items.as_ref(), ParamSchema::Boolean(_))
    ));
    assert!(matches!(
        &schema.properties["sampleSizes"],
        ParamSchema::Array(array) if matches!(array.items.as_ref(), ParamSchema::Number(_))
    ));
    assert_eq!(
        fields,
        vec![
            ("enabledFlag", ParamWidget::Switch),
            ("requiredNote", ParamWidget::Text),
            ("attemptLimit", ParamWidget::Number),
            ("feature_flags", ParamWidget::JsonEditor),
            ("sampleSizes", ParamWidget::JsonEditor),
        ]
    );
    assert_eq!(required_condition.path, "enabledFlag");
    assert_eq!(required_condition.values, vec![json!(false)]);
    assert_eq!(sample_condition.path, "attemptLimit");
    assert_eq!(sample_condition.values, vec![json!(0), json!(3)]);
}

#[test]
fn required_option_rejects_missing_and_null_before_deserialization() {
    assert!(ContractParams::validate(&json!({})).is_err());
    assert!(ContractParams::validate(&json!({"requiredNote": null})).is_err());
    assert!(ContractParams::validate(&json!({"requiredNote": "present"})).is_ok());
}

#[test]
fn optional_field_is_omitted_and_serde_defaults_use_wire_names() {
    assert_eq!(
        ContractParams::default_params(),
        json!({"enabledFlag": false, "attemptLimit": 3, "feature_flags": [], "sampleSizes": []})
    );
}

#[test]
fn cron_next_times_are_deterministic_from_the_supplied_clock() {
    let now = "2026-07-10T12:01:30Z".parse::<DateTime<Utc>>().expect("fixed timestamp must parse");
    let times = next_times_after("0 0/5 * * * ? *", Some(3), now).expect("valid Quartz cron must parse");

    assert_eq!(
        times,
        vec![
            "2026-07-10T12:05:00Z".parse::<DateTime<Utc>>().expect("fixed timestamp must parse"),
            "2026-07-10T12:10:00Z".parse::<DateTime<Utc>>().expect("fixed timestamp must parse"),
            "2026-07-10T12:15:00Z".parse::<DateTime<Utc>>().expect("fixed timestamp must parse"),
        ]
    );
    assert!(next_times_after("0 0/5 * * * ? *", Some(0), now).is_err());
    assert!(next_times_after("0 0/5 * * * ? *", Some(NEXT_TIMES_MAX_COUNT + 1), now).is_err());
}

#[test]
fn scheduler_task_i18n_keys_are_translated() {
    assert_eq!(translate_message(Locale::ZhCn, "scheduler.tasks.http.request.name"), "HTTP 请求");
    assert_eq!(translate_message(Locale::ZhCn, "scheduler.task_groups.system"), "系统");
    assert_eq!(
        translate_message(Locale::ZhCn, "scheduler.tasks.system.refresh_config_cache.name"),
        "刷新参数缓存"
    );
    assert_eq!(
        translate_message(Locale::ZhCn, "scheduler.tasks.system.refresh_dict_cache.name"),
        "刷新字典缓存"
    );
    assert_eq!(
        translate_message(Locale::ZhCn, "scheduler.tasks.observability.system_log_cleanup.name"),
        "系统日志清理"
    );
    assert_eq!(translate_message(Locale::ZhCn, "scheduler.tasks.file.purge_trash.name"), "文件回收站清理");
    assert_eq!(
        translate_message(Locale::ZhCn, "scheduler.tasks.file.cleanup_upload_sessions.name"),
        "文件上传会话清理"
    );
}
