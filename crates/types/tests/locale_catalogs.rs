use std::collections::{BTreeMap, BTreeSet};

use types::http::{Locale, translate_message, translate_message_with_params};

const CATALOGS: &[(&str, &str)] = &[
    ("zh-CN", include_str!("../locales/zh-CN.yml")),
    ("en", include_str!("../locales/en.yml")),
    ("zh-TW", include_str!("../locales/zh-TW.yml")),
];

const RBAC_DATE_FILTER_KEYS: &[&str] = &["errors.rbac.invalid_date_filter", "errors.rbac.invalid_date_range"];
const SYSTEM_DATE_FILTER_KEYS: &[&str] = &["errors.system.invalid_created_time_filter", "errors.system.invalid_created_time_range"];
const USER_DATE_FILTER_KEYS: &[&str] = &["errors.user.invalid_created_time_filter", "errors.user.invalid_created_time_range"];

const SCHEDULER_CONTRACT_KEYS: &[&str] = &[
    "errors.rbac.invalid_export_batch_config",
    "errors.scheduler.cron_required",
    "errors.scheduler.execution_active",
    "errors.scheduler.ids_required",
    "errors.scheduler.invalid_concurrent",
    "errors.scheduler.invalid_cron",
    "errors.scheduler.invalid_date_filter",
    "errors.scheduler.invalid_date_range",
    "errors.scheduler.invalid_export_batch_config",
    "errors.scheduler.invalid_log_status",
    "errors.scheduler.invalid_misfire_policy",
    "errors.scheduler.invalid_params",
    "errors.scheduler.invalid_preview_count",
    "errors.scheduler.invalid_status",
    "errors.scheduler.job_changed",
    "errors.scheduler.job_group_required",
    "errors.scheduler.job_name_required",
    "errors.scheduler.occurrence_already_materialized",
    "errors.scheduler.params_must_be_object",
    "errors.scheduler.repeatable_mismatch",
    "errors.scheduler.task_already_imported",
    "errors.scheduler.task_cache_refresh_failed",
    "errors.scheduler.task_definition_changed",
    "errors.scheduler.task_http_request_failed",
    "errors.scheduler.task_http_status",
    "errors.scheduler.task_key_required",
    "errors.scheduler.task_missing",
    "errors.system.invalid_export_batch_config",
    "errors.user.invalid_system_config",
    "errors.validation.page_overflow",
    "excel.scheduler.job.headers.concurrent",
    "excel.scheduler.job.headers.cron_expression",
    "excel.scheduler.job.headers.group",
    "excel.scheduler.job.headers.invoke_target",
    "excel.scheduler.job.headers.name",
    "excel.scheduler.job.headers.next_run_at",
    "excel.scheduler.job.headers.registry_status",
    "excel.scheduler.job.headers.runtime_error",
    "excel.scheduler.job.headers.status",
    "excel.scheduler.job.headers.task_key",
    "excel.scheduler.job.sheet",
    "excel.scheduler.job_log.headers.end_time",
    "excel.scheduler.job_log.headers.group",
    "excel.scheduler.job_log.headers.message",
    "excel.scheduler.job_log.headers.name",
    "excel.scheduler.job_log.headers.scheduled_at",
    "excel.scheduler.job_log.headers.start_time",
    "excel.scheduler.job_log.headers.status",
    "excel.scheduler.job_log.headers.task_key",
    "excel.scheduler.job_log.headers.trigger",
    "excel.scheduler.job_log.sheet",
    "scheduler.concurrent.allow",
    "scheduler.concurrent.disallow",
    "scheduler.execution.cancelled_deleted",
    "scheduler.execution.cancelled_edited",
    "scheduler.execution.cancelled_paused",
    "scheduler.execution.failed",
    "scheduler.execution.interrupted_executor_lost",
    "scheduler.execution.skipped_misfire",
    "scheduler.execution.skipped_overlap",
    "scheduler.execution.status.failed",
    "scheduler.execution.status.interrupted",
    "scheduler.execution.status.skipped",
    "scheduler.execution.status.success",
    "scheduler.execution.success",
    "scheduler.execution.task_panicked",
    "scheduler.param_fields.http.body",
    "scheduler.param_fields.http.headers",
    "scheduler.param_fields.http.method",
    "scheduler.param_fields.http.url",
    "scheduler.registry_status.invalid_params",
    "scheduler.registry_status.missing",
    "scheduler.registry_status.ok",
    "scheduler.registry_status.repeatable_mismatch",
    "scheduler.runtime_error.invalid_cron",
    "scheduler.runtime_error.invalid_params",
    "scheduler.runtime_error.repeatable_mismatch",
    "scheduler.runtime_error.task_missing",
    "scheduler.status.normal",
    "scheduler.status.paused",
    "scheduler.task_groups.system",
    "scheduler.tasks.http.request.description",
    "scheduler.tasks.http.request.name",
    "scheduler.tasks.system.refresh_config_cache.description",
    "scheduler.tasks.system.refresh_config_cache.name",
    "scheduler.tasks.system.refresh_dict_cache.description",
    "scheduler.tasks.system.refresh_dict_cache.name",
    "scheduler.trigger.manual",
    "scheduler.trigger.misfire",
    "scheduler.trigger.scheduled",
];

