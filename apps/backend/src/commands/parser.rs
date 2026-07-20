use crate::BackendResult;

use super::migration_command::migration_command;

const GLOBAL_OPTIONS: [&str; 3] = ["--data-dir", "--config-encryption-key", "--listen"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum BackendCommand {
    Serve,
    Migration(MigrationCommand),
    Secrets(SecretCommand),
    Installation(InstallationCommand),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MigrationCommand {
    Up,
    Status,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SecretCommand {
    Generate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum InstallationCommand {
    Reset,
}

pub(super) fn command_from_args(args: Vec<String>) -> BackendResult<BackendCommand> {
    let parsed = parse_global_options(args)?;
    match parsed.positionals.as_slice() {
        [] => Ok(BackendCommand::Serve),
        [migration, operands @ ..] if migration == "migration" => Ok(BackendCommand::Migration(migration_command(operands)?)),
        [secrets, generate] if secrets == "secrets" && generate == "generate" => {
            reject_bootstrap_options(&parsed, "secrets generate")?;
            Ok(BackendCommand::Secrets(SecretCommand::Generate))
        }
        [installation, reset, confirmation] if installation == "installation" && reset == "reset" && confirmation == "--confirm-reset" => {
            reject_disallowed_reset_options(&parsed)?;
            Ok(BackendCommand::Installation(InstallationCommand::Reset))
        }
        _ => Err(format!("unsupported taco command: {}", parsed.positionals.join(" ")).into()),
    }
}

struct ParsedCommand {
    positionals: Vec<String>,
    global_options: Vec<String>,
}

fn parse_global_options(args: Vec<String>) -> BackendResult<ParsedCommand> {
    let mut positionals = Vec::new();
    let mut global_options = Vec::new();
    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        if GLOBAL_OPTIONS.contains(&argument.as_str()) {
            args.next().ok_or_else(|| format!("{argument} requires a value"))?;
            global_options.push(argument);
            continue;
        }
        positionals.push(argument);
    }
    Ok(ParsedCommand { positionals, global_options })
}

fn reject_bootstrap_options(parsed: &ParsedCommand, command: &str) -> BackendResult<()> {
    if parsed.global_options.is_empty() {
        return Ok(());
    }
    Err(format!("{command} does not accept bootstrap options").into())
}

fn reject_disallowed_reset_options(parsed: &ParsedCommand) -> BackendResult<()> {
    let invalid = parsed.global_options.iter().any(|option| option != "--data-dir");
    if invalid {
        return Err("installation reset accepts only --data-dir".into());
    }
    Ok(())
}
