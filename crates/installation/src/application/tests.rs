use configuration::{DatabaseSslMode, InstallationProfile, RedisProtocol, RedisScheme};

use crate::{
    application::{InstallationCompleted, SetupError, SetupUseCase, postgres_connection_settings, redis_connection_settings},
    domain::{AdvancedSetupOverrides, PostgresConnection, PostgresConnectionInput, RedisConnection, RedisConnectionInput},
};

mod support;

use support::{Call, FailureStage, TEST_JWT_SECRET, installation_input, installation_input_with_tls, service_with};

#[tokio::test]
async fn final_install_revalidates_connections_before_migration_and_persists_once() {
    let (service, port) = service_with(None);

    let result = service
        .install(installation_input(AdvancedSetupOverrides {
            http_request_timeout_ms: Some(12_345),
            metrics_enabled: Some(false),
            redis_key_prefix: Some("custom:".into()),
            ..Default::default()
        }))
        .await;

    assert_eq!(result, Ok(InstallationCompleted));
    assert_eq!(
        port.calls(),
        vec![
            Call::ValidateOwner,
            Call::PostgresTest,
            Call::DetectExistingInstallation,
            Call::RedisTest,
            Call::ResetData,
            Call::Migrate,
            Call::ProvisionOwner,
            Call::GenerateJwt,
            Call::WriteState,
            Call::Shutdown,
        ]
    );
    let installation = port.written_installation().unwrap();
    assert!(installation.complete);
    assert_eq!(installation.profile.http.request_timeout_ms, 12_345);
    assert!(!installation.profile.metrics.enabled);
    assert_eq!(installation.profile.redis.key_prefix, "custom:");
    assert_eq!(installation.profile.database.host, "postgres.internal");
    assert_eq!(installation.profile.redis.host, "redis.internal");
    assert_eq!(installation.profile.jwt.secret, TEST_JWT_SECRET);
}

#[tokio::test]
async fn existing_installation_stops_before_redis_or_data_reset() {
    let (service, port) = service_with(Some(FailureStage::ExistingInstallation));

    let result = service.install(installation_input(AdvancedSetupOverrides::default())).await;

    assert_eq!(result, Err(SetupError::ExistingInstallationDetected));
    assert_eq!(port.calls(), vec![Call::ValidateOwner, Call::PostgresTest, Call::DetectExistingInstallation]);
    assert_eq!(port.written_installation(), None);
}

#[tokio::test]
async fn invalid_installation_owner_stops_before_connection_tests_and_data_reset() {
    let (service, port) = service_with(Some(FailureStage::OwnerValidation));

    let result = service.install(installation_input(AdvancedSetupOverrides::default())).await;

    assert_eq!(result, Err(SetupError::InstallationOwnerInvalid));
    assert_eq!(port.calls(), vec![Call::ValidateOwner]);
    assert_eq!(port.written_installation(), None);
}

#[tokio::test]
async fn failed_data_reset_skips_migration_and_state_persistence() {
    let (service, port) = service_with(Some(FailureStage::DataReset));

    let result = service.install(installation_input(AdvancedSetupOverrides::default())).await;

    assert_eq!(result, Err(SetupError::InstallationDataResetFailed));
    assert_eq!(
        port.calls(),
        vec![
            Call::ValidateOwner,
            Call::PostgresTest,
            Call::DetectExistingInstallation,
            Call::RedisTest,
            Call::ResetData
        ]
    );
    assert_eq!(port.written_installation(), None);
}

#[tokio::test]
async fn failed_migration_keeps_setup_state_unwritten() {
    let (service, port) = service_with(Some(FailureStage::Migration));

    let result = service.install(installation_input(AdvancedSetupOverrides::default())).await;

    assert_eq!(result, Err(SetupError::MigrationFailed));
    assert_eq!(
        port.calls(),
        vec![
            Call::ValidateOwner,
            Call::PostgresTest,
            Call::DetectExistingInstallation,
            Call::RedisTest,
            Call::ResetData,
            Call::Migrate
        ]
    );
    assert_eq!(port.written_installation(), None);
}