#[test]
fn locale_catalogs_have_identical_keys_and_placeholders() {
    let catalogs = parsed_catalogs();
    let baseline = &catalogs[0].1;
    for (locale, catalog) in &catalogs[1..] {
        assert_eq!(
            catalog.keys().collect::<Vec<_>>(),
            baseline.keys().collect::<Vec<_>>(),
            "locale key mismatch: {locale}"
        );
        for (key, value) in catalog {
            assert_eq!(placeholders(value), placeholders(&baseline[key]), "placeholder mismatch for {locale}:{key}");
        }
    }
}

#[test]
fn scheduler_catalog_contract_is_complete() {
    for (locale, catalog) in parsed_catalogs() {
        for key in SCHEDULER_CONTRACT_KEYS {
            let value = catalog.get(*key).unwrap_or_else(|| panic!("missing scheduler locale key {locale}:{key}"));
            assert_eq!(placeholders(value), expected_placeholders(key), "invalid placeholders for {locale}:{key}");
        }
    }
}

#[test]
fn scheduler_runtime_messages_translate_in_all_locales() {
    assert_eq!(translate_message(Locale::ZhCn, "scheduler.execution.status.interrupted"), "已中断");
    assert_eq!(translate_message(Locale::En, "scheduler.execution.status.interrupted"), "Interrupted");
    assert_eq!(translate_message(Locale::ZhTw, "scheduler.execution.status.interrupted"), "已中斷");
    assert_eq!(
        translate_message_with_params(Locale::En, "errors.scheduler.invalid_preview_count", &[("max", "20".into())]),
        "Preview count must be between 1 and 20"
    );
}

#[test]
fn rbac_date_filter_contract_is_complete() {
    for (locale, catalog) in parsed_catalogs() {
        for key in RBAC_DATE_FILTER_KEYS {
            let value = catalog.get(*key).unwrap_or_else(|| panic!("missing RBAC locale key {locale}:{key}"));
            assert_eq!(placeholders(value), expected_placeholders(key), "invalid placeholders for {locale}:{key}");
        }
    }
    assert_eq!(translate_message(Locale::ZhCn, "errors.rbac.invalid_date_range"), "开始时间不能晚于结束时间");
    assert_eq!(
        translate_message(Locale::En, "errors.rbac.invalid_date_range"),
        "Start time must not be later than end time"
    );
    assert_eq!(translate_message(Locale::ZhTw, "errors.rbac.invalid_date_range"), "開始時間不能晚於結束時間");
    assert_eq!(
        translate_message_with_params(
            Locale::ZhCn,
            "errors.rbac.invalid_date_filter",
            &[("field", "begin_time".into()), ("format", "YYYY-MM-DD / RFC3339".into())],
        ),
        "日期筛选字段begin_time格式无效，请使用YYYY-MM-DD / RFC3339"
    );
    assert_eq!(
        translate_message_with_params(
            Locale::En,
            "errors.rbac.invalid_date_filter",
            &[("field", "begin_time".into()), ("format", "YYYY-MM-DD / RFC3339".into())],
        ),
        "Date filter field begin_time is invalid. Use YYYY-MM-DD / RFC3339."
    );
    assert_eq!(
        translate_message_with_params(
            Locale::ZhTw,
            "errors.rbac.invalid_date_filter",
            &[("field", "begin_time".into()), ("format", "YYYY-MM-DD / RFC3339".into())],
        ),
        "日期篩選欄位begin_time格式無效，請使用YYYY-MM-DD / RFC3339"
    );
}

