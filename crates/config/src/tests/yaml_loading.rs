use std::fs;

use tempfile::NamedTempFile;

use crate::{Settings, SettingsError};

#[test]
fn loads_a_complete_yaml_configuration_from_an_explicit_path() {
    let file = write_config(&valid_yaml());
    let settings = Settings::load_from_args(["taco", "migration", "status", "--config", file.path().to_str().unwrap()]).unwrap();

    assert_eq!(settings.data_directory, std::path::PathBuf::from("/var/lib/taco"));
    assert!(settings.database.auto_migrate);
    assert_eq!(settings.database.password, "database-password");
}

#[test]
fn yaml_loader_requires_one_explicit_configuration_path() {
    assert!(matches!(Settings::load_from_args(["taco"]), Err(SettingsError::MissingConfigArgument)));

    let file = write_config(&valid_yaml());
    let error = Settings::load_from_args(["taco", "--config", file.path().to_str().unwrap(), "--config", file.path().to_str().unwrap()]).unwrap_err();

    assert!(matches!(error, SettingsError::RepeatedConfigArgument));
}

#[test]
fn yaml_loader_reports_the_missing_configuration_file_path() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("missing.yaml");
    let error = Settings::load_from_args(["taco", "--config", path.to_str().unwrap()]).unwrap_err();

    assert!(matches!(error, SettingsError::ReadConfiguration { path: actual, .. } if actual == path.display().to_string()));
}

#[test]
fn yaml_loader_rejects_missing_and_unknown_fields() {
    let missing = write_config(&valid_yaml().replace("  username: null\n", ""));
    let missing_error = Settings::load_from_args(["taco", "--config", missing.path().to_str().unwrap()]).unwrap_err();
    assert!(matches!(missing_error, SettingsError::MissingConfigField("redis.username")));

    let missing_auto_migrate = write_config(&valid_yaml().replace("  auto_migrate: true\n", ""));
    let auto_migrate_error = Settings::load_from_args(["taco", "--config", missing_auto_migrate.path().to_str().unwrap()]).unwrap_err();
    assert!(matches!(auto_migrate_error, SettingsError::Yaml(_)));

    let unknown = write_config(&valid_yaml().replace("  port: 3000", "  port: 3000\n  unexpected: true"));
    let unknown_error = Settings::load_from_args(["taco", "--config", unknown.path().to_str().unwrap()]).unwrap_err();
    assert!(matches!(unknown_error, SettingsError::Yaml(_)));
}

#[test]
fn yaml_loader_resolves_relative_data_directories_from_the_configuration_file_directory() {
    let temporary_directory = tempfile::tempdir().unwrap();
    let project_directory = temporary_directory.path().join("project");
    let configuration_directory = project_directory.join("config");
    let data_directory = project_directory.join("local-data");
    fs::create_dir_all(&configuration_directory).unwrap();
    fs::create_dir(&data_directory).unwrap();
    let configuration_path = configuration_directory.join("config.yaml");
    fs::write(&configuration_path, valid_yaml().replace("/var/lib/taco", "../local-data")).unwrap();

    let settings = Settings::load_from_args(["taco", "--config", configuration_path.to_str().unwrap()]).unwrap();

    assert!(settings.data_directory.is_absolute());
    assert_eq!(fs::canonicalize(settings.data_directory).unwrap(), fs::canonicalize(data_directory).unwrap());
}

#[test]
fn yaml_loader_absolutizes_relative_configuration_paths_before_resolving_data_directories() {
    let current_directory = std::env::current_dir().unwrap();
    let temporary_directory = tempfile::Builder::new()
        .prefix("configuration-relative-path-")
        .tempdir_in(&current_directory)
        .unwrap();
    let configuration_path = temporary_directory.path().join("config.yaml");
    fs::write(&configuration_path, valid_yaml().replace("/var/lib/taco", "local-data")).unwrap();
    let relative_configuration_path = configuration_path.strip_prefix(&current_directory).unwrap();

    let settings = Settings::load_from_args(["taco", "--config", relative_configuration_path.to_str().unwrap()]).unwrap();

    assert_eq!(settings.data_directory, temporary_directory.path().join("local-data"));
}

