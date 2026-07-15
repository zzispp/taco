use super::*;

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
        "postgres://postgres:unit-test-password@localhost:5435/postgres"
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
        "postgres://postgres:unit-test-password@localhost:5435/postgres"
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
fn redis_url_trims_explicit_value() {
    let settings = settings_with_redis(RedisSettings {
        url: Some("  redis://localhost:6379/0  ".into()),
        ..redis_settings()
    });

    assert_eq!(settings.redis_url().unwrap(), "redis://localhost:6379/0");
}

#[test]
fn redis_url_uses_parts_when_url_is_missing() {
    let settings = settings_with_redis(RedisSettings { url: None, ..redis_settings() });

    assert_eq!(settings.redis_url().unwrap(), "redis://default:@localhost:6381?protocol=resp3");
}