#[test]
fn user_date_filter_contract_is_complete() {
    for (locale, catalog) in parsed_catalogs() {
        for key in USER_DATE_FILTER_KEYS {
            let value = catalog.get(*key).unwrap_or_else(|| panic!("missing user locale key {locale}:{key}"));
            assert_eq!(placeholders(value), expected_placeholders(key), "invalid placeholders for {locale}:{key}");
        }
    }
    assert_eq!(
        translate_message_with_params(
            Locale::En,
            "errors.user.invalid_created_time_filter",
            &[("format", "YYYY-MM-DD / RFC3339".into())],
        ),
        "Invalid user creation time filter. Use YYYY-MM-DD / RFC3339."
    );
    assert_eq!(
        translate_message(Locale::ZhCn, "errors.user.invalid_created_time_range"),
        "用户创建时间范围无效，开始时间不能晚于结束时间"
    );
    assert_eq!(
        translate_message(Locale::ZhTw, "errors.user.invalid_created_time_range"),
        "使用者建立時間範圍無效，開始時間不能晚於結束時間"
    );
}

#[test]
fn system_date_filter_contract_is_complete() {
    for (locale, catalog) in parsed_catalogs() {
        for key in SYSTEM_DATE_FILTER_KEYS {
            let value = catalog.get(*key).unwrap_or_else(|| panic!("missing system locale key {locale}:{key}"));
            assert_eq!(placeholders(value), expected_placeholders(key), "invalid placeholders for {locale}:{key}");
        }
    }
    let params = &[("field", "begin_time".into()), ("format", "YYYY-MM-DD / RFC3339".into())];
    assert_eq!(
        translate_message_with_params(Locale::En, "errors.system.invalid_created_time_filter", params),
        "Creation time filter field begin_time is invalid. Use YYYY-MM-DD / RFC3339."
    );
    assert_eq!(
        translate_message(Locale::ZhCn, "errors.system.invalid_created_time_range"),
        "创建时间范围无效，开始时间不能晚于结束时间"
    );
    assert_eq!(
        translate_message(Locale::ZhTw, "errors.system.invalid_created_time_range"),
        "建立時間範圍無效，開始時間不能晚於結束時間"
    );
}

fn parsed_catalogs() -> Vec<(&'static str, BTreeMap<String, String>)> {
    CATALOGS.iter().map(|(locale, source)| (*locale, parse_catalog(source))).collect()
}

fn parse_catalog(source: &str) -> BTreeMap<String, String> {
    let mut entries = BTreeMap::new();
    for (index, line) in source.lines().enumerate() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let (key, value) = line.split_once(':').unwrap_or_else(|| panic!("invalid locale entry on line {}", index + 1));
        let previous = entries.insert(key.trim().to_owned(), value.trim().trim_matches('"').to_owned());
        assert!(previous.is_none(), "duplicate locale key {}", key.trim());
    }
    entries
}

fn placeholders(value: &str) -> BTreeSet<String> {
    let mut remaining = value;
    let mut result = BTreeSet::new();
    while let Some(start) = remaining.find("%{") {
        let after_start = &remaining[start + 2..];
        let end = after_start.find('}').unwrap_or_else(|| panic!("unterminated placeholder in {value}"));
        result.insert(after_start[..end].to_owned());
        remaining = &after_start[end + 1..];
    }
    result
}

fn expected_placeholders(key: &str) -> BTreeSet<String> {
    let names: &[&str] = match key {
        "errors.rbac.invalid_date_filter" => &["field", "format"],
        "errors.scheduler.invalid_date_filter" => &["field"],
        "errors.scheduler.invalid_preview_count" => &["max"],
        "errors.scheduler.task_cache_refresh_failed" => &["kind"],
        "errors.scheduler.task_http_status" => &["status"],
        "errors.system.invalid_created_time_filter" => &["field", "format"],
        "errors.user.invalid_created_time_filter" => &["format"],
        "errors.user.invalid_system_config" => &["key"],
        _ => &[],
    };
    names.iter().map(|name| (*name).to_owned()).collect()
}
