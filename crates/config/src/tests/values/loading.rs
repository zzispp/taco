use std::{ffi::OsString, path::PathBuf};

use super::*;

#[test]
fn explicit_config_path_reads_path_after_config_arg() {
    let args = vec![OsString::from("backend"), OsString::from("--config"), OsString::from("custom.yaml")];

    assert_eq!(crate::loader::explicit_config_path(&args).unwrap(), PathBuf::from("custom.yaml"));
}

#[test]
fn explicit_config_path_errors_without_value() {
    let args = vec![OsString::from("backend"), OsString::from("--config")];

    assert!(matches!(crate::loader::explicit_config_path(&args), Err(SettingsError::MissingConfigArgument)));
}

#[test]
fn settings_loading_requires_config_argument() {
    let args = vec![OsString::from("backend")];

    assert!(matches!(Settings::load_from_args(args), Err(SettingsError::MissingConfigArgument)));
}

#[test]
fn settings_loading_rejects_all_ambient_postgres_variables() {
    struct PresentEnvironment(&'static str);

    impl EnvironmentReader for PresentEnvironment {
        fn read(&self, variable: &str) -> Result<Option<String>, EnvironmentReadError> {
            Ok((variable == self.0).then(|| "must-not-be-used".into()))
        }
    }

    for variable in crate::loader::FORBIDDEN_POSTGRES_ENVIRONMENT_VARIABLES {
        let error = crate::loader::reject_postgres_environment(&PresentEnvironment(variable)).unwrap_err();
        assert!(
            matches!(error, SettingsError::ConflictingPostgresEnvironmentVariable(actual) if actual == variable),
            "{variable}"
        );
    }
}
