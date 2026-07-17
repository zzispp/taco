use super::*;

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
    let missing = deserialize_settings(&minimal_config().replace(scheduler_yaml(), ""));
    let unknown = deserialize_settings(&minimal_config().replace(
        scheduler_yaml(),
        &format!("{scheduler}\n  unexpected: true\n", scheduler = scheduler_yaml().trim_end()),
    ));

    assert!(missing.is_err());
    assert!(unknown.is_err());
}

#[test]
fn audit_and_client_info_configs_are_required_and_strict() {
    let source = minimal_config();
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
    let source = minimal_config();
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
fn database_auto_migrate_is_required() {
    let source = minimal_config().replacen("  auto_migrate: false\n", "", 1);

    assert!(matches!(
        deserialize_settings(&source),
        Err(SettingsError::InvalidConfigValue { ref path, .. }) if path == "database"
    ));
}

#[test]
fn database_auto_migrate_reads_explicit_true() {
    let settings = deserialize_settings(&minimal_config().replace("auto_migrate: false", "auto_migrate: true")).unwrap();

    assert!(settings.database.auto_migrate);
}

#[test]
fn runtime_policy_fields_are_required_in_yaml() {
    let source = minimal_config();
    let cases = [
        (source.replacen("  request_timeout_ms: 30000\n", "", 1), "http"),
        (source.replace("  compression_enabled: true\n", ""), "http"),
        (source.replace("  enabled: true\n", ""), "metrics"),
    ];

    for (missing, expected_path) in cases {
        let error = deserialize_settings(&missing).unwrap_err();
        assert!(
            matches!(&error, SettingsError::InvalidConfigValue { path, .. } if path == expected_path),
            "{error:?}"
        );
    }
}

#[test]
fn refresh_cookie_domain_is_not_part_of_the_schema() {
    let source = minimal_config().replace(
        "  refresh_cookie:\n    secure: true",
        "  refresh_cookie:\n    secure: true\n    domain: admin.example.test",
    );

    let error = deserialize_settings(&source).unwrap_err();
    assert!(
        matches!(&error, SettingsError::InvalidConfigValue { path, .. } if path == "auth.refresh_cookie.domain"),
        "{error:?}"
    );
}

#[test]
fn required_connection_and_storage_strings_reject_blank_values() {
    let cases = [
        settings_with_database(DatabaseSettings {
            host: " ".into(),
            ..database_parts()
        }),
        settings_with_database(DatabaseSettings {
            username: " ".into(),
            ..database_parts()
        }),
        settings_with_database(DatabaseSettings {
            password: " ".into(),
            ..database_parts()
        }),
        settings_with_database(DatabaseSettings {
            name: " ".into(),
            ..database_parts()
        }),
        Settings {
            redis: RedisSettings {
                host: " ".into(),
                ..redis_settings()
            },
            ..settings_with_database(database_parts())
        },
        Settings {
            redis: RedisSettings {
                key_prefix: " ".into(),
                ..redis_settings()
            },
            ..settings_with_database(database_parts())
        },
        Settings {
            uploads: UploadSettings { avatar_directory: " ".into() },
            ..settings_with_database(database_parts())
        },
    ];
    let keys = [
        "database.host",
        "database.username",
        "database.password",
        "database.name",
        "redis.host",
        "redis.key_prefix",
        "uploads.avatar_directory",
    ];

    for (settings, key) in cases.into_iter().zip(keys) {
        assert!(matches!(settings.validate(), Err(SettingsError::BlankConfigValue(actual)) if actual == key));
    }
}

#[test]
fn listener_and_connection_ports_must_be_positive() {
    let mut server = settings_with_database(database_parts());
    server.server.port = 0;
    let database = settings_with_database(DatabaseSettings { port: 0, ..database_parts() });
    let mut redis = settings_with_database(database_parts());
    redis.redis.port = 0;

    assert!(matches!(server.validate(), Err(SettingsError::NonPositiveNumber("server.port"))));
    assert!(matches!(database.validate(), Err(SettingsError::NonPositiveNumber("database.port"))));
    assert!(matches!(redis.validate(), Err(SettingsError::NonPositiveNumber("redis.port"))));
}
