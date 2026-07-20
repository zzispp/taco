use super::{
    AdvancedSetupOverrides, InitialAdministrator, InitialAdministratorInput, PostgresConnection, PostgresConnectionInput, RedisConnection,
    RedisConnectionInput, SetupInputError,
};

#[test]
fn postgres_requires_complete_structured_connection_fields() {
    let error = PostgresConnection::new(PostgresConnectionInput {
        host: "postgres.internal".into(),
        port: 0,
        username: "taco".into(),
        password: "secret".into(),
        database: "taco".into(),
        use_tls: true,
    })
    .unwrap_err();

    assert_eq!(error, SetupInputError::NonPositiveNumber("postgres.port"));
}

#[test]
fn redis_blank_optional_values_are_omitted_without_changing_explicit_database_zero() {
    let redis = RedisConnection::new(RedisConnectionInput {
        host: "redis.internal".into(),
        port: 6_379,
        username: Some("  ".into()),
        password: Some("".into()),
        database: Some(0),
        use_tls: false,
    })
    .unwrap();

    assert_eq!(redis.username(), None);
    assert_eq!(redis.password(), None);
    assert_eq!(redis.database(), Some(0));
    assert!(!redis.use_tls());
}

#[test]
fn administrator_password_is_required_but_not_trimmed() {
    let administrator = InitialAdministrator::new(InitialAdministratorInput {
        username: " owner ".into(),
        email: " owner@example.test ".into(),
        password: " secret ".into(),
    })
    .unwrap();

    assert_eq!(administrator.username(), "owner");
    assert_eq!(administrator.email(), "owner@example.test");
    assert_eq!(administrator.password(), " secret ");
}

#[test]
fn advanced_overrides_reject_zero_and_normalize_key_prefix() {
    let valid = AdvancedSetupOverrides {
        redis_key_prefix: Some("  taco-setup:  ".into()),
        ..Default::default()
    }
    .validate()
    .unwrap();
    let invalid = AdvancedSetupOverrides {
        audit_outbox_worker_count: Some(0),
        ..Default::default()
    }
    .validate()
    .unwrap_err();

    assert_eq!(valid.redis_key_prefix.as_deref(), Some("taco-setup:"));
    assert_eq!(invalid, SetupInputError::NonPositiveNumber("advanced.audit_outbox_worker_count"));
}
