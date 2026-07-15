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
