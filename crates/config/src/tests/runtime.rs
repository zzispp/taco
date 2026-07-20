use super::*;

#[test]
fn audit_outbox_config_rejects_each_zero_value() {
    let mut workers = settings_with_database(database_parts());
    workers.audit.outbox.worker_count = 0;
    let mut lease = settings_with_database(database_parts());
    lease.audit.outbox.lease_duration_ms = 0;
    let mut retention = settings_with_database(database_parts());
    retention.audit.outbox.processed_retention_days = 0;

    assert!(matches!(
        workers.audit_config(),
        Err(SettingsError::NonPositiveNumber("audit.outbox.worker_count"))
    ));
    assert!(matches!(
        lease.audit_config(),
        Err(SettingsError::NonPositiveNumber("audit.outbox.lease_duration_ms"))
    ));
    assert!(matches!(
        retention.audit_config(),
        Err(SettingsError::NonPositiveNumber("audit.outbox.processed_retention_days"))
    ));
}

#[test]
fn client_info_http_timeout_must_be_positive() {
    let mut settings = settings_with_database(database_parts());
    settings.client_info.ip_location.request_timeout_ms = 0;

    assert!(matches!(
        settings.client_info_config(),
        Err(SettingsError::NonPositiveNumber("client_info.ip_location.request_timeout_ms"))
    ));
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
    let mut request_timeout = valid_settings();
    request_timeout.scheduler.http_client.request_timeout_ms = 0;
    let mut reconcile_interval = valid_settings();
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
fn online_session_cleanup_config_requires_positive_values() {
    let mut interval = valid_settings();
    interval.user.online_sessions.cleanup_interval_ms = 0;
    let mut batch = valid_settings();
    batch.user.online_sessions.cleanup_batch_size = 0;

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
fn full_settings_validation_rejects_blank_connection_and_derived_storage_values() {
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
            ..valid_settings()
        },
        Settings {
            redis: RedisSettings {
                key_prefix: " ".into(),
                ..redis_settings()
            },
            ..valid_settings()
        },
        Settings {
            uploads: UploadSettings { avatar_directory: " ".into() },
            ..valid_settings()
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
    let mut server = valid_settings();
    server.server.port = 0;
    let database = settings_with_database(DatabaseSettings { port: 0, ..database_parts() });
    let mut redis = valid_settings();
    redis.redis.port = 0;

    assert!(matches!(server.validate(), Err(SettingsError::NonPositiveNumber("server.port"))));
    assert!(matches!(database.validate(), Err(SettingsError::NonPositiveNumber("database.port"))));
    assert!(matches!(redis.validate(), Err(SettingsError::NonPositiveNumber("redis.port"))));
}
