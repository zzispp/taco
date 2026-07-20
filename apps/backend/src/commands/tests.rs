use super::parser::{BackendCommand, InstallationCommand, MigrationCommand, SecretCommand, command_from_args};

#[test]
fn defaults_to_serve_command() {
    assert_eq!(command_from_args(vec![]).unwrap(), BackendCommand::Serve);
}

#[test]
fn detects_forward_migration_commands() {
    let up = vec!["migration".into(), "up".into()];
    let status = vec!["migration".into(), "status".into()];

    assert_eq!(command_from_args(up).unwrap(), BackendCommand::Migration(MigrationCommand::Up));
    assert_eq!(command_from_args(status).unwrap(), BackendCommand::Migration(MigrationCommand::Status));
}

#[test]
fn recognizes_secret_generation_without_runtime_configuration() {
    let args = vec!["secrets".into(), "generate".into()];

    assert_eq!(command_from_args(args).unwrap(), BackendCommand::Secrets(SecretCommand::Generate));
}

#[test]
fn recognizes_configuration_reset_without_runtime_configuration() {
    let args = vec!["installation".into(), "reset".into(), "--confirm-reset".into()];

    assert_eq!(command_from_args(args).unwrap(), BackendCommand::Installation(InstallationCommand::Reset));
}

#[test]
fn accepts_data_directory_with_reset_and_strips_it_from_command_positionals() {
    let args = vec![
        "--data-dir".into(),
        "/var/lib/taco".into(),
        "installation".into(),
        "reset".into(),
        "--confirm-reset".into(),
    ];

    assert_eq!(command_from_args(args).unwrap(), BackendCommand::Installation(InstallationCommand::Reset));
}

#[test]
fn recognizes_explicit_installation_recovery_commands() {
    let reconfigure = vec![
        "installation".into(),
        "reconfigure".into(),
        "--connections".into(),
        "/operator/connections.json".into(),
    ];
    let recover = vec!["installation".into(), "recover".into(), "--profile".into(), "/operator/profile.json".into()];

    assert_eq!(
        command_from_args(reconfigure).unwrap(),
        BackendCommand::Installation(InstallationCommand::Reconfigure {
            connections_path: "/operator/connections.json".into(),
        })
    );
    assert_eq!(
        command_from_args(recover).unwrap(),
        BackendCommand::Installation(InstallationCommand::Recover {
            profile_path: "/operator/profile.json".into(),
        })
    );
}

#[test]
fn recognizes_profile_template_without_bootstrap_configuration() {
    let args = vec!["installation".into(), "profile".into(), "template".into()];

    assert_eq!(
        command_from_args(args).unwrap(),
        BackendCommand::Installation(InstallationCommand::ProfileTemplate)
    );
}

#[test]
fn rejects_reverse_migration_commands() {
    for command in ["down", "fresh", "refresh", "reset"] {
        let args = vec!["migration".into(), command.into()];

        assert!(command_from_args(args).is_err());
    }
}

#[test]
fn rejects_unknown_commands() {
    let args = vec!["unsupported-command".into()];

    assert!(command_from_args(args).is_err());
}

#[test]
fn rejects_bootstrap_options_for_secret_generation() {
    let args = vec!["--data-dir".into(), "/var/lib/taco".into(), "secrets".into(), "generate".into()];

    assert!(command_from_args(args).is_err());
}

#[test]
fn requires_explicit_reset_confirmation() {
    let args = vec!["installation".into(), "reset".into()];

    assert!(command_from_args(args).is_err());
}