#[cfg(unix)]
#[test]
fn yaml_loader_preserves_configuration_file_symlink_base_for_relative_data_directories() {
    use std::os::unix::fs::symlink;

    let temporary_directory = tempfile::tempdir().unwrap();
    let actual_configuration_directory = temporary_directory.path().join("actual");
    let data_directory = temporary_directory.path().join("local-data");
    fs::create_dir(&actual_configuration_directory).unwrap();
    fs::create_dir(&data_directory).unwrap();
    let actual_configuration_path = actual_configuration_directory.join("config.yaml");
    fs::write(&actual_configuration_path, valid_yaml().replace("/var/lib/taco", "local-data")).unwrap();
    let linked_configuration_path = temporary_directory.path().join("config.yaml");
    symlink(&actual_configuration_path, &linked_configuration_path).unwrap();

    let settings = Settings::load_from_args(["taco", "--config", linked_configuration_path.to_str().unwrap()]).unwrap();

    assert_eq!(fs::canonicalize(settings.data_directory).unwrap(), fs::canonicalize(data_directory).unwrap());
}

#[test]
fn yaml_loader_rejects_blank_and_example_data_directory_values() {
    let blank = write_config(&valid_yaml().replace("/var/lib/taco", "''"));
    let blank_error = Settings::load_from_args(["taco", "--config", blank.path().to_str().unwrap()]).unwrap_err();
    assert!(matches!(blank_error, SettingsError::BlankConfigValue("data_directory")));

    let placeholder_directory = write_config(&valid_yaml().replace("/var/lib/taco", "<DATA_DIRECTORY>"));
    let placeholder_directory_error = Settings::load_from_args(["taco", "--config", placeholder_directory.path().to_str().unwrap()]).unwrap_err();
    assert!(matches!(placeholder_directory_error, SettingsError::PlaceholderConfigValue("data_directory")));

    let placeholder = write_config(&valid_yaml().replace("database-password", "<DATABASE_PASSWORD>"));
    let placeholder_error = Settings::load_from_args(["taco", "--config", placeholder.path().to_str().unwrap()]).unwrap_err();
    assert!(matches!(placeholder_error, SettingsError::PlaceholderConfigValue("database.password")));
}

#[test]
fn yaml_loader_rejects_environment_interpolation() {
    let directory = write_config(&valid_yaml().replace("/var/lib/taco", "${DATA_DIRECTORY}"));
    let directory_error = Settings::load_from_args(["taco", "--config", directory.path().to_str().unwrap()]).unwrap_err();
    assert!(matches!(directory_error, SettingsError::EnvironmentInterpolation("data_directory")));

    let file = write_config(&valid_yaml().replace("database-password", "${DATABASE_PASSWORD}"));
    let error = Settings::load_from_args(["taco", "--config", file.path().to_str().unwrap()]).unwrap_err();

    assert!(matches!(error, SettingsError::EnvironmentInterpolation("database.password")));
}

#[test]
fn yaml_loader_rejects_administrator_bootstrap_configuration() {
    let file = write_config(&format!("{}bootstrap_administrator: {{}}\n", valid_yaml()));

    let error = Settings::load_from_args(["taco", "--config", file.path().to_str().unwrap()]).unwrap_err();

    let SettingsError::Yaml(error) = error else {
        panic!("administrator bootstrap configuration must be rejected as an unknown YAML field");
    };
    assert!(error.to_string().contains("unknown field `bootstrap_administrator`"));
}

fn write_config(contents: &str) -> NamedTempFile {
    let file = NamedTempFile::new().unwrap();
    fs::write(file.path(), contents).unwrap();
    file
}

const VALID_YAML: &str = r#"
data_directory: /var/lib/taco
server:
  host: 127.0.0.1
  port: 3000
database:
  scheme: postgres
  ssl_mode: disable
  host: localhost
  port: 5432
  username: taco
  password: database-password
  name: taco
  auto_migrate: true
jwt:
  secret: config-test-jwt-secret-32-bytes!
user:
  online_sessions:
    cleanup_interval_ms: 60000
    cleanup_batch_size: 1000
http:
  request_timeout_ms: 30000
  compression_enabled: true
metrics:
  enabled: true
audit:
  outbox:
    worker_count: 4
    claim_batch_size: 64
    poll_interval_ms: 250
    lease_duration_ms: 30000
    retry_delay_ms: 5000
    cleanup_interval_ms: 3600000
    cleanup_batch_size: 1000
    processed_retention_days: 7
client_info:
  ip_location:
    request_timeout_ms: 3000
redis:
  scheme: redis
  host: localhost
  port: 6379
  username: null
  password: null
  database: null
  protocol: resp3
  key_prefix: 'taco:'
scheduler:
  http_client:
    request_timeout_ms: 30000
  runtime:
    reconcile_interval_ms: 1000
"#;

fn valid_yaml() -> String {
    VALID_YAML.into()
}
