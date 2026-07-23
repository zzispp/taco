use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

use super::{
    parser::{AdministratorBootstrapInput, AdministratorCommand, BackendCommand, MigrationCommand, SecretCommand, command_from_args},
    secret_command,
};

#[test]
fn defaults_to_serve_command_with_an_explicit_configuration_path() {
    let args = vec!["--config".into(), "config/config.yaml".into()];

    assert_eq!(command_from_args(args).unwrap(), BackendCommand::Serve);
}

#[test]
fn detects_migration_commands_when_config_path_precedes_or_follows_them() {
    let up = vec!["--config".into(), "config/config.yaml".into(), "migration".into(), "up".into()];
    let status = vec!["migration".into(), "status".into(), "--config".into(), "config/config.yaml".into()];

    assert_eq!(command_from_args(up).unwrap(), BackendCommand::Migration(MigrationCommand::Up));
    assert_eq!(command_from_args(status).unwrap(), BackendCommand::Migration(MigrationCommand::Status));
}

#[test]
fn detects_jwt_secret_generation_without_runtime_configuration() {
    let args = vec!["secret".into(), "generate-jwt".into()];

    assert_eq!(command_from_args(args).unwrap(), BackendCommand::Secret(SecretCommand::GenerateJwt));
}

#[test]
fn rejects_configuration_for_jwt_secret_generation() {
    for args in [
        vec!["secret".into(), "generate-jwt".into(), "--config".into(), "config/config.yaml".into()],
        vec!["--config".into(), "config/config.yaml".into(), "secret".into(), "generate-jwt".into()],
    ] {
        assert_eq!(command_from_args(args).unwrap_err().to_string(), "secret generate-jwt does not accept --config");
    }
}

#[test]
fn encodes_a_fixed_jwt_secret_as_unpadded_base64url() {
    let encoded = secret_command::encode_jwt_secret(&[0; secret_command::JWT_SECRET_BYTES]);

    assert_eq!(encoded, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
}

#[test]
fn generates_a_single_line_jwt_secret_from_32_random_bytes() {
    let secret = secret_command::generate_encoded_jwt_secret().unwrap();
    let mut output = Vec::new();

    secret_command::write_jwt_secret(&mut output, &secret).unwrap();

    assert_eq!(secret.len(), 43);
    assert!(!secret.contains('='));
    assert_eq!(URL_SAFE_NO_PAD.decode(&secret).unwrap().len(), secret_command::JWT_SECRET_BYTES);
    assert_eq!(String::from_utf8(output).unwrap(), format!("{secret}\n"));
}

#[test]
fn detects_administrator_bootstrap_with_identity_options() {
    let args = vec![
        "administrator".into(),
        "bootstrap".into(),
        "--email".into(),
        "admin@example.test".into(),
        "--config".into(),
        "config/config.yaml".into(),
        "--username".into(),
        "admin".into(),
        "--password-stdin".into(),
    ];

    assert_eq!(
        command_from_args(args).unwrap(),
        BackendCommand::Administrator(AdministratorCommand::Bootstrap(AdministratorBootstrapInput {
            username: "admin".into(),
            email: "admin@example.test".into(),
        }))
    );
}

#[test]
fn rejects_unknown_commands() {
    for args in [vec!["obsolete".into()], vec!["unrecognized".into(), "command".into()]] {
        assert!(command_from_args(args).is_err());
    }
}

#[test]
fn rejects_unknown_options_and_missing_config_values() {
    for option in ["--unknown", "--legacy-option"] {
        let args = vec![option.into(), "value".into()];
        assert!(command_from_args(args).is_err());
    }

    assert!(command_from_args(vec!["--config".into()]).is_err());
}

#[test]
fn rejects_invalid_administrator_bootstrap_options() {
    let missing_email = vec![
        "administrator".into(),
        "bootstrap".into(),
        "--username".into(),
        "admin".into(),
        "--password-stdin".into(),
    ];
    let password = vec![
        "administrator".into(),
        "bootstrap".into(),
        "--username".into(),
        "admin".into(),
        "--email".into(),
        "admin@example.test".into(),
        "--password-stdin".into(),
        "--password".into(),
        "not-accepted".into(),
    ];
    let duplicated_username = vec![
        "administrator".into(),
        "bootstrap".into(),
        "--username".into(),
        "admin".into(),
        "--username".into(),
        "another-admin".into(),
        "--email".into(),
        "admin@example.test".into(),
        "--password-stdin".into(),
    ];

    for args in [missing_email, password, duplicated_username] {
        assert!(command_from_args(args).is_err());
    }
}

#[test]
fn rejects_reverse_migration_and_unknown_commands() {
    assert!(command_from_args(vec!["migration".into(), "down".into()]).is_err());
    assert!(command_from_args(vec!["unsupported-command".into()]).is_err());
}
