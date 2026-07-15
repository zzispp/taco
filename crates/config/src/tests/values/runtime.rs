use super::*;

#[test]
fn tracing_config_validates_log_level_and_file_settings() {
    let settings = settings_with_tracing(TracingSettings {
        log_level: "debug".into(),
        file: TracingFileSettings {
            enabled: true,
            directory: " logs ".into(),
            prefix: " app.log ".into(),
        },
    });

    let tracing = settings.tracing_config().unwrap();

    assert_eq!(tracing.log_level, "debug");
    assert!(tracing.file.enabled);
}

#[test]
fn http_config_rejects_zero_timeout() {
    let settings = settings_with_http(HttpSettings {
        request_timeout_ms: 0,
        compression_enabled: true,
    });

    assert!(matches!(
        settings.http_config(),
        Err(SettingsError::NonPositiveNumber("http.request_timeout_ms"))
    ));
}

#[test]
fn scheduler_config_rejects_non_positive_runtime_values() {
    let mut request_timeout = settings_with_database(database_parts());
    request_timeout.scheduler.http_client.request_timeout_ms = 0;
    let mut reconcile_interval = settings_with_database(database_parts());
    reconcile_interval.scheduler.runtime.reconcile_interval_ms = 0;

    assert!(matches!(
        request_timeout.scheduler_config(),
        Err(SettingsError::NonPositiveNumber("scheduler.http_client.request_timeout_ms"))
    ));
    assert!(matches!(
        reconcile_interval.scheduler_config(),
        Err(SettingsError::NonPositiveNumber("scheduler.runtime.reconcile_interval_ms"))
    ));
}

#[test]
fn scheduler_config_is_required_and_rejects_unknown_fields() {
    let missing = deserialize_settings(&minimal_config_without_auto_migrate().replace(scheduler_yaml(), ""));
    let unknown = deserialize_settings(&minimal_config_without_auto_migrate().replace(
        scheduler_yaml(),
        &format!("{scheduler}\n  unexpected: true\n", scheduler = scheduler_yaml().trim_end()),
    ));

    assert!(missing.is_err());
    assert!(unknown.is_err());
}

#[test]
fn audit_and_client_info_configs_are_required_and_strict() {
    let source = minimal_config_without_auto_migrate();
    let missing_audit = deserialize_settings(&source.replace(audit_yaml(), ""));
    let missing_client = deserialize_settings(&source.replace(client_info_yaml(), ""));
    let unknown_audit = deserialize_settings(&source.replace("    worker_count: 4", "    worker_count: 4\n    unexpected: true"));
    let unknown_client = deserialize_settings(&source.replace("    request_timeout_ms: 3000", "    request_timeout_ms: 3000\n    unexpected: true"));

    assert!(missing_audit.is_err());
    assert!(missing_client.is_err());
    assert!(unknown_audit.is_err());
    assert!(unknown_client.is_err());
}

#[test]
fn online_session_cleanup_config_is_required_strict_and_positive() {
    let source = minimal_config_without_auto_migrate();
    let missing = deserialize_settings(&source.replace(user_yaml(), ""));
    let unknown = deserialize_settings(&source.replace("    cleanup_interval_ms: 60000", "    cleanup_interval_ms: 60000\n    unexpected: true"));
    let mut interval = settings_with_database(database_parts());
    interval.user.online_sessions.cleanup_interval_ms = 0;
    let mut batch = settings_with_database(database_parts());
    batch.user.online_sessions.cleanup_batch_size = 0;

    assert!(missing.is_err());
    assert!(unknown.is_err());
    assert!(matches!(
        interval.online_session_config(),
        Err(SettingsError::NonPositiveNumber("user.online_sessions.cleanup_interval_ms"))
    ));
    assert!(matches!(
        batch.online_session_config(),
        Err(SettingsError::NonPositiveNumber("user.online_sessions.cleanup_batch_size"))
    ));
}

#[test]
fn database_auto_migrate_defaults_to_false() {
    let settings = deserialize_settings(&minimal_config_without_auto_migrate()).unwrap();

    assert!(!settings.database.auto_migrate);
}

#[test]
fn database_auto_migrate_reads_explicit_true() {
    let settings = deserialize_settings(&minimal_config_without_auto_migrate().replacen("database:\n", "database:\n  auto_migrate: true\n", 1)).unwrap();

    assert!(settings.database.auto_migrate);
}
