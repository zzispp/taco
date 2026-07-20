use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf};

use crate::{BootstrapInputError, BootstrapInputs, ConfigEncryptionKey, DataDirectory, EnvironmentReadError, EnvironmentReader};

const KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

#[derive(Default)]
struct MapEnvironment {
    values: BTreeMap<&'static str, &'static str>,
}

impl MapEnvironment {
    fn with(values: [(&'static str, &'static str); 1]) -> Self {
        Self {
            values: values.into_iter().collect(),
        }
    }
}

impl EnvironmentReader for MapEnvironment {
    fn read(&self, variable: &str) -> Result<Option<String>, EnvironmentReadError> {
        Ok(self.values.get(variable).map(|value| (*value).to_owned()))
    }
}

#[test]
fn data_directory_is_independently_resolved_for_reset_commands() {
    let directory =
        DataDirectory::load_from_args_with_environment(["taco", "installation", "reset", "--data-dir", "/var/lib/taco"], &MapEnvironment::default()).unwrap();

    assert_eq!(directory.as_path(), PathBuf::from("/var/lib/taco"));
}

#[test]
fn data_directory_rejects_missing_and_conflicting_sources() {
    assert_eq!(
        DataDirectory::load_from_args_with_environment(["taco"], &MapEnvironment::default()),
        Err(BootstrapInputError::MissingInput("--data-dir"))
    );
    assert_eq!(
        DataDirectory::load_from_args_with_environment(["taco", "--data-dir", "/cli"], &MapEnvironment::with([("TACO_DATA_DIR", "/environment")]),),
        Err(BootstrapInputError::ConflictingSources {
            argument: "--data-dir",
            environment_variable: "TACO_DATA_DIR",
        })
    );
}

#[test]
fn root_key_is_strict_base64url_and_round_trips_without_display() {
    let key = ConfigEncryptionKey::parse(KEY).unwrap();

    assert_eq!(key.encode(), KEY);
    assert!(matches!(
        ConfigEncryptionKey::parse("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="),
        Err(BootstrapInputError::InvalidConfigEncryptionKey)
    ));
    assert!(matches!(
        ConfigEncryptionKey::parse("AAAA"),
        Err(BootstrapInputError::InvalidConfigEncryptionKey)
    ));
}

#[test]
fn root_key_rejects_cli_and_environment_sources_together() {
    let result = ConfigEncryptionKey::load_from_args_with_environment(
        ["taco", "--config-encryption-key", KEY],
        &MapEnvironment::with([("TACO_CONFIG_ENCRYPTION_KEY", KEY)]),
    );
    let error = key_error(result);

    assert_eq!(
        error,
        BootstrapInputError::ConflictingSources {
            argument: "--config-encryption-key",
            environment_variable: "TACO_CONFIG_ENCRYPTION_KEY",
        }
    );
}

#[test]
fn full_bootstrap_inputs_ignore_command_words_and_default_listen_address() {
    let inputs = BootstrapInputs::load_from_args_with_environment(
        ["taco", "migration", "status", "--data-dir", "/state", "--config-encryption-key", KEY],
        &MapEnvironment::default(),
    )
    .unwrap();

    assert_eq!(inputs.data_dir.as_path(), PathBuf::from("/state"));
    assert_eq!(inputs.config_encryption_key.encode(), KEY);
    assert_eq!(inputs.listen_addr, "0.0.0.0:3000".parse::<SocketAddr>().unwrap());
}

#[test]
fn bootstrap_inputs_resolve_environment_and_reject_listen_source_conflicts() {
    let environment = MapEnvironment::with([("TACO_LISTEN_ADDR", "127.0.0.1:3100")]);
    let inputs = BootstrapInputs::load_from_args_with_environment(["taco", "--data-dir", "/state", "--config-encryption-key", KEY], &environment).unwrap();
    assert_eq!(inputs.listen_addr, "127.0.0.1:3100".parse::<SocketAddr>().unwrap());

    let conflicting_environment = MapEnvironment::with([("TACO_LISTEN_ADDR", "127.0.0.1:3100")]);
    let result = BootstrapInputs::load_from_args_with_environment(
        ["taco", "--data-dir", "/state", "--config-encryption-key", KEY, "--listen", "0.0.0.0:3000"],
        &conflicting_environment,
    );
    let error = bootstrap_error(result);
    assert_eq!(
        error,
        BootstrapInputError::ConflictingSources {
            argument: "--listen",
            environment_variable: "TACO_LISTEN_ADDR",
        }
    );
}

#[test]
fn bootstrap_inputs_reject_invalid_key_without_echoing_it() {
    let invalid_key = "not-a-valid-key";
    let result = BootstrapInputs::load_from_args_with_environment(
        ["taco", "--data-dir", "/state", "--config-encryption-key", invalid_key],
        &MapEnvironment::default(),
    );
    let error = bootstrap_error(result);

    assert_eq!(error, BootstrapInputError::InvalidConfigEncryptionKey);
    assert!(!error.to_string().contains(invalid_key));
}

fn bootstrap_error(result: Result<BootstrapInputs, BootstrapInputError>) -> BootstrapInputError {
    match result {
        Ok(_) => panic!("bootstrap inputs should fail"),
        Err(error) => error,
    }
}

fn key_error(result: Result<ConfigEncryptionKey, BootstrapInputError>) -> BootstrapInputError {
    match result {
        Ok(_) => panic!("configuration encryption key should fail"),
        Err(error) => error,
    }
}