#[tokio::test]
async fn failed_state_write_does_not_request_shutdown() {
    let (service, port) = service_with(Some(FailureStage::StateWrite));

    let result = service.install(installation_input(AdvancedSetupOverrides::default())).await;

    assert_eq!(result, Err(SetupError::InstallationStatePersistenceFailed));
    assert_eq!(port.calls().last(), Some(&Call::WriteState));
    assert!(!port.calls().contains(&Call::Shutdown));
}

#[tokio::test]
async fn completed_setup_rejects_a_second_final_submission() {
    let (service, port) = service_with(None);

    service.install(installation_input(AdvancedSetupOverrides::default())).await.unwrap();
    let result = service.install(installation_input(AdvancedSetupOverrides::default())).await;

    assert_eq!(result, Err(SetupError::AlreadyInstalled));
    assert_eq!(port.calls().iter().filter(|call| **call == Call::Migrate).count(), 1);
}

#[tokio::test]
async fn failed_owner_creation_does_not_persist_installation_state() {
    let (service, port) = service_with(Some(FailureStage::OwnerProvisioning));

    let result = service.install(installation_input(AdvancedSetupOverrides::default())).await;

    assert_eq!(result, Err(SetupError::InstallationOwnerProvisioningFailed));
    assert_eq!(port.written_installation(), None);
    assert!(!port.calls().contains(&Call::WriteState));
}

#[tokio::test]
async fn tls_switches_map_to_verified_or_plaintext_profile_connections() {
    let (service, port) = service_with(None);

    service
        .install(installation_input_with_tls(AdvancedSetupOverrides::default(), false, false))
        .await
        .unwrap();

    let profile = port.written_installation().unwrap().profile;
    assert_eq!(profile.database.ssl_mode, DatabaseSslMode::Disable);
    assert_eq!(profile.redis.scheme, RedisScheme::Redis);
}

#[test]
fn connection_setting_mappers_reuse_installation_profile_defaults() {
    let postgres = PostgresConnection::new(PostgresConnectionInput {
        host: "postgres.internal".into(),
        port: 5_436,
        username: "taco".into(),
        password: "postgres-secret".into(),
        database: "taco".into(),
        use_tls: true,
    })
    .unwrap();
    let redis = RedisConnection::new(RedisConnectionInput {
        host: "redis.internal".into(),
        port: 6_380,
        username: Some("cache-user".into()),
        password: Some("redis-secret".into()),
        database: Some(4),
        use_tls: false,
    })
    .unwrap();

    let database = postgres_connection_settings(&postgres);
    let cache = redis_connection_settings(&redis);
    let defaults = InstallationProfile::default();

    assert_eq!(database.scheme, defaults.database.scheme);
    assert_eq!(database.ssl_mode, DatabaseSslMode::VerifyFull);
    assert_eq!(database.host, "postgres.internal");
    assert_eq!(database.port, 5_436);
    assert_eq!(cache.scheme, RedisScheme::Redis);
    assert_eq!(cache.protocol, Some(RedisProtocol::Resp3));
    assert_eq!(cache.key_prefix, defaults.redis.key_prefix);
    assert_eq!(cache.host, "redis.internal");
    assert_eq!(cache.port, 6_380);
    assert_eq!(cache.username.as_deref(), Some("cache-user"));
    assert_eq!(cache.password.as_deref(), Some("redis-secret"));
    assert_eq!(cache.database, Some(4));
}

#[test]
fn secure_redis_connection_mapping_preserves_the_profile_tls_default() {
    let redis = RedisConnection::new(RedisConnectionInput {
        host: "redis.internal".into(),
        port: 6_379,
        username: None,
        password: None,
        database: None,
        use_tls: true,
    })
    .unwrap();

    assert_eq!(redis_connection_settings(&redis).scheme, InstallationProfile::default().redis.scheme);
}
