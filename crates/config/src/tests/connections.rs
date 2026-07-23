use super::*;

#[test]
fn database_url_encodes_structured_components() {
    let database = DatabaseSettings {
        username: "db:%user%2F".into(),
        password: "p@ss:/#?%2F".into(),
        name: "app/name".into(),
        ..database_parts()
    };

    assert_eq!(
        database.url().unwrap(),
        "postgres://db%3A%25user%252F:p%40ss%3A%2F%23%3F%252F@localhost:5435/app%2Fname?sslmode=disable"
    );
}

#[test]
fn database_url_uses_only_structured_fields() {
    assert_eq!(
        database_parts().url().unwrap(),
        "postgres://postgres:unit-test-password@localhost:5435/postgres?sslmode=disable"
    );
}

#[test]
fn redis_url_supports_password_without_username() {
    let redis = RedisSettings {
        username: None,
        password: Some("p@ss:/#?%2F".into()),
        ..redis_settings()
    };

    assert_eq!(redis.url().unwrap(), "redis://:p%40ss%3A%2F%23%3F%252F@localhost:6381?protocol=resp3");
}

#[test]
fn redis_url_encodes_database_and_protocol() {
    let redis = RedisSettings {
        username: Some("cache:%user%2F".into()),
        password: Some("secret".into()),
        database: Some(7),
        protocol: Some(RedisProtocol::Resp3),
        ..redis_settings()
    };

    assert_eq!(redis.url().unwrap(), "redis://cache%3A%25user%252F:secret@localhost:6381/7?protocol=resp3");
}

#[test]
fn rediss_url_selects_verified_tls_in_the_runtime_client() {
    let redis = RedisSettings {
        scheme: RedisScheme::Rediss,
        ..redis_settings()
    };
    let client = redis::Client::open(redis.url().unwrap()).unwrap();

    match client.get_connection_info().addr() {
        redis::ConnectionAddr::TcpTls { host, port, insecure, .. } => {
            assert_eq!(host, "localhost");
            assert_eq!(*port, 6_381);
            assert!(!insecure);
        }
        address => panic!("expected verified TLS Redis address, got {address:?}"),
    }
}

#[test]
fn connection_components_must_be_present_and_non_blank() {
    let blank_database_host = DatabaseSettings {
        host: "   ".into(),
        ..database_parts()
    };
    let blank_redis_password = RedisSettings {
        password: Some("".into()),
        ..redis_settings()
    };

    assert!(matches!(blank_database_host.url(), Err(SettingsError::BlankConfigValue("database.host"))));
    assert!(matches!(blank_redis_password.url(), Err(SettingsError::BlankConfigValue("redis.password"))));
}
