use std::path::PathBuf;

use serde_json::{Value, json};

use super::*;

#[test]
fn installation_profile_round_trips_without_runtime_bootstrap_fields() {
    let profile = valid_installation_profile();
    let value = serde_json::to_value(&profile).unwrap();

    assert!(value.get("server").is_none());
    assert!(value.get("uploads").is_none());
    assert!(value.get("data_dir").is_none());
    assert!(value.get("auth").is_none());
    assert!(value.get("captcha").is_none());
    assert!(value.get("cors").is_none());
    assert!(value["database"].get("auto_migrate").is_none());
    assert_eq!(serde_json::from_value::<InstallationProfile>(value).unwrap(), profile);
}

#[test]
fn installation_profile_rejects_unknown_root_and_nested_fields() {
    let profile = valid_installation_profile();
    let mut root = serde_json::to_value(&profile).unwrap();
    root.as_object_mut().unwrap().insert("unexpected".into(), json!(true));

    let mut nested = serde_json::to_value(profile).unwrap();
    nested["database"]["unexpected"] = json!(true);

    assert!(serde_json::from_value::<InstallationProfile>(root).is_err());
    assert!(serde_json::from_value::<InstallationProfile>(nested).is_err());
}

#[test]
fn installation_profile_rejects_missing_required_sections() {
    let mut value = serde_json::to_value(valid_installation_profile()).unwrap();
    value.as_object_mut().unwrap().remove("scheduler");

    assert!(serde_json::from_value::<InstallationProfile>(value).is_err());
}

#[test]
fn default_profile_contains_only_installation_form_and_operational_defaults() {
    let profile = InstallationProfile::default();

    assert_eq!(profile.database.host, "");
    assert_eq!(profile.database.username, "");
    assert_eq!(profile.database.password, "");
    assert_eq!(profile.database.name, "");
    assert_eq!(profile.database.scheme, DatabaseScheme::Postgres);
    assert_eq!(profile.database.ssl_mode, DatabaseSslMode::VerifyFull);
    assert_eq!(profile.database.port, 5_432);
    assert_eq!(profile.jwt.secret, "");
    assert_eq!(profile.redis.host, "");
    assert_eq!(profile.redis.port, 6_379);
    assert_eq!(profile.redis.scheme, RedisScheme::Rediss);
    assert_eq!(profile.redis.protocol, Some(RedisProtocol::Resp3));
    assert_eq!(profile.redis.key_prefix, "taco:");
    assert_eq!(profile.http.request_timeout_ms, 30_000);
    assert!(profile.http.compression_enabled);
    assert!(profile.metrics.enabled);
    assert_eq!(profile.user.online_sessions.cleanup_interval_ms, 60_000);
    assert_eq!(profile.user.online_sessions.cleanup_batch_size, 1_000);
    assert_eq!(profile.audit.outbox.worker_count, 4);
    assert_eq!(profile.scheduler.runtime.reconcile_interval_ms, 1_000);
}

#[test]
fn settings_derives_listener_and_avatar_path_from_bootstrap_inputs() {
    let inputs = bootstrap_inputs_at(PathBuf::from("/srv/taco"), "[::1]:4321".parse().unwrap());
    let settings = Settings::from_installation_profile(valid_installation_profile(), &inputs).unwrap();

    assert_eq!(settings.server.host, "::1");
    assert_eq!(settings.server.port, 4_321);
    assert_eq!(settings.bind_addr(), "[::1]:4321");
    assert_eq!(settings.uploads.avatar_directory, "/srv/taco/uploads/avatars");
}

#[test]
fn only_completed_persisted_installations_can_build_runtime_settings() {
    let inputs = bootstrap_inputs();
    let incomplete = PersistedInstallation {
        complete: false,
        profile: valid_installation_profile(),
    };
    let completed = PersistedInstallation::completed(valid_installation_profile());

    assert!(matches!(
        Settings::from_persisted_installation(incomplete, &inputs),
        Err(SettingsError::IncompleteInstallation)
    ));
    assert_eq!(Settings::from_persisted_installation(completed, &inputs).unwrap().database, database_parts());
}

#[test]
fn persisted_installation_round_trips_strictly() {
    let installation = PersistedInstallation::completed(valid_installation_profile());
    let mut value = serde_json::to_value(&installation).unwrap();
    value["profile"]["audit"]["outbox"]["unexpected"] = Value::Bool(true);

    assert!(serde_json::from_value::<PersistedInstallation>(value).is_err());
}
