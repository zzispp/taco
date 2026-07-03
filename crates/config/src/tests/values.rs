use super::*;
use std::{ffi::OsString, path::PathBuf};

#[test]
fn database_url_prefers_explicit_url() {
    let settings = settings_with_database(DatabaseSettings {
        url: Some("postgres://user:pass@remote:5432/app".into()),
        password: Some("ignored".into()),
        ..database_parts()
    });

    assert_eq!(settings.database_url().unwrap(), "postgres://user:pass@remote:5432/app");
}

#[test]
fn database_url_uses_parts_when_url_is_missing() {
    assert_eq!(
        settings_with_database(database_parts()).database_url().unwrap(),
        "postgres://postgres:123456@localhost:5433/postgres"
    );
}

#[test]
fn database_url_uses_parts_when_url_is_blank() {
    let settings = settings_with_database(DatabaseSettings {
        url: Some("  ".into()),
        ..database_parts()
    });

    assert_eq!(
        settings.database_url().unwrap(),
        "postgres://postgres:123456@localhost:5433/postgres"
    );
}

#[test]
fn database_url_errors_without_password_when_url_is_missing() {
    let settings = settings_with_database(DatabaseSettings {
        password: None,
        ..database_parts()
    });

    assert!(matches!(settings.database_url(), Err(SettingsError::MissingDatabasePassword)));
}

#[test]
fn secrets_and_hashes_trim_config_values() {
    let jwt = settings_with_jwt(JwtSettings {
        secret: "  secret-from-config  ".into(),
        ..jwt_settings()
    });
    let admin = settings_with_admin(AdminSettings {
        password_hash: "  hash-from-config  ".into(),
        ..admin_settings()
    });

    assert_eq!(jwt.jwt_secret().unwrap(), "secret-from-config");
    assert_eq!(admin.admin_password_hash().unwrap(), "hash-from-config");
}

#[test]
fn redis_url_trims_explicit_value() {
    let settings = settings_with_redis(RedisSettings {
        url: Some("  redis://localhost:6379/0  ".into()),
        ..redis_settings()
    });

    assert_eq!(settings.redis_url().unwrap(), "redis://localhost:6379/0");
}

#[test]
fn redis_url_uses_parts_when_url_is_missing() {
    let settings = settings_with_redis(RedisSettings {
        url: None,
        ..redis_settings()
    });

    assert_eq!(settings.redis_url().unwrap(), "redis://default:@localhost:6380?protocol=resp3");
}

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
fn explicit_config_path_reads_path_after_config_arg() {
    let args = vec![OsString::from("backend"), OsString::from("--config"), OsString::from("custom.yaml")];

    assert_eq!(
        crate::loader::explicit_config_path(&args).unwrap(),
        Some(PathBuf::from("custom.yaml"))
    );
}

#[test]
fn explicit_config_path_errors_without_value() {
    let args = vec![OsString::from("backend"), OsString::from("--config")];

    assert!(matches!(
        crate::loader::explicit_config_path(&args),
        Err(SettingsError::MissingConfigArgument)
    ));
}

#[test]
fn default_config_paths_are_ordered() {
    assert_eq!(crate::loader::MODULE_CONFIG_PATH, "config/config.yaml");
    assert_eq!(crate::loader::ROOT_CONFIG_PATH, "config.yaml");
}
