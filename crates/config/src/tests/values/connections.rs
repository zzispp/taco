use super::*;

#[test]
fn database_url_encodes_structured_components() {
    let settings = settings_with_database(DatabaseSettings {
        username: "db:%user%2F".into(),
        password: "p@ss:/#?%2F".into(),
        name: "app/name".into(),
        ..database_parts()
    });

    assert_eq!(
        settings.database_url().unwrap(),
        "postgres://db%3A%25user%252F:p%40ss%3A%2F%23%3F%252F@localhost:5435/app%2Fname?sslmode=disable"
    );
}

#[test]
fn database_url_uses_only_structured_fields() {
    assert_eq!(
        settings_with_database(database_parts()).database_url().unwrap(),
        "postgres://postgres:unit-test-password@localhost:5435/postgres?sslmode=disable"
    );
}

#[test]
fn redis_url_supports_password_without_username() {
    let settings = settings_with_redis(RedisSettings {
        username: None,
        password: Some("p@ss:/#?%2F".into()),
        ..redis_settings()
    });

    assert_eq!(settings.redis_url().unwrap(), "redis://:p%40ss%3A%2F%23%3F%252F@localhost:6381?protocol=resp3");
}

#[test]
fn redis_url_encodes_database_and_protocol() {
    let settings = settings_with_redis(RedisSettings {
        username: Some("cache:%user%2F".into()),
        password: Some("secret".into()),
        database: Some(7),
        protocol: Some(RedisProtocol::Resp3),
        ..redis_settings()
    });

    assert_eq!(
        settings.redis_url().unwrap(),
        "redis://cache%3A%25user%252F:secret@localhost:6381/7?protocol=resp3"
    );
}

#[test]
fn rediss_url_selects_verified_tls_in_the_runtime_client() {
    let settings = settings_with_redis(RedisSettings {
        scheme: RedisScheme::Rediss,
        ..redis_settings()
    });
    let client = redis::Client::open(settings.redis_url().unwrap()).unwrap();

    match client.get_connection_info().addr() {
        redis::ConnectionAddr::TcpTls { host, port, insecure, .. } => {
            assert_eq!(host, "localhost");
            assert_eq!(*port, 6381);
            assert!(!insecure);
        }
        address => panic!("expected verified TLS Redis address, got {address:?}"),
    }
}

#[test]
fn typed_connection_values_reject_unsupported_protocols() {
    let cases = [
        (minimal_config().replace("scheme: \"postgres\"", "scheme: \"http\""), "database.scheme"),
        (minimal_config().replacen("scheme: \"redis\"", "scheme: \"http\"", 1), "redis.scheme"),
        (
            minimal_config().replace("ssl_mode: \"disable\"", "ssl_mode: \"opportunistic\""),
            "database.ssl_mode",
        ),
        (minimal_config().replace("protocol: \"resp3\"", "protocol: \"resp4\""), "redis.protocol"),
    ];

    for (source, expected_path) in cases {
        let error = deserialize_settings(&source).unwrap_err();
        assert!(
            matches!(&error, SettingsError::InvalidConfigValue { path, .. } if path == expected_path),
            "{error:?}"
        );
    }
}

#[test]
fn legacy_connection_url_fields_are_rejected() {
    let source = minimal_config();
    let database_url = source.replacen("database:\n", "database:\n  url: \"postgres://localhost/app\"\n", 1);
    let redis_url = source.replacen("redis:\n", "redis:\n  url: \"redis://localhost\"\n", 1);

    let database_error = deserialize_settings(&database_url).unwrap_err();
    let redis_error = deserialize_settings(&redis_url).unwrap_err();

    assert!(
        matches!(&database_error, SettingsError::InvalidConfigValue { path, .. } if path == "database.url"),
        "{database_error:?}"
    );
    assert!(
        matches!(&redis_error, SettingsError::InvalidConfigValue { path, .. } if path == "redis.url"),
        "{redis_error:?}"
    );
}

#[test]
fn connection_components_must_be_present_and_non_blank() {
    let blank_database_host = settings_with_database(DatabaseSettings {
        host: "   ".into(),
        ..database_parts()
    });
    let blank_redis_password = settings_with_redis(RedisSettings {
        password: Some("".into()),
        ..redis_settings()
    });

    assert!(matches!(
        blank_database_host.database_url(),
        Err(SettingsError::BlankConfigValue("database.host"))
    ));
    assert!(matches!(
        blank_redis_password.redis_url(),
        Err(SettingsError::BlankConfigValue("redis.password"))
    ));
}
